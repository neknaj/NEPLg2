extern crate alloc;

use alloc::collections::BTreeSet;
use alloc::string::String;
use alloc::vec::Vec;

use crate::source_map::SourceCapabilities;
use crate::span::{FileId, Span};

use super::super::model::{ResourceFunction, ResourceMatchArm, ResourceOp, ResourceTerminator};
use super::candidate_key::{
    ResourceSummaryCacheNamespaceHash, ResourceSummarySourceCapabilityPolicyHash,
};
use super::stable_hash::ResourceSummaryStableHasher;

/// Resource summary value cache の compile-local 入力 context。
///
/// Resource IR は `SourceMap` を直接所有しないため、compiler pipeline 側で source map
/// から `FileId -> source capability policy hash` を作り、この context だけを Resource
/// initialized check へ渡す。これにより、Resource summary value key は実際の namespace
/// と source policy だけを使い、未計算値を `0` などの sentinel で代用しない。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceSummaryValueCacheContext {
    namespace_hash: u64,
    source_policy_hashes: Vec<(FileId, u64)>,
    source_policy_files: Vec<ResourceSummarySourcePolicyFile>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ResourceSummarySourcePolicyFile {
    file_id: FileId,
    canonical_path: String,
    source: String,
    capabilities: SourceCapabilities,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SourcePolicyFileRange {
    file_id: FileId,
    start: u32,
    end: u32,
}

impl ResourceSummaryValueCacheContext {
    pub fn new(namespace_hash: u64) -> Self {
        Self {
            namespace_hash,
            source_policy_hashes: Vec::new(),
            source_policy_files: Vec::new(),
        }
    }

    pub fn insert_source_policy_hash(&mut self, file_id: FileId, policy_hash: u64) {
        if let Some((_, existing)) = self
            .source_policy_hashes
            .iter_mut()
            .find(|(existing_file, _)| *existing_file == file_id)
        {
            *existing = policy_hash;
            return;
        }
        self.source_policy_hashes.push((file_id, policy_hash));
    }

    pub fn insert_source_policy_file(
        &mut self,
        file_id: FileId,
        canonical_path: &str,
        source: &str,
        capabilities: SourceCapabilities,
    ) {
        if let Some(existing) = self
            .source_policy_files
            .iter_mut()
            .find(|existing| existing.file_id == file_id)
        {
            existing.canonical_path = String::from(canonical_path);
            existing.source = String::from(source);
            existing.capabilities = capabilities;
            return;
        }
        self.source_policy_files
            .push(ResourceSummarySourcePolicyFile {
                file_id,
                canonical_path: String::from(canonical_path),
                source: String::from(source),
                capabilities,
            });
    }

    pub(super) fn namespace_hash(&self) -> ResourceSummaryCacheNamespaceHash {
        ResourceSummaryCacheNamespaceHash::from_stable_hash(self.namespace_hash)
    }

    /// 関数本文に含まれる source capability policy hash を集約する。
    ///
    /// `ResourceFunction.span` だけに依存すると、include や lowering 由来で本文内 op が
    /// 別 file の span を持つ場合に source policy の入力を落としてしまう。この関数は
    /// function / block / op / terminator / nested control-flow op の file id を集め、対応する
    /// policy hash がすべて存在する場合だけ per-function policy hash を返す。
    pub(super) fn source_capability_policy_hash_for_function(
        &self,
        function: &ResourceFunction,
    ) -> Option<ResourceSummarySourceCapabilityPolicyHash> {
        let mut file_ranges = Vec::new();
        push_span_range(&mut file_ranges, function.span);
        for block in &function.blocks {
            push_span_range(&mut file_ranges, block.span);
            collect_op_span_ranges(&block.ops, &mut file_ranges);
            push_span_range(&mut file_ranges, terminator_span(&block.terminator));
        }
        if file_ranges.is_empty() {
            return None;
        }
        let mut policy_hashes = BTreeSet::new();
        for range in file_ranges {
            policy_hashes.insert(self.source_policy_hash_for_file_range(range)?);
        }
        let mut hash = ResourceSummaryStableHasher::new("neplg2-resource-summary-source-policy-v1");
        hash.write_usize(policy_hashes.len());
        for policy_hash in policy_hashes {
            hash.write_u64(policy_hash);
        }
        Some(ResourceSummarySourceCapabilityPolicyHash::from_stable_hash(
            hash.finish(),
        ))
    }

    fn source_policy_hash_for_file(&self, file_id: FileId) -> Option<u64> {
        self.source_policy_hashes
            .iter()
            .find(|(existing_file, _)| *existing_file == file_id)
            .map(|(_, policy_hash)| *policy_hash)
    }

    fn source_policy_hash_for_file_range(&self, range: SourcePolicyFileRange) -> Option<u64> {
        if let Some(file) = self
            .source_policy_files
            .iter()
            .find(|file| file.file_id == range.file_id)
        {
            return file.capabilities.stable_scoped_policy_hash(
                file.canonical_path.as_str(),
                file.source.as_str(),
                range.start,
                range.end,
            );
        }
        self.source_policy_hash_for_file(range.file_id)
    }
}

fn push_span_range(ranges: &mut Vec<SourcePolicyFileRange>, span: Span) {
    if span == Span::dummy() {
        return;
    }
    if let Some(existing) = ranges
        .iter_mut()
        .find(|existing| existing.file_id == span.file_id)
    {
        existing.start = existing.start.min(span.start);
        existing.end = existing.end.max(span.end);
        return;
    }
    ranges.push(SourcePolicyFileRange {
        file_id: span.file_id,
        start: span.start,
        end: span.end,
    });
}

fn collect_op_span_ranges(ops: &[ResourceOp], ranges: &mut Vec<SourcePolicyFileRange>) {
    for op in ops {
        push_span_range(ranges, op_span(op));
        match op {
            ResourceOp::Branch {
                then_ops, else_ops, ..
            } => {
                collect_op_span_ranges(then_ops, ranges);
                collect_op_span_ranges(else_ops, ranges);
            }
            ResourceOp::Loop {
                condition_ops,
                body_ops,
                ..
            } => {
                collect_op_span_ranges(condition_ops, ranges);
                collect_op_span_ranges(body_ops, ranges);
            }
            ResourceOp::Match { arms, .. } => collect_match_arm_span_ranges(arms, ranges),
            ResourceOp::Expr { .. }
            | ResourceOp::DeclareLocal { .. }
            | ResourceOp::Read { .. }
            | ResourceOp::Assign { .. }
            | ResourceOp::Borrow { .. }
            | ResourceOp::Move { .. }
            | ResourceOp::Drop { .. }
            | ResourceOp::EndScope { .. }
            | ResourceOp::CallEffect { .. }
            | ResourceOp::FunctionValue { .. }
            | ResourceOp::Call { .. }
            | ResourceOp::IndirectCall { .. }
            | ResourceOp::RawMemory { .. }
            | ResourceOp::RawAddressAlias { .. }
            | ResourceOp::RawAddressView { .. }
            | ResourceOp::StorageOrigin { .. }
            | ResourceOp::CollectionSlotLifecycle { .. }
            | ResourceOp::CollectionStorageRelocate { .. }
            | ResourceOp::CollectionSlotDropTraversal { .. }
            | ResourceOp::CollectionSlotTransformRange { .. }
            | ResourceOp::Construct { .. } => {}
        }
    }
}

fn collect_match_arm_span_ranges(
    arms: &[ResourceMatchArm],
    ranges: &mut Vec<SourcePolicyFileRange>,
) {
    for arm in arms {
        push_span_range(ranges, arm.span);
        collect_op_span_ranges(&arm.ops, ranges);
    }
}

fn op_span(op: &ResourceOp) -> Span {
    match op {
        ResourceOp::Expr { span, .. }
        | ResourceOp::DeclareLocal { span, .. }
        | ResourceOp::Read { span, .. }
        | ResourceOp::Assign { span, .. }
        | ResourceOp::Borrow { span, .. }
        | ResourceOp::Move { span, .. }
        | ResourceOp::Drop { span, .. }
        | ResourceOp::EndScope { span, .. }
        | ResourceOp::CallEffect { span, .. }
        | ResourceOp::FunctionValue { span, .. }
        | ResourceOp::Call { span, .. }
        | ResourceOp::IndirectCall { span, .. }
        | ResourceOp::RawMemory { span, .. }
        | ResourceOp::RawAddressAlias { span, .. }
        | ResourceOp::RawAddressView { span, .. }
        | ResourceOp::StorageOrigin { span, .. }
        | ResourceOp::CollectionSlotLifecycle { span, .. }
        | ResourceOp::CollectionStorageRelocate { span, .. }
        | ResourceOp::CollectionSlotDropTraversal { span, .. }
        | ResourceOp::CollectionSlotTransformRange { span, .. }
        | ResourceOp::Construct { span, .. }
        | ResourceOp::Branch { span, .. }
        | ResourceOp::Loop { span, .. }
        | ResourceOp::Match { span, .. } => *span,
    }
}

fn terminator_span(terminator: &ResourceTerminator) -> Span {
    match terminator {
        ResourceTerminator::Return { span, .. }
        | ResourceTerminator::Unreachable { span }
        | ResourceTerminator::RawBody { span, .. } => *span,
    }
}

#[cfg(test)]
mod tests {
    use alloc::vec;
    use alloc::vec::Vec;

    use crate::ast::Effect;
    use crate::effects::RawMemoryOp;
    use crate::source_map::{SourceCapabilities, SourceCapabilitySpan, SourceCapabilityUseSite};
    use crate::span::{FileId, Span};
    use crate::types::TypeCtx;

    use super::super::super::model::{
        Place, ResourceBlock, ResourceBlockId, ResourceExprKind, ResourceFunction, ResourceId,
        ResourceOp, ResourceTerminator,
    };
    use super::*;

    fn simple_function(
        types: &TypeCtx,
        function_file: FileId,
        op_file: FileId,
    ) -> ResourceFunction {
        let ty = types.i32();
        let output = Place::temporary(ResourceId(0), ty);
        ResourceFunction {
            name: "source_policy".into(),
            origin_name: "source_policy".into(),
            type_params: Vec::new(),
            params: Vec::new(),
            result: ty,
            effect: Effect::Pure,
            entry_block: ResourceBlockId(0),
            blocks: vec![ResourceBlock {
                id: ResourceBlockId(0),
                ops: vec![ResourceOp::Expr {
                    kind: ResourceExprKind::LiteralI32(1),
                    output: output.clone(),
                    ty,
                    span: Span::new(op_file, 10, 11),
                }],
                terminator: ResourceTerminator::Return {
                    value: Some(output),
                    span: Span::new(function_file, 11, 12),
                },
                span: Span::new(function_file, 0, 12),
            }],
            span: Span::new(function_file, 0, 12),
        }
    }

    #[test]
    fn source_policy_hash_requires_all_function_source_files() {
        let types = TypeCtx::new();
        let function = simple_function(&types, FileId(0), FileId(1));
        let mut context = ResourceSummaryValueCacheContext::new(7);
        context.insert_source_policy_hash(FileId(0), 100);

        assert!(context
            .source_capability_policy_hash_for_function(&function)
            .is_none());

        context.insert_source_policy_hash(FileId(1), 200);

        assert!(context
            .source_capability_policy_hash_for_function(&function)
            .is_some());
    }

    #[test]
    fn source_policy_hash_rejects_functions_with_only_dummy_spans() {
        let types = TypeCtx::new();
        let ty = types.i32();
        let output = Place::temporary(ResourceId(0), ty);
        let function = ResourceFunction {
            name: "dummy".into(),
            origin_name: "dummy".into(),
            type_params: Vec::new(),
            params: Vec::new(),
            result: ty,
            effect: Effect::Pure,
            entry_block: ResourceBlockId(0),
            blocks: vec![ResourceBlock {
                id: ResourceBlockId(0),
                ops: vec![ResourceOp::Expr {
                    kind: ResourceExprKind::LiteralI32(1),
                    output: output.clone(),
                    ty,
                    span: Span::dummy(),
                }],
                terminator: ResourceTerminator::Return {
                    value: Some(output),
                    span: Span::dummy(),
                },
                span: Span::dummy(),
            }],
            span: Span::dummy(),
        };
        let mut context = ResourceSummaryValueCacheContext::new(7);
        context.insert_source_policy_hash(FileId(0), 100);

        assert!(context
            .source_capability_policy_hash_for_function(&function)
            .is_none());
    }

    #[test]
    fn source_policy_hash_tracks_namespace_independently_from_source_policy() {
        let first = ResourceSummaryValueCacheContext::new(7);
        let second = ResourceSummaryValueCacheContext::new(8);

        assert_ne!(first.namespace_hash(), second.namespace_hash());
    }

    #[test]
    fn source_policy_hash_tracks_function_file_policy_inputs() {
        let types = TypeCtx::new();
        let function = simple_function(&types, FileId(0), FileId(1));
        let mut first = ResourceSummaryValueCacheContext::new(7);
        first.insert_source_policy_hash(FileId(0), 100);
        first.insert_source_policy_hash(FileId(1), 200);
        let mut second = ResourceSummaryValueCacheContext::new(7);
        second.insert_source_policy_hash(FileId(0), 100);
        second.insert_source_policy_hash(FileId(1), 201);

        assert_ne!(
            first.source_capability_policy_hash_for_function(&function),
            second.source_capability_policy_hash_for_function(&function)
        );
    }

    #[test]
    fn source_policy_hash_prefers_scoped_file_surface_over_file_level_hash() {
        let types = TypeCtx::new();
        let function = simple_function(&types, FileId(0), FileId(0));
        let mut capabilities = SourceCapabilities::none();
        capabilities.insert_use_site(SourceCapabilityUseSite::RawMemoryOperationBoundary {
            operation: RawMemoryOp::Load,
            span: SourceCapabilitySpan::from_span(Span::new(FileId(0), 24, 31)),
        });
        let source = "fn a:\n    1\n\nfn b:\n    load_u8 raw\n";
        let mut first = ResourceSummaryValueCacheContext::new(7);
        first.insert_source_policy_hash(FileId(0), 100);
        first.insert_source_policy_file(
            FileId(0),
            "stdlib/core/mem/raw.nepl",
            source,
            capabilities.clone(),
        );
        let mut second = ResourceSummaryValueCacheContext::new(7);
        second.insert_source_policy_hash(FileId(0), 999);
        second.insert_source_policy_file(
            FileId(0),
            "stdlib/core/mem/raw.nepl",
            source,
            capabilities,
        );

        assert_eq!(
            first.source_capability_policy_hash_for_function(&function),
            second.source_capability_policy_hash_for_function(&function)
        );
    }
}
