//! Source text table and diagnostic path labels.
//!
//! The compiler core only needs stable source labels for diagnostics. Host
//! filesystem paths are converted to strings by the loader layer so this module
//! can stay `no_std` + `alloc`.

use alloc::collections::BTreeSet;
use alloc::string::String;
use alloc::vec::Vec;
use core::fmt;

use crate::span::FileId;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SourcePath {
    inner: String,
}

impl SourcePath {
    pub fn new(path: impl Into<String>) -> Self {
        Self { inner: path.into() }
    }

    pub fn as_str(&self) -> &str {
        self.inner.as_str()
    }

    pub fn display(&self) -> SourcePathDisplay<'_> {
        SourcePathDisplay(self.as_str())
    }

    pub fn to_string_lossy(&self) -> &str {
        self.as_str()
    }
}

impl From<String> for SourcePath {
    fn from(value: String) -> Self {
        SourcePath::new(value)
    }
}

impl From<&str> for SourcePath {
    fn from(value: &str) -> Self {
        SourcePath::new(value)
    }
}

impl fmt::Display for SourcePath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

pub struct SourcePathDisplay<'a>(&'a str);

impl fmt::Display for SourcePathDisplay<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.0)
    }
}

/// Compiler-owned privilege attached to a source file.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SourceCapability {
    RawMemoryBoundary,
    OwnerAggregateConstructorBoundary(String),
    OwnerAggregateFieldBoundary,
    CompilerMemoryTypeDefinition(CompilerMemoryType),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CompilerMemoryType {
    RawPointer,
    OwnerToken,
}

/// Compiler-owned privileges attached to a source file.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SourceCapabilities {
    capabilities: BTreeSet<SourceCapability>,
}

impl SourceCapabilities {
    pub fn none() -> Self {
        Self {
            capabilities: BTreeSet::new(),
        }
    }

    pub fn raw_memory_boundary() -> Self {
        Self::with(SourceCapability::RawMemoryBoundary)
    }

    pub fn owner_aggregate_constructor_boundary(name: impl Into<String>) -> Self {
        Self::with(SourceCapability::OwnerAggregateConstructorBoundary(
            name.into(),
        ))
    }

    pub fn owner_aggregate_field_boundary() -> Self {
        Self::with(SourceCapability::OwnerAggregateFieldBoundary)
    }

    pub fn compiler_memory_type_definition(memory_type: CompilerMemoryType) -> Self {
        Self::with(SourceCapability::CompilerMemoryTypeDefinition(memory_type))
    }

    pub fn with(capability: SourceCapability) -> Self {
        let mut capabilities = BTreeSet::new();
        capabilities.insert(capability);
        Self { capabilities }
    }

    pub fn insert(&mut self, capability: SourceCapability) {
        self.capabilities.insert(capability);
    }

    pub fn allows(&self, capability: SourceCapability) -> bool {
        self.capabilities.contains(&capability)
    }

    pub fn allows_raw_memory_boundary(&self) -> bool {
        self.allows(SourceCapability::RawMemoryBoundary)
    }

    pub fn allows_owner_aggregate_constructor_boundary(&self, name: &str) -> bool {
        self.allows(SourceCapability::OwnerAggregateConstructorBoundary(
            String::from(name),
        ))
    }

    pub fn allows_owner_aggregate_field_boundary(&self) -> bool {
        self.allows(SourceCapability::OwnerAggregateFieldBoundary)
    }

    pub fn allows_compiler_memory_type_definition(&self, memory_type: CompilerMemoryType) -> bool {
        self.allows(SourceCapability::CompilerMemoryTypeDefinition(memory_type))
    }
}

#[derive(Debug, Clone)]
struct SourceFile {
    path: SourcePath,
    src: String,
    capabilities: SourceCapabilities,
}

/// Holds all loaded sources and their assigned FileId.
#[derive(Debug, Clone, Default)]
pub struct SourceMap {
    files: Vec<SourceFile>,
}

impl SourceMap {
    pub fn new() -> Self {
        Self { files: Vec::new() }
    }

    pub fn path(&self, id: FileId) -> Option<&SourcePath> {
        self.files.get(id.0 as usize).map(|file| &file.path)
    }

    pub fn capabilities(&self, id: FileId) -> SourceCapabilities {
        self.files
            .get(id.0 as usize)
            .map(|file| file.capabilities.clone())
            .unwrap_or_default()
    }

    pub fn set_capabilities(&mut self, id: FileId, capabilities: SourceCapabilities) {
        if let Some(file) = self.files.get_mut(id.0 as usize) {
            file.capabilities = capabilities;
        }
    }

    pub fn raw_memory_boundary_allowed(&self, id: FileId) -> bool {
        self.capabilities(id).allows_raw_memory_boundary()
    }

    pub fn owner_aggregate_constructor_boundary_allowed(&self, id: FileId, name: &str) -> bool {
        self.capabilities(id)
            .allows_owner_aggregate_constructor_boundary(name)
    }

