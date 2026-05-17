//! Source text table and diagnostic path labels.
//!
//! The compiler core only needs stable source labels for diagnostics. Host
//! filesystem paths are converted to strings by the loader layer so this module
//! can stay `no_std` + `alloc`.

use alloc::collections::BTreeSet;
use alloc::string::String;
use alloc::vec::Vec;
use core::fmt;

use crate::effects::{LlvmRawBodyMemoryOp, RawBodyMemoryOp, RawMemoryOp, WasmRawBodyMemoryOp};
use crate::span::{FileId, Span};

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
    RawMemoryStructuralBoundary,
    RawAddressViewBoundary,
    RawMemoryOperationBoundary(RawMemoryOp),
    RawBodyMemoryOperationBoundary(RawBodyMemoryOp),
    OwnerAggregateConstructorBoundary(String),
    OwnerAggregateFieldBoundary,
    CompilerMemoryFieldBoundary,
    CompilerMemoryTypeDefinition(CompilerMemoryType),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SourceCapabilitySpan {
    start: u32,
    end: u32,
}

impl SourceCapabilitySpan {
    pub fn from_span(span: Span) -> Self {
        Self {
            start: span.start,
            end: span.end,
        }
    }
}

/// Compiler-owned privilege proven for one syntactic use site.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SourceCapabilityUseSite {
    RawMemoryStructuralBoundary {
        span: SourceCapabilitySpan,
    },
    RawAddressViewBoundary {
        span: SourceCapabilitySpan,
    },
    RawMemoryOperationBoundary {
        operation: RawMemoryOp,
        span: SourceCapabilitySpan,
    },
    RawBodyMemoryOperationBoundary {
        operation: RawBodyMemoryOp,
        span: SourceCapabilitySpan,
    },
    OwnerAggregateConstructorBoundary {
        name: String,
        span: SourceCapabilitySpan,
    },
    OwnerAggregateFieldBoundary {
        span: SourceCapabilitySpan,
    },
    CompilerMemoryFieldBoundary {
        span: SourceCapabilitySpan,
    },
    CompilerMemoryTypeDefinition {
        memory_type: CompilerMemoryType,
        span: SourceCapabilitySpan,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CompilerMemoryType {
    RawPointer,
    OwnerToken,
}

const ALL_RAW_MEMORY_OPS: &[RawMemoryOp] = &[
    RawMemoryOp::Alloc,
    RawMemoryOp::Dealloc,
    RawMemoryOp::Realloc,
    RawMemoryOp::Load,
    RawMemoryOp::Store,
    RawMemoryOp::BulkCopy,
    RawMemoryOp::BulkMove,
    RawMemoryOp::MemorySize,
    RawMemoryOp::MemoryGrow,
    RawMemoryOp::FillBytes,
    RawMemoryOp::Fill,
];

const ALL_RAW_BODY_MEMORY_OPS: &[RawBodyMemoryOp] = &[
    RawBodyMemoryOp::Wasm(WasmRawBodyMemoryOp::Load),
    RawBodyMemoryOp::Wasm(WasmRawBodyMemoryOp::Store),
    RawBodyMemoryOp::Wasm(WasmRawBodyMemoryOp::MemorySize),
    RawBodyMemoryOp::Wasm(WasmRawBodyMemoryOp::MemoryGrow),
    RawBodyMemoryOp::Wasm(WasmRawBodyMemoryOp::MemoryCopy),
    RawBodyMemoryOp::Wasm(WasmRawBodyMemoryOp::MemoryFill),
    RawBodyMemoryOp::Wasm(WasmRawBodyMemoryOp::MemoryInit),
    RawBodyMemoryOp::Wasm(WasmRawBodyMemoryOp::DataDrop),
    RawBodyMemoryOp::Wasm(WasmRawBodyMemoryOp::Memory),
    RawBodyMemoryOp::Llvm(LlvmRawBodyMemoryOp::Alloca),
    RawBodyMemoryOp::Llvm(LlvmRawBodyMemoryOp::Load),
    RawBodyMemoryOp::Llvm(LlvmRawBodyMemoryOp::Store),
    RawBodyMemoryOp::Llvm(LlvmRawBodyMemoryOp::AtomicRmw),
    RawBodyMemoryOp::Llvm(LlvmRawBodyMemoryOp::Cmpxchg),
    RawBodyMemoryOp::Llvm(LlvmRawBodyMemoryOp::Fence),
    RawBodyMemoryOp::Llvm(LlvmRawBodyMemoryOp::Memcpy),
    RawBodyMemoryOp::Llvm(LlvmRawBodyMemoryOp::Memmove),
    RawBodyMemoryOp::Llvm(LlvmRawBodyMemoryOp::Memset),
];

/// Compiler-owned privileges attached to a source file.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SourceCapabilities {
    capabilities: BTreeSet<SourceCapability>,
    use_sites: BTreeSet<SourceCapabilityUseSite>,
}

