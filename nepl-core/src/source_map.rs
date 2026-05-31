//! Source text table and diagnostic path labels.
//!
//! The compiler core only needs stable source labels for diagnostics. Host
//! filesystem paths are converted to strings by the loader layer so this module
//! can stay `no_std` + `alloc`.

use alloc::collections::BTreeSet;
use alloc::string::String;
use alloc::vec::Vec;
use core::fmt;

use crate::effects::{PrivateCacheOp, PrivateEffectRegion, RawBodyMemoryOp, RawMemoryOp};
pub use crate::resource_primitives::{
    CollectionSlotBorrowPrimitive, CollectionSlotLifecyclePrimitive,
};
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
    RawAddressAliasBoundary {
        span: SourceCapabilitySpan,
    },
    OwnerTokenConstructBoundary {
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
        field: CompilerMemoryField,
        span: SourceCapabilitySpan,
    },
    CompilerMemoryTypeDefinition {
        memory_type: CompilerMemoryType,
        span: SourceCapabilitySpan,
    },
    CollectionSlotLifecycleBoundary {
        primitive: CollectionSlotLifecyclePrimitive,
        span: SourceCapabilitySpan,
    },
    CollectionSlotBorrowBoundary {
        primitive: CollectionSlotBorrowPrimitive,
        span: SourceCapabilitySpan,
    },
    PrivateCacheBoundary {
        operation: PrivateCacheOp,
        region: PrivateEffectRegion,
        span: SourceCapabilitySpan,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CompilerMemoryType {
    RawPointer,
    OwnerToken,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CompilerMemoryField {
    Raw,
    Size,
}

impl CompilerMemoryField {
    pub(crate) fn from_name(name: &str) -> Option<Self> {
        match name {
            "raw" => Some(Self::Raw),
            "size" => Some(Self::Size),
            _ => None,
        }
    }
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

    pub(crate) fn retain_use_sites(
        &mut self,
        mut keep: impl FnMut(&SourceCapabilityUseSite) -> bool,
    ) {
        self.use_sites.retain(|site| keep(site));
    }

    /// Resource summary cache key に含める source capability policy hash を作る。
    ///
    /// capability proof が存在する file では、proof span の byte range だけを hash すると
    /// 別 source へ誤って再利用できてしまう。この hash は canonical path と source hash を
    /// 含め、同じ file content 上の同じ use-site proof set だけを同一 policy とみなす。
    ///
    /// capability proof が空の file では、Resource IR body hash と typed signature が
    /// function semantics を固定するため、source text 全体を policy に混ぜない。通常の
    /// source edit が raw memory / collection slot privilege を一切持たない関数まで
    /// invalidation しないよう、path と空 proof set だけを policy surface にする。
    #[allow(dead_code)]
    fn stable_policy_hash(&self, canonical_path: &str, source_hash: u64) -> u64 {
        let mut hash = 0xcbf29ce484222325;
        source_capability_policy_hash_str(&mut hash, "neplg2-source-capability-policy-v1");
        source_capability_policy_hash_str(&mut hash, canonical_path);
        if !self.use_sites.is_empty() {
            source_capability_policy_hash_u64(&mut hash, source_hash);
        }
        for use_site in &self.use_sites {
            source_capability_use_site_hash(&mut hash, use_site);
        }
        hash
    }

    /// Resource summary cache key に含める、関数範囲へ閉じた source capability policy hash を作る。
    ///
    /// file 全体の source hash を使うと、同じ file の sibling function edit でも capability
    /// proof を持つ関数の summary cache が miss する。この hash は scope 内にある
    /// capability proof だけを取り込み、use-site span は scope 先頭からの相対位置として
    /// hash する。関数より前のコメントや別関数の変更で absolute byte offset がずれても、
    /// 関数本文と proof surface が同じなら同じ policy とみなすためである。
    ///
    /// scope 内に capability proof がない場合は、file-level policy と同じく source text を
    /// 混ぜない。source semantics は Resource IR body hash と typed signature/type boundary
    /// が固定し、source capability policy は privilege proof surface のみに責務を限定する。
    #[allow(dead_code)]
    pub(crate) fn stable_scoped_policy_hash(
        &self,
        canonical_path: &str,
        source: &str,
        scope_start: u32,
        scope_end: u32,
    ) -> Option<u64> {
        if scope_start > scope_end || scope_end as usize > source.len() {
            return None;
        }
        let scoped_use_sites = self.scoped_use_sites(scope_start, scope_end)?;
        let mut hash = 0xcbf29ce484222325;
        source_capability_policy_hash_str(&mut hash, "neplg2-source-capability-scoped-policy-v1");
        source_capability_policy_hash_str(&mut hash, canonical_path);
        if !scoped_use_sites.is_empty() {
            source_capability_policy_hash_u64(&mut hash, u64::from(scope_end - scope_start));
            source_capability_policy_hash_bytes(
                &mut hash,
                &source.as_bytes()[scope_start as usize..scope_end as usize],
            );
        }
        for use_site in scoped_use_sites {
            source_capability_scoped_use_site_hash(&mut hash, use_site, scope_start);
        }
        Some(hash)
    }

    fn scoped_use_sites(
        &self,
        scope_start: u32,
        scope_end: u32,
    ) -> Option<Vec<&SourceCapabilityUseSite>> {
        let mut scoped = Vec::new();
        for use_site in &self.use_sites {
            let span = source_capability_use_site_span(use_site);
            if span.start >= scope_start && span.end <= scope_end {
                scoped.push(use_site);
            } else if span.start < scope_end && span.end > scope_start {
                return None;
            }
        }
        Some(scoped)
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

    pub fn allows_raw_address_alias_boundary_at(&self, span: Span) -> bool {
        self.allows_use_site(SourceCapabilityUseSite::RawAddressAliasBoundary {
            span: SourceCapabilitySpan::from_span(span),
        })
    }

    pub fn allows_owner_token_construct_boundary_at(&self, span: Span) -> bool {
        self.allows_use_site(SourceCapabilityUseSite::OwnerTokenConstructBoundary {
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

    pub fn allows_compiler_memory_field_boundary_at(
        &self,
        field: CompilerMemoryField,
        span: Span,
    ) -> bool {
        self.allows_use_site(SourceCapabilityUseSite::CompilerMemoryFieldBoundary {
            field,
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

    pub fn allows_collection_slot_lifecycle_boundary_at(
        &self,
        primitive: CollectionSlotLifecyclePrimitive,
        span: Span,
    ) -> bool {
        self.allows_use_site(SourceCapabilityUseSite::CollectionSlotLifecycleBoundary {
            primitive,
            span: SourceCapabilitySpan::from_span(span),
        })
    }

    pub fn allows_collection_slot_borrow_boundary_at(
        &self,
        primitive: CollectionSlotBorrowPrimitive,
        span: Span,
    ) -> bool {
        self.allows_use_site(SourceCapabilityUseSite::CollectionSlotBorrowBoundary {
            primitive,
            span: SourceCapabilitySpan::from_span(span),
        })
    }

    pub fn allows_private_cache_boundary_at(&self, operation: PrivateCacheOp, span: Span) -> bool {
        self.allows_private_cache_boundary_in_region_at(
            operation,
            PrivateEffectRegion::UnsealedIntrinsic,
            span,
        )
    }

    pub fn allows_private_cache_boundary_in_region_at(
        &self,
        operation: PrivateCacheOp,
        region: PrivateEffectRegion,
        span: Span,
    ) -> bool {
        self.allows_use_site(SourceCapabilityUseSite::PrivateCacheBoundary {
            operation,
            region,
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

    pub(crate) fn capabilities(&self, id: FileId) -> SourceCapabilities {
        self.files
            .get(id.0 as usize)
            .map(|file| file.capabilities.clone())
            .unwrap_or_default()
    }

    /// Resource summary value cache key に含める source capability policy hash を返す。
    ///
    /// source text は `SourceMap` が保持しているため、caller に source hash を渡させない。
    /// これにより別 source の hash や sentinel 値を誤って渡す経路を作らない。
    /// capability proof がある file は path、source content、proof set が同じ場合だけ
    /// 同じ policy とみなし、proof が空の file は path と空 proof set だけを policy とする。
    pub(crate) fn source_capability_policy_hash_for_file(&self, id: FileId) -> Option<u64> {
        self.files.get(id.0 as usize).map(|file| {
            let source_hash = source_capability_source_hash(file.src.as_bytes());
            file.capabilities
                .stable_policy_hash(file.path.as_str(), source_hash)
        })
    }

    /// Resource summary value cache key に含める scoped source capability policy hash を返す。
    ///
    /// `scope` は Resource IR の関数本文や block/op span から作られた file-local 範囲である。
    /// source capability proof が scope 内にある場合だけ、その source slice と相対 proof span
    /// を key に含める。scope と capability proof が部分的にしか重ならない場合は、関数境界を
    /// 安全に確定できないため `None` に倒し、summary cache は store/replay しない。
    #[cfg(test)]
    pub(crate) fn source_capability_policy_hash_for_span_scope(
        &self,
        id: FileId,
        scope_start: u32,
        scope_end: u32,
    ) -> Option<u64> {
        self.files.get(id.0 as usize).and_then(|file| {
            file.capabilities.stable_scoped_policy_hash(
                file.path.as_str(),
                file.src.as_str(),
                scope_start,
                scope_end,
            )
        })
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

    pub fn raw_address_alias_boundary_allowed_at(&self, span: Span) -> bool {
        self.capabilities(span.file_id)
            .allows_raw_address_alias_boundary_at(span)
    }

    pub fn owner_token_construct_boundary_allowed_at(&self, span: Span) -> bool {
        self.capabilities(span.file_id)
            .allows_owner_token_construct_boundary_at(span)
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

    pub fn compiler_memory_field_boundary_allowed_at(
        &self,
        field: CompilerMemoryField,
        span: Span,
    ) -> bool {
        self.capabilities(span.file_id)
            .allows_compiler_memory_field_boundary_at(field, span)
    }

    pub fn compiler_memory_type_definition_allowed_at(
        &self,
        span: Span,
        memory_type: CompilerMemoryType,
    ) -> bool {
        self.capabilities(span.file_id)
            .allows_compiler_memory_type_definition_at(memory_type, span)
    }

    pub fn collection_slot_lifecycle_boundary_allowed_at(
        &self,
        span: Span,
        primitive: CollectionSlotLifecyclePrimitive,
    ) -> bool {
        self.capabilities(span.file_id)
            .allows_collection_slot_lifecycle_boundary_at(primitive, span)
    }

    pub fn collection_slot_borrow_boundary_allowed_at(
        &self,
        span: Span,
        primitive: CollectionSlotBorrowPrimitive,
    ) -> bool {
        self.capabilities(span.file_id)
            .allows_collection_slot_borrow_boundary_at(primitive, span)
    }

    pub fn private_cache_boundary_allowed_at(
        &self,
        span: Span,
        operation: PrivateCacheOp,
        region: PrivateEffectRegion,
    ) -> bool {
        self.capabilities(span.file_id)
            .allows_private_cache_boundary_in_region_at(operation, region, span)
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

#[allow(dead_code)]
fn source_capability_use_site_hash(hash: &mut u64, use_site: &SourceCapabilityUseSite) {
    match use_site {
        SourceCapabilityUseSite::RawMemoryStructuralBoundary { span } => {
            source_capability_policy_hash_str(hash, "raw-memory-structural");
            source_capability_span_hash(hash, *span);
        }
        SourceCapabilityUseSite::RawAddressViewBoundary { span } => {
            source_capability_policy_hash_str(hash, "raw-address-view");
            source_capability_span_hash(hash, *span);
        }
        SourceCapabilityUseSite::RawAddressAliasBoundary { span } => {
            source_capability_policy_hash_str(hash, "raw-address-alias");
            source_capability_span_hash(hash, *span);
        }
        SourceCapabilityUseSite::OwnerTokenConstructBoundary { span } => {
            source_capability_policy_hash_str(hash, "owner-token-construct");
            source_capability_span_hash(hash, *span);
        }
        SourceCapabilityUseSite::RawMemoryOperationBoundary { operation, span } => {
            source_capability_policy_hash_str(hash, "raw-memory-operation");
            source_capability_policy_hash_str(hash, operation.as_str());
            source_capability_span_hash(hash, *span);
        }
        SourceCapabilityUseSite::RawBodyMemoryOperationBoundary { operation, span } => {
            source_capability_policy_hash_str(hash, "raw-body-memory-operation");
            source_capability_raw_body_memory_op_hash(hash, *operation);
            source_capability_span_hash(hash, *span);
        }
        SourceCapabilityUseSite::OwnerAggregateConstructorBoundary { name, span } => {
            source_capability_policy_hash_str(hash, "owner-aggregate-constructor");
            source_capability_policy_hash_str(hash, name);
            source_capability_span_hash(hash, *span);
        }
        SourceCapabilityUseSite::OwnerAggregateFieldBoundary { span } => {
            source_capability_policy_hash_str(hash, "owner-aggregate-field");
            source_capability_span_hash(hash, *span);
        }
        SourceCapabilityUseSite::CompilerMemoryFieldBoundary { field, span } => {
            source_capability_policy_hash_str(hash, "compiler-memory-field");
            source_capability_policy_hash_str(hash, compiler_memory_field_tag(*field));
            source_capability_span_hash(hash, *span);
        }
        SourceCapabilityUseSite::CompilerMemoryTypeDefinition { memory_type, span } => {
            source_capability_policy_hash_str(hash, "compiler-memory-type-definition");
            source_capability_policy_hash_str(hash, compiler_memory_type_tag(*memory_type));
            source_capability_span_hash(hash, *span);
        }
        SourceCapabilityUseSite::CollectionSlotLifecycleBoundary { primitive, span } => {
            source_capability_policy_hash_str(hash, "collection-slot-lifecycle");
            source_capability_policy_hash_str(
                hash,
                collection_slot_lifecycle_primitive_tag(*primitive),
            );
            source_capability_span_hash(hash, *span);
        }
        SourceCapabilityUseSite::CollectionSlotBorrowBoundary { primitive, span } => {
            source_capability_policy_hash_str(hash, "collection-slot-borrow");
            source_capability_policy_hash_str(
                hash,
                collection_slot_borrow_primitive_tag(*primitive),
            );
            source_capability_span_hash(hash, *span);
        }
        SourceCapabilityUseSite::PrivateCacheBoundary {
            operation,
            region,
            span,
        } => {
            source_capability_policy_hash_str(hash, "private-cache");
            source_capability_policy_hash_str(hash, operation.as_str());
            source_capability_policy_hash_str(hash, region.as_str());
            source_capability_span_hash(hash, *span);
        }
    }
}

#[allow(dead_code)]
fn source_capability_scoped_use_site_hash(
    hash: &mut u64,
    use_site: &SourceCapabilityUseSite,
    scope_start: u32,
) {
    match use_site {
        SourceCapabilityUseSite::RawMemoryStructuralBoundary { span } => {
            source_capability_policy_hash_str(hash, "raw-memory-structural");
            source_capability_relative_span_hash(hash, *span, scope_start);
        }
        SourceCapabilityUseSite::RawAddressViewBoundary { span } => {
            source_capability_policy_hash_str(hash, "raw-address-view");
            source_capability_relative_span_hash(hash, *span, scope_start);
        }
        SourceCapabilityUseSite::RawAddressAliasBoundary { span } => {
            source_capability_policy_hash_str(hash, "raw-address-alias");
            source_capability_relative_span_hash(hash, *span, scope_start);
        }
        SourceCapabilityUseSite::OwnerTokenConstructBoundary { span } => {
            source_capability_policy_hash_str(hash, "owner-token-construct");
            source_capability_relative_span_hash(hash, *span, scope_start);
        }
        SourceCapabilityUseSite::RawMemoryOperationBoundary { operation, span } => {
            source_capability_policy_hash_str(hash, "raw-memory-operation");
            source_capability_policy_hash_str(hash, operation.as_str());
            source_capability_relative_span_hash(hash, *span, scope_start);
        }
        SourceCapabilityUseSite::RawBodyMemoryOperationBoundary { operation, span } => {
            source_capability_policy_hash_str(hash, "raw-body-memory-operation");
            source_capability_raw_body_memory_op_hash(hash, *operation);
            source_capability_relative_span_hash(hash, *span, scope_start);
        }
        SourceCapabilityUseSite::OwnerAggregateConstructorBoundary { name, span } => {
            source_capability_policy_hash_str(hash, "owner-aggregate-constructor");
            source_capability_policy_hash_str(hash, name);
            source_capability_relative_span_hash(hash, *span, scope_start);
        }
        SourceCapabilityUseSite::OwnerAggregateFieldBoundary { span } => {
            source_capability_policy_hash_str(hash, "owner-aggregate-field");
            source_capability_relative_span_hash(hash, *span, scope_start);
        }
        SourceCapabilityUseSite::CompilerMemoryFieldBoundary { field, span } => {
            source_capability_policy_hash_str(hash, "compiler-memory-field");
            source_capability_policy_hash_str(hash, compiler_memory_field_tag(*field));
            source_capability_relative_span_hash(hash, *span, scope_start);
        }
        SourceCapabilityUseSite::CompilerMemoryTypeDefinition { memory_type, span } => {
            source_capability_policy_hash_str(hash, "compiler-memory-type-definition");
            source_capability_policy_hash_str(hash, compiler_memory_type_tag(*memory_type));
            source_capability_relative_span_hash(hash, *span, scope_start);
        }
        SourceCapabilityUseSite::CollectionSlotLifecycleBoundary { primitive, span } => {
            source_capability_policy_hash_str(hash, "collection-slot-lifecycle");
            source_capability_policy_hash_str(
                hash,
                collection_slot_lifecycle_primitive_tag(*primitive),
            );
            source_capability_relative_span_hash(hash, *span, scope_start);
        }
        SourceCapabilityUseSite::CollectionSlotBorrowBoundary { primitive, span } => {
            source_capability_policy_hash_str(hash, "collection-slot-borrow");
            source_capability_policy_hash_str(
                hash,
                collection_slot_borrow_primitive_tag(*primitive),
            );
            source_capability_relative_span_hash(hash, *span, scope_start);
        }
        SourceCapabilityUseSite::PrivateCacheBoundary {
            operation,
            region,
            span,
        } => {
            source_capability_policy_hash_str(hash, "private-cache");
            source_capability_policy_hash_str(hash, operation.as_str());
            source_capability_policy_hash_str(hash, region.as_str());
            source_capability_relative_span_hash(hash, *span, scope_start);
        }
    }
}

#[allow(dead_code)]
fn source_capability_use_site_span(use_site: &SourceCapabilityUseSite) -> SourceCapabilitySpan {
    match use_site {
        SourceCapabilityUseSite::RawMemoryStructuralBoundary { span }
        | SourceCapabilityUseSite::RawAddressViewBoundary { span }
        | SourceCapabilityUseSite::RawAddressAliasBoundary { span }
        | SourceCapabilityUseSite::OwnerTokenConstructBoundary { span }
        | SourceCapabilityUseSite::RawMemoryOperationBoundary { span, .. }
        | SourceCapabilityUseSite::RawBodyMemoryOperationBoundary { span, .. }
        | SourceCapabilityUseSite::OwnerAggregateConstructorBoundary { span, .. }
        | SourceCapabilityUseSite::OwnerAggregateFieldBoundary { span }
        | SourceCapabilityUseSite::CompilerMemoryFieldBoundary { span, .. }
        | SourceCapabilityUseSite::CompilerMemoryTypeDefinition { span, .. }
        | SourceCapabilityUseSite::CollectionSlotLifecycleBoundary { span, .. }
        | SourceCapabilityUseSite::CollectionSlotBorrowBoundary { span, .. }
        | SourceCapabilityUseSite::PrivateCacheBoundary { span, .. } => *span,
    }
}

#[allow(dead_code)]
fn source_capability_span_hash(hash: &mut u64, span: SourceCapabilitySpan) {
    source_capability_policy_hash_u32(hash, span.start);
    source_capability_policy_hash_u32(hash, span.end);
}

#[allow(dead_code)]
fn source_capability_relative_span_hash(
    hash: &mut u64,
    span: SourceCapabilitySpan,
    scope_start: u32,
) {
    source_capability_policy_hash_u32(hash, span.start.saturating_sub(scope_start));
    source_capability_policy_hash_u32(hash, span.end.saturating_sub(scope_start));
}

#[allow(dead_code)]
fn source_capability_raw_body_memory_op_hash(hash: &mut u64, operation: RawBodyMemoryOp) {
    match operation {
        RawBodyMemoryOp::Wasm(operation) => {
            source_capability_policy_hash_str(hash, "wasm");
            source_capability_policy_hash_str(hash, operation.as_str());
        }
        RawBodyMemoryOp::Llvm(operation) => {
            source_capability_policy_hash_str(hash, "llvm");
            source_capability_policy_hash_str(hash, operation.as_str());
        }
    }
}

#[allow(dead_code)]
fn compiler_memory_field_tag(field: CompilerMemoryField) -> &'static str {
    match field {
        CompilerMemoryField::Raw => "raw",
        CompilerMemoryField::Size => "size",
    }
}

#[allow(dead_code)]
fn compiler_memory_type_tag(memory_type: CompilerMemoryType) -> &'static str {
    match memory_type {
        CompilerMemoryType::RawPointer => "raw-pointer",
        CompilerMemoryType::OwnerToken => "owner-token",
    }
}

#[allow(dead_code)]
fn collection_slot_lifecycle_primitive_tag(
    primitive: CollectionSlotLifecyclePrimitive,
) -> &'static str {
    match primitive {
        CollectionSlotLifecyclePrimitive::InitializeEmpty => "initialize-empty",
        CollectionSlotLifecyclePrimitive::BorrowRead => "borrow-read",
        CollectionSlotLifecyclePrimitive::MoveOut => "move-out",
        CollectionSlotLifecyclePrimitive::ReplaceReturnOld => "replace-return-old",
        CollectionSlotLifecyclePrimitive::ReplaceDropOld => "replace-drop-old",
        CollectionSlotLifecyclePrimitive::DropInitialized => "drop-initialized",
        CollectionSlotLifecyclePrimitive::DropTraversal => "drop-traversal",
        CollectionSlotLifecyclePrimitive::TransformRange => "transform-range",
        CollectionSlotLifecyclePrimitive::StorageDealloc => "storage-dealloc",
        CollectionSlotLifecyclePrimitive::StorageRelocate => "storage-relocate",
    }
}

#[allow(dead_code)]
fn collection_slot_borrow_primitive_tag(primitive: CollectionSlotBorrowPrimitive) -> &'static str {
    match primitive {
        CollectionSlotBorrowPrimitive::BorrowRef => "borrow-ref",
    }
}

#[allow(dead_code)]
fn source_capability_policy_hash_str(hash: &mut u64, value: &str) {
    source_capability_policy_hash_bytes(hash, value.as_bytes());
    source_capability_policy_hash_bytes(hash, &[0]);
}

#[allow(dead_code)]
fn source_capability_policy_hash_u64(hash: &mut u64, value: u64) {
    source_capability_policy_hash_bytes(hash, &value.to_le_bytes());
}

#[allow(dead_code)]
fn source_capability_policy_hash_u32(hash: &mut u64, value: u32) {
    source_capability_policy_hash_bytes(hash, &value.to_le_bytes());
}

#[allow(dead_code)]
fn source_capability_policy_hash_bytes(hash: &mut u64, bytes: &[u8]) {
    for byte in bytes {
        *hash ^= u64::from(*byte);
        *hash = hash.wrapping_mul(0x100000001b3);
    }
}

#[allow(dead_code)]
/// SourceMap 内の source text を policy hash 入力へ変換する deterministic hash。
///
/// loader cache の source hash と同じ FNV-1a 64bit 形を使うが、この関数は source
/// capability policy の内部入力に閉じる。caller へ source hash の指定権を渡さないことが
/// stale capability proof を避けるための重要な境界である。
fn source_capability_source_hash(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf29ce484222325;
    source_capability_policy_hash_bytes(&mut hash, bytes);
    hash
}

#[cfg(test)]
mod tests {
    use alloc::string::String;

    use crate::effects::{
        PrivateCacheOp, PrivateEffectRegion, RawBodyMemoryOp, RawMemoryOp, WasmRawBodyMemoryOp,
    };
    use crate::span::{FileId, Span};

    use super::{
        CompilerMemoryField, CompilerMemoryType, SourceCapabilities, SourceCapabilitySpan,
        SourceCapabilityUseSite, SourceMap,
    };

    fn use_site_capabilities(use_site: SourceCapabilityUseSite) -> SourceCapabilities {
        let mut capabilities = SourceCapabilities::none();
        capabilities.insert_use_site(use_site);
        capabilities
    }

    fn raw_load_use_site(span: Span) -> SourceCapabilityUseSite {
        SourceCapabilityUseSite::RawMemoryOperationBoundary {
            operation: RawMemoryOp::Load,
            span: SourceCapabilitySpan::from_span(span),
        }
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
        assert!(!none.allows_raw_address_alias_boundary_at(proven));
        assert!(!none.allows_owner_token_construct_boundary_at(proven));
        assert!(!none.allows_raw_memory_operation_boundary_at(RawMemoryOp::Load, proven));
        assert!(!none.allows_private_cache_boundary_at(PrivateCacheOp::Lookup, proven));
        assert!(!none
            .allows_compiler_memory_type_definition_at(CompilerMemoryType::RawPointer, proven,));

        let raw_boundary =
            use_site_capabilities(SourceCapabilityUseSite::RawMemoryStructuralBoundary {
                span: SourceCapabilitySpan::from_span(proven),
            });
        assert!(raw_boundary.allows_raw_memory_structural_boundary_at(proven));
        assert!(!raw_boundary.allows_raw_memory_structural_boundary_at(other));
        assert!(!raw_boundary.allows_raw_address_view_boundary_at(proven));
        assert!(!raw_boundary.allows_owner_token_construct_boundary_at(proven));
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
        assert!(!raw_load.allows_raw_address_alias_boundary_at(proven));
        assert!(!raw_load.allows_owner_token_construct_boundary_at(proven));

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
        assert!(!owner_constructor.allows_owner_token_construct_boundary_at(proven));

        let owner_field =
            use_site_capabilities(SourceCapabilityUseSite::OwnerAggregateFieldBoundary {
                span: SourceCapabilitySpan::from_span(proven),
            });
        assert!(owner_field.allows_owner_aggregate_field_boundary_at(proven));
        assert!(!owner_field.allows_owner_aggregate_field_boundary_at(other));
        assert!(!owner_field.allows_owner_aggregate_constructor_boundary_at("Vec", proven));
        assert!(
            !owner_field.allows_compiler_memory_field_boundary_at(CompilerMemoryField::Raw, proven)
        );
        assert!(!owner_field.allows_raw_memory_structural_boundary_at(proven));
        assert!(!owner_field.allows_raw_address_view_boundary_at(proven));
        assert!(!owner_field.allows_owner_token_construct_boundary_at(proven));

        let compiler_field =
            use_site_capabilities(SourceCapabilityUseSite::CompilerMemoryFieldBoundary {
                field: CompilerMemoryField::Raw,
                span: SourceCapabilitySpan::from_span(proven),
            });
        assert!(compiler_field
            .allows_compiler_memory_field_boundary_at(CompilerMemoryField::Raw, proven));
        assert!(!compiler_field
            .allows_compiler_memory_field_boundary_at(CompilerMemoryField::Raw, other));
        assert!(!compiler_field
            .allows_compiler_memory_field_boundary_at(CompilerMemoryField::Size, proven));
        assert!(!compiler_field.allows_owner_aggregate_field_boundary_at(proven));
        assert!(!compiler_field.allows_owner_token_construct_boundary_at(proven));

        let address_view = use_site_capabilities(SourceCapabilityUseSite::RawAddressViewBoundary {
            span: SourceCapabilitySpan::from_span(proven),
        });
        assert!(address_view.allows_raw_address_view_boundary_at(proven));
        assert!(!address_view.allows_raw_address_view_boundary_at(other));
        assert!(!address_view.allows_raw_memory_structural_boundary_at(proven));
        assert!(!address_view.allows_raw_address_alias_boundary_at(proven));
        assert!(!address_view.allows_owner_token_construct_boundary_at(proven));

        let address_alias =
            use_site_capabilities(SourceCapabilityUseSite::RawAddressAliasBoundary {
                span: SourceCapabilitySpan::from_span(proven),
            });
        assert!(address_alias.allows_raw_address_alias_boundary_at(proven));
        assert!(!address_alias.allows_raw_address_alias_boundary_at(other));
        assert!(!address_alias.allows_raw_address_view_boundary_at(proven));
        assert!(!address_alias.allows_raw_memory_structural_boundary_at(proven));
        assert!(!address_alias.allows_owner_token_construct_boundary_at(proven));

        let owner_token_construct =
            use_site_capabilities(SourceCapabilityUseSite::OwnerTokenConstructBoundary {
                span: SourceCapabilitySpan::from_span(proven),
            });
        assert!(owner_token_construct.allows_owner_token_construct_boundary_at(proven));
        assert!(!owner_token_construct.allows_owner_token_construct_boundary_at(other));
        assert!(!owner_token_construct.allows_raw_address_alias_boundary_at(proven));
        assert!(!owner_token_construct.allows_raw_memory_structural_boundary_at(proven));

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

        let private_cache = use_site_capabilities(SourceCapabilityUseSite::PrivateCacheBoundary {
            operation: PrivateCacheOp::Lookup,
            region: PrivateEffectRegion::UnsealedIntrinsic,
            span: SourceCapabilitySpan::from_span(proven),
        });
        assert!(private_cache.allows_private_cache_boundary_at(PrivateCacheOp::Lookup, proven));
        assert!(!private_cache.allows_private_cache_boundary_at(PrivateCacheOp::Lookup, other));
        assert!(!private_cache.allows_private_cache_boundary_at(PrivateCacheOp::Insert, proven));
        assert!(!private_cache.allows_raw_memory_operation_boundary_at(RawMemoryOp::Load, proven));
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
        assert!(!source_map.raw_address_alias_boundary_allowed_at(plain_span));
        assert!(!source_map.owner_token_construct_boundary_allowed_at(plain_span));
        assert!(source_map.raw_memory_structural_boundary_allowed_at(raw_span));
        assert!(!source_map.raw_address_view_boundary_allowed_at(raw_span));
        assert!(!source_map.raw_address_alias_boundary_allowed_at(raw_span));
        assert!(!source_map.owner_token_construct_boundary_allowed_at(raw_span));
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

    #[test]
    fn source_capability_policy_hash_is_order_independent() {
        let file = FileId(0);
        let first_span = Span::new(file, 8, 16);
        let second_span = Span::new(file, 24, 32);
        let mut first = SourceCapabilities::none();
        first.insert_use_site(raw_load_use_site(first_span));
        first.insert_use_site(SourceCapabilityUseSite::RawBodyMemoryOperationBoundary {
            operation: RawBodyMemoryOp::Wasm(WasmRawBodyMemoryOp::MemoryGrow),
            span: SourceCapabilitySpan::from_span(second_span),
        });
        let mut second = SourceCapabilities::none();
        second.insert_use_site(SourceCapabilityUseSite::RawBodyMemoryOperationBoundary {
            operation: RawBodyMemoryOp::Wasm(WasmRawBodyMemoryOp::MemoryGrow),
            span: SourceCapabilitySpan::from_span(second_span),
        });
        second.insert_use_site(raw_load_use_site(first_span));

        assert_eq!(
            first.stable_policy_hash("core/mem.nepl", 7),
            second.stable_policy_hash("core/mem.nepl", 7)
        );
    }

    #[test]
    fn source_capability_policy_hash_tracks_source_and_use_site_inputs() {
        let file = FileId(0);
        let proven = Span::new(file, 8, 16);
        let shifted = Span::new(file, 9, 17);
        let base = use_site_capabilities(raw_load_use_site(proven));
        let shifted_span = use_site_capabilities(raw_load_use_site(shifted));
        let other_operation =
            use_site_capabilities(SourceCapabilityUseSite::RawMemoryOperationBoundary {
                operation: RawMemoryOp::Store,
                span: SourceCapabilitySpan::from_span(proven),
            });
        let private_cache_operation =
            use_site_capabilities(SourceCapabilityUseSite::PrivateCacheBoundary {
                operation: PrivateCacheOp::Lookup,
                region: PrivateEffectRegion::UnsealedIntrinsic,
                span: SourceCapabilitySpan::from_span(proven),
            });

        assert_ne!(
            base.stable_policy_hash("core/mem.nepl", 7),
            base.stable_policy_hash("core/other.nepl", 7)
        );
        assert_ne!(
            base.stable_policy_hash("core/mem.nepl", 7),
            base.stable_policy_hash("core/mem.nepl", 8)
        );
        assert_ne!(
            base.stable_policy_hash("core/mem.nepl", 7),
            shifted_span.stable_policy_hash("core/mem.nepl", 7)
        );
        assert_ne!(
            base.stable_policy_hash("core/mem.nepl", 7),
            other_operation.stable_policy_hash("core/mem.nepl", 7)
        );
        assert_ne!(
            base.stable_policy_hash("core/mem.nepl", 7),
            private_cache_operation.stable_policy_hash("core/mem.nepl", 7)
        );
    }

    #[test]
    fn empty_source_capability_policy_hash_ignores_source_text() {
        let none = SourceCapabilities::none();

        assert_eq!(
            none.stable_policy_hash("examples/rpn.nepl", 7),
            none.stable_policy_hash("examples/rpn.nepl", 8)
        );
        assert_ne!(
            none.stable_policy_hash("examples/rpn.nepl", 7),
            none.stable_policy_hash("examples/other.nepl", 7)
        );
    }

    #[test]
    fn source_capability_source_hash_matches_loader_source_hash_contract() {
        assert_eq!(
            super::source_capability_source_hash(b"fn load %i32 1"),
            0x2aa54b28b4377481
        );
    }

    #[test]
    fn source_map_source_capability_policy_hash_uses_file_path_and_source_text() {
        let mut source_map = SourceMap::new();
        let capabilities = use_site_capabilities(raw_load_use_site(Span::new(FileId(0), 8, 16)));
        let file = source_map.add_with_capabilities(
            "core/mem.nepl",
            String::from("fn load %i32 1"),
            capabilities.clone(),
        );

        assert_eq!(
            source_map.source_capability_policy_hash_for_file(file),
            Some(capabilities.stable_policy_hash(
                "core/mem.nepl",
                super::source_capability_source_hash(b"fn load %i32 1")
            ))
        );

        let same_path_different_source = source_map.add_with_capabilities(
            "core/mem.nepl",
            String::from("fn load %i32 2"),
            capabilities.clone(),
        );
        let different_path_same_source = source_map.add_with_capabilities(
            "core/other.nepl",
            String::from("fn load %i32 1"),
            capabilities,
        );

        assert_ne!(
            source_map.source_capability_policy_hash_for_file(file),
            source_map.source_capability_policy_hash_for_file(same_path_different_source)
        );
        assert_ne!(
            source_map.source_capability_policy_hash_for_file(file),
            source_map.source_capability_policy_hash_for_file(different_path_same_source)
        );
    }

    #[test]
    fn source_map_empty_source_capability_policy_hash_ignores_file_source_text() {
        let mut source_map = SourceMap::new();
        let first = source_map.add_with_capabilities(
            "examples/rpn.nepl",
            String::from("fn main %impure fn unit unit \\unit: 1"),
            SourceCapabilities::none(),
        );
        let same_path_different_source = source_map.add_with_capabilities(
            "examples/rpn.nepl",
            String::from("fn main %impure fn unit unit \\unit: 2"),
            SourceCapabilities::none(),
        );
        let different_path_same_source = source_map.add_with_capabilities(
            "examples/other.nepl",
            String::from("fn main %impure fn unit unit \\unit: 1"),
            SourceCapabilities::none(),
        );

        assert_eq!(
            source_map.source_capability_policy_hash_for_file(first),
            source_map.source_capability_policy_hash_for_file(same_path_different_source)
        );
        assert_ne!(
            source_map.source_capability_policy_hash_for_file(first),
            source_map.source_capability_policy_hash_for_file(different_path_same_source)
        );
    }

    #[test]
    fn scoped_source_capability_policy_hash_uses_relative_function_surface() {
        let source = String::from("before\nfn a:\n    load_u8 raw\n\nafter\n");
        let scope_start = source.find("fn a").unwrap() as u32;
        let scope_end = source.find("\n\nafter").unwrap() as u32;
        let load_start = source.find("load_u8").unwrap() as u32;
        let load_end = load_start + "load_u8".len() as u32;
        let mut source_map = SourceMap::new();
        let file = FileId(0);
        let id = source_map.add_with_capabilities(
            "stdlib/core/mem/raw.nepl",
            source,
            use_site_capabilities(raw_load_use_site(Span::new(file, load_start, load_end))),
        );

        let shifted_source =
            String::from("// shifted prefix\nbefore\nfn a:\n    load_u8 raw\n\nafter\n");
        let shifted_scope_start = shifted_source.find("fn a").unwrap() as u32;
        let shifted_scope_end = shifted_source.find("\n\nafter").unwrap() as u32;
        let shifted_load_start = shifted_source.find("load_u8").unwrap() as u32;
        let shifted_load_end = shifted_load_start + "load_u8".len() as u32;
        let mut shifted_map = SourceMap::new();
        let shifted_file = FileId(0);
        let shifted_id = shifted_map.add_with_capabilities(
            "stdlib/core/mem/raw.nepl",
            shifted_source,
            use_site_capabilities(raw_load_use_site(Span::new(
                shifted_file,
                shifted_load_start,
                shifted_load_end,
            ))),
        );

        assert_eq!(
            source_map.source_capability_policy_hash_for_span_scope(id, scope_start, scope_end),
            shifted_map.source_capability_policy_hash_for_span_scope(
                shifted_id,
                shifted_scope_start,
                shifted_scope_end
            )
        );
    }

    #[test]
    fn scoped_source_capability_policy_hash_ignores_sibling_source_text() {
        let source = String::from("fn a:\n    load_u8 raw\n\nfn b:\n    1\n");
        let scope_start = source.find("fn a").unwrap() as u32;
        let scope_end = source.find("\n\nfn b").unwrap() as u32;
        let load_start = source.find("load_u8").unwrap() as u32;
        let load_end = load_start + "load_u8".len() as u32;
        let sibling_edit = String::from("fn a:\n    load_u8 raw\n\nfn b:\n    2\n");
        let mut first_map = SourceMap::new();
        let mut second_map = SourceMap::new();
        let file = FileId(0);
        let capabilities =
            use_site_capabilities(raw_load_use_site(Span::new(file, load_start, load_end)));
        let first = first_map.add_with_capabilities(
            "stdlib/core/mem/raw.nepl",
            source,
            capabilities.clone(),
        );
        let second = second_map.add_with_capabilities(
            "stdlib/core/mem/raw.nepl",
            sibling_edit,
            capabilities,
        );

        assert_eq!(
            first_map.source_capability_policy_hash_for_span_scope(first, scope_start, scope_end),
            second_map.source_capability_policy_hash_for_span_scope(second, scope_start, scope_end)
        );
    }
}
