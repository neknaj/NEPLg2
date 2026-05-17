//! Source text table and diagnostic path labels.
//!
//! The compiler core only needs stable source labels for diagnostics. Host
//! filesystem paths are converted to strings by the loader layer so this module
//! can stay `no_std` + `alloc`.

use alloc::collections::BTreeSet;
use alloc::string::String;
use alloc::vec::Vec;
use core::fmt;

use crate::effects::{RawBodyMemoryOp, RawMemoryOp};
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

/// Compiler-owned privileges proven from source for one file.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SourceCapabilities {
    use_sites: BTreeSet<SourceCapabilityUseSite>,
}

impl SourceCapabilities {
    pub fn none() -> Self {
        Self {
            use_sites: BTreeSet::new(),
        }
    }

    pub(crate) fn insert_use_site(&mut self, use_site: SourceCapabilityUseSite) {
        self.use_sites.insert(use_site);
    }

    #[cfg(test)]
    pub(crate) fn use_sites_for_tests(&self) -> impl Iterator<Item = &SourceCapabilityUseSite> {
        self.use_sites.iter()
    }

    fn allows_use_site(&self, use_site: SourceCapabilityUseSite) -> bool {
        self.use_sites.contains(&use_site)
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
    use crate::span::{FileId, Span};

    use super::{
        CompilerMemoryType, SourceCapabilities, SourceCapabilitySpan, SourceCapabilityUseSite,
        SourceMap,
    };

    fn use_site_capabilities(use_site: SourceCapabilityUseSite) -> SourceCapabilities {
        let mut capabilities = SourceCapabilities::none();
        capabilities.insert_use_site(use_site);
        capabilities
    }

    fn span(file: FileId) -> Span {
        Span::new(file, 8, 16)
    }

    #[test]
    fn source_capabilities_are_use_site_enum_keyed() {
        let file = FileId(0);
        let proven = span(file);
        let other = Span::new(file, 24, 32);
        let none = SourceCapabilities::none();
        assert!(!none.allows_raw_memory_structural_boundary_at(proven));
        assert!(!none.allows_raw_address_view_boundary_at(proven));
        assert!(!none.allows_raw_memory_operation_boundary_at(RawMemoryOp::Load, proven));
        assert!(!none
            .allows_compiler_memory_type_definition_at(CompilerMemoryType::RawPointer, proven,));

        let raw_boundary =
            use_site_capabilities(SourceCapabilityUseSite::RawMemoryStructuralBoundary {
                span: SourceCapabilitySpan::from_span(proven),
            });
        assert!(raw_boundary.allows_raw_memory_structural_boundary_at(proven));
        assert!(!raw_boundary.allows_raw_memory_structural_boundary_at(other));
        assert!(!raw_boundary.allows_raw_address_view_boundary_at(proven));
        assert!(!raw_boundary.allows_raw_memory_operation_boundary_at(RawMemoryOp::Load, proven));

        let raw_load = use_site_capabilities(SourceCapabilityUseSite::RawMemoryOperationBoundary {
            operation: RawMemoryOp::Load,
            span: SourceCapabilitySpan::from_span(proven),
        });
        assert!(raw_load.allows_raw_memory_operation_boundary_at(RawMemoryOp::Load, proven));
        assert!(!raw_load.allows_raw_memory_operation_boundary_at(RawMemoryOp::Load, other));
        assert!(!raw_load.allows_raw_memory_operation_boundary_at(RawMemoryOp::Store, proven));
        assert!(!raw_load.allows_raw_memory_structural_boundary_at(proven));
        assert!(!raw_load.allows_raw_address_view_boundary_at(proven));

        let owner_constructor =
            use_site_capabilities(SourceCapabilityUseSite::OwnerAggregateConstructorBoundary {
                name: String::from("Vec"),
                span: SourceCapabilitySpan::from_span(proven),
            });
        assert!(owner_constructor.allows_owner_aggregate_constructor_boundary_at("Vec", proven));
        assert!(!owner_constructor.allows_owner_aggregate_constructor_boundary_at("Vec", other));
        assert!(!owner_constructor.allows_owner_aggregate_constructor_boundary_at("Diag", proven));
        assert!(!owner_constructor.allows_owner_aggregate_field_boundary_at(proven));
        assert!(!owner_constructor.allows_raw_memory_structural_boundary_at(proven));
        assert!(!owner_constructor.allows_raw_address_view_boundary_at(proven));

        let owner_field =
            use_site_capabilities(SourceCapabilityUseSite::OwnerAggregateFieldBoundary {
                span: SourceCapabilitySpan::from_span(proven),
            });
        assert!(owner_field.allows_owner_aggregate_field_boundary_at(proven));
        assert!(!owner_field.allows_owner_aggregate_field_boundary_at(other));
        assert!(!owner_field.allows_owner_aggregate_constructor_boundary_at("Vec", proven));
        assert!(!owner_field.allows_compiler_memory_field_boundary_at(proven));
        assert!(!owner_field.allows_raw_memory_structural_boundary_at(proven));
        assert!(!owner_field.allows_raw_address_view_boundary_at(proven));

        let compiler_field =
            use_site_capabilities(SourceCapabilityUseSite::CompilerMemoryFieldBoundary {
                span: SourceCapabilitySpan::from_span(proven),
            });
        assert!(compiler_field.allows_compiler_memory_field_boundary_at(proven));
        assert!(!compiler_field.allows_compiler_memory_field_boundary_at(other));
        assert!(!compiler_field.allows_owner_aggregate_field_boundary_at(proven));

        let address_view = use_site_capabilities(SourceCapabilityUseSite::RawAddressViewBoundary {
            span: SourceCapabilitySpan::from_span(proven),
        });
        assert!(address_view.allows_raw_address_view_boundary_at(proven));
        assert!(!address_view.allows_raw_address_view_boundary_at(other));
        assert!(!address_view.allows_raw_memory_structural_boundary_at(proven));

        let memory_type =
            use_site_capabilities(SourceCapabilityUseSite::CompilerMemoryTypeDefinition {
                memory_type: CompilerMemoryType::OwnerToken,
                span: SourceCapabilitySpan::from_span(proven),
            });
        assert!(memory_type
            .allows_compiler_memory_type_definition_at(CompilerMemoryType::OwnerToken, proven));
        assert!(!memory_type
            .allows_compiler_memory_type_definition_at(CompilerMemoryType::OwnerToken, other));
        assert!(!memory_type
            .allows_compiler_memory_type_definition_at(CompilerMemoryType::RawPointer, proven));
    }

    #[test]
    fn source_map_keeps_capabilities_per_file() {
        let mut source_map = SourceMap::new();
        let plain = source_map.add("plain.nepl", String::from(""));
        let plain_span = Span::new(plain, 0, 8);
        let raw_span = Span::new(FileId(1), 0, 8);
        let _raw = source_map.add_with_capabilities(
            "core/mem.nepl",
            String::from(""),
            use_site_capabilities(SourceCapabilityUseSite::RawMemoryStructuralBoundary {
                span: SourceCapabilitySpan::from_span(raw_span),
            }),
        );

        assert!(!source_map.raw_memory_structural_boundary_allowed_at(plain_span));
        assert!(!source_map.raw_address_view_boundary_allowed_at(plain_span));
        assert!(source_map.raw_memory_structural_boundary_allowed_at(raw_span));
        assert!(!source_map.raw_address_view_boundary_allowed_at(raw_span));
        assert!(!source_map.raw_memory_operation_boundary_allowed_at(raw_span, RawMemoryOp::Load));
        assert!(!source_map.owner_aggregate_constructor_boundary_allowed_at(raw_span, "Vec"));
        assert!(!source_map.owner_aggregate_field_boundary_allowed_at(raw_span));

        source_map.set_capabilities(
            plain,
            use_site_capabilities(SourceCapabilityUseSite::CompilerMemoryTypeDefinition {
                memory_type: CompilerMemoryType::RawPointer,
                span: SourceCapabilitySpan::from_span(plain_span),
            }),
        );
        assert!(source_map.compiler_memory_type_definition_allowed_at(
            plain_span,
            CompilerMemoryType::RawPointer
        ));
        assert!(!source_map
            .compiler_memory_type_definition_allowed_at(raw_span, CompilerMemoryType::RawPointer));
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
    }
}