impl SourceCapabilities {
    pub fn none() -> Self {
        Self {
            capabilities: BTreeSet::new(),
            use_sites: BTreeSet::new(),
        }
    }

    pub fn raw_memory_boundary() -> Self {
        let mut capabilities = Self::none();
        capabilities.insert(SourceCapability::RawMemoryStructuralBoundary);
        capabilities.insert(SourceCapability::RawAddressViewBoundary);
        for operation in ALL_RAW_MEMORY_OPS {
            capabilities.insert(SourceCapability::RawMemoryOperationBoundary(*operation));
        }
        for operation in ALL_RAW_BODY_MEMORY_OPS {
            capabilities.insert(SourceCapability::RawBodyMemoryOperationBoundary(*operation));
        }
        capabilities
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
        Self {
            capabilities,
            use_sites: BTreeSet::new(),
        }
    }

    pub fn insert(&mut self, capability: SourceCapability) {
        self.capabilities.insert(capability);
    }

    pub(crate) fn insert_use_site(&mut self, use_site: SourceCapabilityUseSite) {
        self.use_sites.insert(use_site);
    }

    pub fn allows(&self, capability: SourceCapability) -> bool {
        self.capabilities.contains(&capability)
    }

    fn allows_use_site(&self, use_site: SourceCapabilityUseSite) -> bool {
        self.use_sites.contains(&use_site)
    }

    pub fn allows_raw_memory_structural_boundary(&self) -> bool {
        self.allows(SourceCapability::RawMemoryStructuralBoundary)
            || self.use_sites.iter().any(|site| {
                matches!(
                    site,
                    SourceCapabilityUseSite::RawMemoryStructuralBoundary { .. }
                )
            })
    }

    pub fn allows_raw_address_view_boundary(&self) -> bool {
        self.allows(SourceCapability::RawAddressViewBoundary)
            || self
                .use_sites
                .iter()
                .any(|site| matches!(site, SourceCapabilityUseSite::RawAddressViewBoundary { .. }))
    }

    pub fn allows_raw_memory_operation_boundary(&self, operation: RawMemoryOp) -> bool {
        self.allows(SourceCapability::RawMemoryOperationBoundary(operation))
            || self.use_sites.iter().any(|site| {
                matches!(
                    site,
                    SourceCapabilityUseSite::RawMemoryOperationBoundary { operation: site_op, .. }
                        if *site_op == operation
                )
            })
    }

    pub fn allows_raw_body_memory_operation_boundary(&self, operation: RawBodyMemoryOp) -> bool {
        self.allows(SourceCapability::RawBodyMemoryOperationBoundary(operation))
            || self.use_sites.iter().any(|site| {
                matches!(
                    site,
                    SourceCapabilityUseSite::RawBodyMemoryOperationBoundary {
                        operation: site_op,
                        ..
                    } if *site_op == operation
                )
            })
    }

    pub fn allows_owner_aggregate_constructor_boundary(&self, name: &str) -> bool {
        self.allows(SourceCapability::OwnerAggregateConstructorBoundary(
            String::from(name),
        )) || self.use_sites.iter().any(|site| {
            matches!(
                site,
                SourceCapabilityUseSite::OwnerAggregateConstructorBoundary {
                    name: site_name,
                    ..
                } if site_name == name
            )
        })
    }