    pub fn owner_aggregate_field_boundary_allowed(&self, id: FileId) -> bool {
        self.capabilities(id)
            .allows_owner_aggregate_field_boundary()
    }

    pub fn compiler_memory_type_definition_allowed(
        &self,
        id: FileId,
        memory_type: CompilerMemoryType,
    ) -> bool {
        self.capabilities(id)
            .allows_compiler_memory_type_definition(memory_type)
    }

    pub fn iter_paths(&self) -> impl Iterator<Item = (FileId, &SourcePath)> {
        self.files
            .iter()
            .enumerate()
            .map(|(idx, file)| (FileId(idx as u32), &file.path))
    }

    /// Convert a byte offset to (line, column) 0-based.
    pub fn line_col(&self, id: FileId, byte: u32) -> Option<(usize, usize)> {
        let src = self.get(id)?;
        let mut line = 0;
        let mut col = 0;
        let mut count = 0;
        for ch in src.bytes() {
            if count as u32 == byte {
                return Some((line, col));
            }
            count += 1;
            if ch == b'\n' {
                line += 1;
                col = 0;
            } else {
                col += 1;
            }
        }
        if count as u32 == byte {
            Some((line, col))
        } else {
            None
        }
    }

    pub fn line_str(&self, id: FileId, line: usize) -> Option<&str> {
        let src = self.get(id)?;
        src.lines().nth(line)
    }

    pub fn get(&self, id: FileId) -> Option<&str> {
        self.files.get(id.0 as usize).map(|file| file.src.as_str())
    }

    pub fn add(&mut self, path: impl Into<SourcePath>, src: String) -> FileId {
        self.add_with_capabilities(path, src, SourceCapabilities::none())
    }

    pub fn add_with_capabilities(
        &mut self,
        path: impl Into<SourcePath>,
        src: String,
        capabilities: SourceCapabilities,
    ) -> FileId {
        let id = self.files.len() as u32;
        self.files.push(SourceFile {
            path: path.into(),
            src,
            capabilities,
        });
        FileId(id)
    }
}

#[cfg(test)]
mod tests {
    use alloc::string::String;

    use super::{CompilerMemoryType, SourceCapabilities, SourceCapability, SourceMap};

    #[test]
    fn source_capabilities_are_enum_keyed() {
        let none = SourceCapabilities::none();
        assert!(!none.allows(SourceCapability::RawMemoryBoundary));
        assert!(!none.allows(SourceCapability::CompilerMemoryTypeDefinition(
            CompilerMemoryType::RawPointer,
        )));

        let raw_boundary = SourceCapabilities::with(SourceCapability::RawMemoryBoundary);
        assert!(raw_boundary.allows(SourceCapability::RawMemoryBoundary));
        assert!(raw_boundary.allows_raw_memory_boundary());

        let owner_constructor = SourceCapabilities::owner_aggregate_constructor_boundary("Vec");
        assert!(owner_constructor.allows_owner_aggregate_constructor_boundary("Vec"));
        assert!(!owner_constructor.allows_owner_aggregate_constructor_boundary("Diag"));
        assert!(!owner_constructor.allows_owner_aggregate_field_boundary());
        assert!(!owner_constructor.allows_raw_memory_boundary());

        let owner_field = SourceCapabilities::owner_aggregate_field_boundary();
        assert!(owner_field.allows_owner_aggregate_field_boundary());
        assert!(!owner_field.allows_owner_aggregate_constructor_boundary("Vec"));
        assert!(!owner_field.allows_raw_memory_boundary());

        let memory_type =
            SourceCapabilities::compiler_memory_type_definition(CompilerMemoryType::OwnerToken);
        assert!(memory_type.allows_compiler_memory_type_definition(CompilerMemoryType::OwnerToken));
        assert!(!memory_type.allows_compiler_memory_type_definition(CompilerMemoryType::RawPointer));
    }

    #[test]
    fn source_map_keeps_capabilities_per_file() {
        let mut source_map = SourceMap::new();
        let plain = source_map.add("plain.nepl", String::from(""));
        let raw = source_map.add_with_capabilities(
            "core/mem.nepl",
            String::from(""),
            SourceCapabilities::with(SourceCapability::RawMemoryBoundary),
        );

        assert!(!source_map.raw_memory_boundary_allowed(plain));
        assert!(source_map.raw_memory_boundary_allowed(raw));
        assert!(!source_map.owner_aggregate_constructor_boundary_allowed(raw, "Vec"));
        assert!(!source_map.owner_aggregate_field_boundary_allowed(raw));

        source_map.set_capabilities(
            plain,
            SourceCapabilities::compiler_memory_type_definition(CompilerMemoryType::RawPointer),
        );
        assert!(source_map
            .compiler_memory_type_definition_allowed(plain, CompilerMemoryType::RawPointer));
        assert!(!source_map
            .compiler_memory_type_definition_allowed(raw, CompilerMemoryType::RawPointer));
    }
}