    pub fn allows_owner_aggregate_field_boundary(&self) -> bool {
        self.allows(SourceCapability::OwnerAggregateFieldBoundary)
            || self.use_sites.iter().any(|site| {
                matches!(
                    site,
                    SourceCapabilityUseSite::OwnerAggregateFieldBoundary { .. }
                )
            })
    }

    pub fn allows_compiler_memory_field_boundary(&self) -> bool {
        self.allows(SourceCapability::CompilerMemoryFieldBoundary)
            || self.use_sites.iter().any(|site| {
                matches!(
                    site,
                    SourceCapabilityUseSite::CompilerMemoryFieldBoundary { .. }
                )
            })
    }

    pub fn allows_compiler_memory_type_definition(&self, memory_type: CompilerMemoryType) -> bool {
        self.allows(SourceCapability::CompilerMemoryTypeDefinition(memory_type))
            || self.use_sites.iter().any(|site| {
                matches!(
                    site,
                    SourceCapabilityUseSite::CompilerMemoryTypeDefinition {
                        memory_type: site_type,
                        ..
                    } if *site_type == memory_type
                )
            })
    }

    pub fn allows_raw_memory_structural_boundary_at(&self, span: Span) -> bool {
        self.allows_use_site(SourceCapabilityUseSite::RawMemoryStructuralBoundary {
            span: SourceCapabilitySpan::from_span(span),
        })
    }

    pub fn allows_raw_address_view_boundary_at(&self, span: Span) -> bool {
        self.allows_use_site(SourceCapabilityUseSite::RawAddressViewBoundary {
            span: SourceCapabilitySpan::from_span(span),
        })
    }

    pub fn allows_raw_memory_operation_boundary_at(
        &self,
        operation: RawMemoryOp,
        span: Span,
    ) -> bool {
        self.allows_use_site(SourceCapabilityUseSite::RawMemoryOperationBoundary {
            operation,
            span: SourceCapabilitySpan::from_span(span),
        })
    }

    pub fn allows_raw_body_memory_operation_boundary_at(
        &self,
        operation: RawBodyMemoryOp,
        span: Span,
    ) -> bool {
        self.allows_use_site(SourceCapabilityUseSite::RawBodyMemoryOperationBoundary {
            operation,
            span: SourceCapabilitySpan::from_span(span),
        })
    }

    pub fn allows_owner_aggregate_constructor_boundary_at(&self, name: &str, span: Span) -> bool {
        self.allows_use_site(SourceCapabilityUseSite::OwnerAggregateConstructorBoundary {
            name: String::from(name),
            span: SourceCapabilitySpan::from_span(span),
        })
    }

    pub fn allows_owner_aggregate_field_boundary_at(&self, span: Span) -> bool {
        self.allows_use_site(SourceCapabilityUseSite::OwnerAggregateFieldBoundary {
            span: SourceCapabilitySpan::from_span(span),
        })
    }

    pub fn allows_compiler_memory_field_boundary_at(&self, span: Span) -> bool {
        self.allows_use_site(SourceCapabilityUseSite::CompilerMemoryFieldBoundary {
            span: SourceCapabilitySpan::from_span(span),
        })
    }

    pub fn allows_compiler_memory_type_definition_at(
        &self,
        memory_type: CompilerMemoryType,
        span: Span,
    ) -> bool {
        self.allows_use_site(SourceCapabilityUseSite::CompilerMemoryTypeDefinition {
            memory_type,
            span: SourceCapabilitySpan::from_span(span),
        })
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

    pub fn raw_memory_structural_boundary_allowed(&self, id: FileId) -> bool {
        self.capabilities(id)
            .allows_raw_memory_structural_boundary()
    }

    pub fn raw_address_view_boundary_allowed(&self, id: FileId) -> bool {
        self.capabilities(id).allows_raw_address_view_boundary()
    }

    pub fn raw_memory_operation_boundary_allowed(
        &self,
        id: FileId,
        operation: RawMemoryOp,
    ) -> bool {
        self.capabilities(id)
            .allows_raw_memory_operation_boundary(operation)
    }

    pub fn raw_body_memory_operation_boundary_allowed(
        &self,
        id: FileId,
        operation: RawBodyMemoryOp,
    ) -> bool {
        self.capabilities(id)
            .allows_raw_body_memory_operation_boundary(operation)
    }

    pub fn owner_aggregate_constructor_boundary_allowed(&self, id: FileId, name: &str) -> bool {
        self.capabilities(id)
            .allows_owner_aggregate_constructor_boundary(name)
    }

    pub fn owner_aggregate_field_boundary_allowed(&self, id: FileId) -> bool {
        self.capabilities(id)
            .allows_owner_aggregate_field_boundary()
    }

    pub fn compiler_memory_field_boundary_allowed(&self, id: FileId) -> bool {
        self.capabilities(id)
            .allows_compiler_memory_field_boundary()
    }

    pub fn compiler_memory_type_definition_allowed(
        &self,
        id: FileId,
        memory_type: CompilerMemoryType,
    ) -> bool {
        self.capabilities(id)
            .allows_compiler_memory_type_definition(memory_type)
    }

    pub fn raw_memory_structural_boundary_allowed_at(&self, span: Span) -> bool {
        self.capabilities(span.file_id)
            .allows_raw_memory_structural_boundary_at(span)
    }

    pub fn raw_address_view_boundary_allowed_at(&self, span: Span) -> bool {
        self.capabilities(span.file_id)
            .allows_raw_address_view_boundary_at(span)
    }

    pub fn raw_memory_operation_boundary_allowed_at(
        &self,
        span: Span,
        operation: RawMemoryOp,
    ) -> bool {
        self.capabilities(span.file_id)
            .allows_raw_memory_operation_boundary_at(operation, span)
    }

    pub fn raw_body_memory_operation_boundary_allowed_at(
        &self,
        span: Span,
        operation: RawBodyMemoryOp,
    ) -> bool {
        self.capabilities(span.file_id)
            .allows_raw_body_memory_operation_boundary_at(operation, span)
    }

    pub fn owner_aggregate_constructor_boundary_allowed_at(&self, span: Span, name: &str) -> bool {
        self.capabilities(span.file_id)
            .allows_owner_aggregate_constructor_boundary_at(name, span)
    }

    pub fn owner_aggregate_field_boundary_allowed_at(&self, span: Span) -> bool {
        self.capabilities(span.file_id)
            .allows_owner_aggregate_field_boundary_at(span)
    }

    pub fn compiler_memory_field_boundary_allowed_at(&self, span: Span) -> bool {
        self.capabilities(span.file_id)
            .allows_compiler_memory_field_boundary_at(span)
    }

    pub fn compiler_memory_type_definition_allowed_at(
        &self,
        span: Span,
        memory_type: CompilerMemoryType,
    ) -> bool {
        self.capabilities(span.file_id)
            .allows_compiler_memory_type_definition_at(memory_type, span)
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

    use crate::effects::RawMemoryOp;
    use crate::span::Span;

    use super::{
        CompilerMemoryType, SourceCapabilities, SourceCapability, SourceCapabilitySpan,
        SourceCapabilityUseSite, SourceMap,
    };

    #[test]
    fn source_capabilities_are_enum_keyed() {
        let none = SourceCapabilities::none();
        assert!(!none.allows(SourceCapability::RawMemoryStructuralBoundary));
        assert!(!none.allows(SourceCapability::RawAddressViewBoundary));
        assert!(!none.allows_raw_memory_operation_boundary(RawMemoryOp::Load));
        assert!(!none.allows(SourceCapability::CompilerMemoryTypeDefinition(
            CompilerMemoryType::RawPointer,
        )));

        let raw_boundary = SourceCapabilities::raw_memory_boundary();
        assert!(raw_boundary.allows(SourceCapability::RawMemoryStructuralBoundary));
        assert!(raw_boundary.allows(SourceCapability::RawAddressViewBoundary));
        assert!(raw_boundary.allows_raw_memory_structural_boundary());
        assert!(raw_boundary.allows_raw_address_view_boundary());
        assert!(raw_boundary.allows_raw_memory_operation_boundary(RawMemoryOp::Load));
        assert!(raw_boundary.allows_raw_memory_operation_boundary(RawMemoryOp::Store));

        let raw_load = SourceCapabilities::with(SourceCapability::RawMemoryOperationBoundary(
            RawMemoryOp::Load,
        ));
        assert!(raw_load.allows_raw_memory_operation_boundary(RawMemoryOp::Load));
        assert!(!raw_load.allows_raw_memory_operation_boundary(RawMemoryOp::Store));
        assert!(!raw_load.allows_raw_memory_structural_boundary());
        assert!(!raw_load.allows_raw_address_view_boundary());

        let owner_constructor = SourceCapabilities::owner_aggregate_constructor_boundary("Vec");
        assert!(owner_constructor.allows_owner_aggregate_constructor_boundary("Vec"));
        assert!(!owner_constructor.allows_owner_aggregate_constructor_boundary("Diag"));
        assert!(!owner_constructor.allows_owner_aggregate_field_boundary());
        assert!(!owner_constructor.allows_raw_memory_structural_boundary());
        assert!(!owner_constructor.allows_raw_address_view_boundary());

        let owner_field = SourceCapabilities::owner_aggregate_field_boundary();
        assert!(owner_field.allows_owner_aggregate_field_boundary());
        assert!(!owner_field.allows_owner_aggregate_constructor_boundary("Vec"));
        assert!(!owner_field.allows_compiler_memory_field_boundary());
        assert!(!owner_field.allows_raw_memory_structural_boundary());
        assert!(!owner_field.allows_raw_address_view_boundary());

        let compiler_field =
            SourceCapabilities::with(SourceCapability::CompilerMemoryFieldBoundary);
        assert!(compiler_field.allows_compiler_memory_field_boundary());
        assert!(!compiler_field.allows_owner_aggregate_field_boundary());

        let address_view = SourceCapabilities::with(SourceCapability::RawAddressViewBoundary);
        assert!(address_view.allows_raw_address_view_boundary());
        assert!(!address_view.allows_raw_memory_structural_boundary());

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
            SourceCapabilities::with(SourceCapability::RawMemoryStructuralBoundary),
        );

        assert!(!source_map.raw_memory_structural_boundary_allowed(plain));
        assert!(!source_map.raw_address_view_boundary_allowed(plain));
        assert!(source_map.raw_memory_structural_boundary_allowed(raw));
        assert!(!source_map.raw_address_view_boundary_allowed(raw));
        assert!(!source_map.raw_memory_operation_boundary_allowed(raw, RawMemoryOp::Load));
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

    #[test]
    fn source_capabilities_keep_source_proof_at_exact_use_site() {
        let mut source_map = SourceMap::new();
        let file = source_map.add("core/mem/pointer/scalar.nepl", String::from(""));
        let proven = Span::new(file, 12, 20);
        let unproven = Span::new(file, 32, 40);
        let mut capabilities = SourceCapabilities::none();
        capabilities.insert_use_site(SourceCapabilityUseSite::RawMemoryOperationBoundary {
            operation: RawMemoryOp::Load,
            span: SourceCapabilitySpan::from_span(proven),
        });
        source_map.set_capabilities(file, capabilities);

        assert!(source_map.raw_memory_operation_boundary_allowed_at(proven, RawMemoryOp::Load));
        assert!(!source_map.raw_memory_operation_boundary_allowed_at(unproven, RawMemoryOp::Load));
        assert!(!source_map.raw_memory_operation_boundary_allowed_at(proven, RawMemoryOp::Store));

        let broad = source_map.add_with_capabilities(
            "test/raw_boundary_fixture.nepl",
            String::from(""),
            SourceCapabilities::with(SourceCapability::RawMemoryOperationBoundary(
                RawMemoryOp::Load,
            )),
        );
        assert!(source_map.raw_memory_operation_boundary_allowed(broad, RawMemoryOp::Load));
        assert!(!source_map.raw_memory_operation_boundary_allowed_at(
            Span::new(broad, 100, 108),
            RawMemoryOp::Load,
        ));
    }
}
