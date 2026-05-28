extern crate alloc;

use alloc::collections::BTreeMap;
use alloc::vec::Vec;

use crate::types::{TypeCtx, TypeId};

use super::collection_slot_summary_build_ops::collect_summary_ops_from_ops;
use super::collection_slot_summary_build_state::CollectionSlotSummaryBuildState;
use super::collection_slot_summary_model::{
    CollectionSlotLifecycleFunctionSummary, CollectionSlotLifecycleFunctionSummaryIndex,
    CollectionSlotLifecycleReturnPath, CollectionSlotLifecycleSummaryDropTraversalCoverage,
    CollectionSlotLifecycleSummaryOp,
};
use super::collection_slot_summary_relevance::collection_slot_summary_relevant_functions;
use super::collection_slot_summary_return_build::collect_return_facts_from_terminator;
use super::initialized::ResourceCheckEngine;
use super::initialized_alias_flow::{
    RawCellAddressReturnSummary, RawCellAddressReturnSummaryIndex,
};
use super::initialized_scalar_flow::{I32ScalarReturnSummary, I32ScalarReturnSummaryIndex};
use super::initialized_summary::RawCellInitializationFunctionSummary;
use super::initialized_summary::RawCellInitializationFunctionSummaryIndex;
use super::model::{ResourceFunction, ResourceModule};
use super::owner_summary_type_params::owner_summary_type_params;
use super::report::ResourceCheckDeferred;
use super::resource_summary_value_cache::ResourceSummaryValueCache;
use super::summary_worklist::SummaryWorklist;
use super::timing::ResourceFunctionTimer;

pub(super) fn compute_collection_slot_lifecycle_function_summaries(
    module: &ResourceModule,
    types: &TypeCtx,
    raw_alias_summaries: &[RawCellAddressReturnSummary],
    i32_scalar_summaries: &[I32ScalarReturnSummary],
    raw_init_summaries: &[RawCellInitializationFunctionSummary],
    mut summary_value_cache: Option<&mut ResourceSummaryValueCache>,
) -> Vec<CollectionSlotLifecycleFunctionSummary> {
    let mut summaries = Vec::new();
    let relevant_functions = collection_slot_summary_relevant_functions(module, types);
    let mut worklist = SummaryWorklist::new_filtered(module, relevant_functions);
    let raw_alias_summary_index = RawCellAddressReturnSummaryIndex::new(raw_alias_summaries);
    let i32_scalar_summary_index = I32ScalarReturnSummaryIndex::new(i32_scalar_summaries);
    let raw_init_summary_index = RawCellInitializationFunctionSummaryIndex::new(raw_init_summaries);
    while let Some(function_index) = worklist.pop() {
        let collection_summary_index = CollectionSlotLifecycleFunctionSummaryIndex::new(&summaries);
        let function = &module.functions[function_index];
        let function_start = ResourceFunctionTimer::start();
        let summary = function_collection_slot_lifecycle_summary(
            function,
            types,
            &raw_alias_summary_index,
            &i32_scalar_summary_index,
            &raw_init_summary_index,
            &collection_summary_index,
        );
        function_start.log("collection_slot_summary", function);
        if update_collection_slot_lifecycle_summary(&mut summaries, summary) {
            worklist.notify_changed(function_index);
        }
    }
    if let Some(cache) = summary_value_cache.as_deref_mut() {
        record_resource_summary_value_cache_bypass_candidates(cache, types, module, &summaries);
    }
    summaries
}

fn record_resource_summary_value_cache_bypass_candidates(
    cache: &mut ResourceSummaryValueCache,
    types: &TypeCtx,
    module: &ResourceModule,
    summaries: &[CollectionSlotLifecycleFunctionSummary],
) {
    let mut functions = BTreeMap::new();
    for function in &module.functions {
        functions.insert(function.name.as_str(), function);
    }
    for summary in summaries {
        let Some(function) = functions.get(summary.function.as_str()) else {
            continue;
        };
        record_resource_summary_value_cache_bypass_candidates_from_summary(
            cache, types, function, summary,
        );
    }
}

fn record_resource_summary_value_cache_bypass_candidates_from_summary(
    cache: &mut ResourceSummaryValueCache,
    types: &TypeCtx,
    function: &ResourceFunction,
    summary: &CollectionSlotLifecycleFunctionSummary,
) {
    record_resource_summary_value_cache_bypass_candidates_from_top_level_ops(
        cache,
        types,
        function,
        &summary.type_params,
        &summary.ops,
    );
}

fn record_resource_summary_value_cache_bypass_candidates_from_top_level_ops(
    cache: &mut ResourceSummaryValueCache,
    types: &TypeCtx,
    function: &ResourceFunction,
    type_params: &[TypeId],
    ops: &[CollectionSlotLifecycleSummaryOp],
) {
    // 初期 MVP では top-level leaf だけを store 候補にする。
    // return path や control-flow container 内の leaf は、分岐条件や path precondition
    // と一体で stable mirror を設計する必要があるため、この counter へ含めない。
    for op in ops {
        match op {
            CollectionSlotLifecycleSummaryOp::DropTraversal {
                coverage:
                    CollectionSlotLifecycleSummaryDropTraversalCoverage::ForallInitializedRange(_),
                ..
            } => cache.record_drop_traversal_forall_bypass_if_stable(
                types,
                function,
                type_params,
                op,
            ),
            CollectionSlotLifecycleSummaryOp::Event { .. }
            | CollectionSlotLifecycleSummaryOp::Relocate { .. }
            | CollectionSlotLifecycleSummaryOp::DropTraversal { .. }
            | CollectionSlotLifecycleSummaryOp::TransformRange { .. }
            | CollectionSlotLifecycleSummaryOp::TransformRangeSourceDrain { .. }
            | CollectionSlotLifecycleSummaryOp::Merge { .. }
            | CollectionSlotLifecycleSummaryOp::Loop { .. } => {}
        }
    }
}

fn update_collection_slot_lifecycle_summary(
    summaries: &mut Vec<CollectionSlotLifecycleFunctionSummary>,
    summary: CollectionSlotLifecycleFunctionSummary,
) -> bool {
    let has_facts = !summary.ops.is_empty()
        || !summary.return_transfers.is_empty()
        || !summary.return_slots.is_empty()
        || !summary.return_ranges.is_empty()
        || !summary.return_paths.is_empty();
    let position = summaries
        .iter()
        .position(|existing| existing.function == summary.function);
    match (has_facts, position) {
        (true, Some(index)) if summaries[index] == summary => false,
        (true, Some(index)) => {
            summaries[index] = summary;
            true
        }
        (true, None) => {
            summaries.push(summary);
            true
        }
        (false, Some(index)) => {
            summaries.remove(index);
            true
        }
        (false, None) => false,
    }
}

fn function_collection_slot_lifecycle_summary(
    function: &ResourceFunction,
    types: &TypeCtx,
    raw_alias_summaries: &RawCellAddressReturnSummaryIndex<'_>,
    i32_scalar_summaries: &I32ScalarReturnSummaryIndex<'_>,
    raw_init_summaries: &RawCellInitializationFunctionSummaryIndex<'_>,
    collection_slot_summaries: &CollectionSlotLifecycleFunctionSummaryIndex<'_>,
) -> CollectionSlotLifecycleFunctionSummary {
    let mut engine = ResourceCheckEngine {
        function: function.name.as_str(),
        types,
        raw_alias_summaries,
        i32_scalar_summaries,
        raw_init_summaries,
        collection_slot_summaries,
        transform_range_certificates: None,
        diagnostics: Vec::new(),
        auto_drop_points: Vec::new(),
        deferred: ResourceCheckDeferred::default(),
        path_alternatives: Default::default(),
    };
    let mut state = CollectionSlotSummaryBuildState::new(types, function);
    let mut ops = Vec::new();
    let mut return_transfers = Vec::new();
    let mut return_slots = Vec::new();
    let mut return_ranges = Vec::new();
    let mut return_paths = Vec::<CollectionSlotLifecycleReturnPath>::new();
    for block in &function.blocks {
        let block_entry_state = state.clone();
        collect_summary_ops_from_ops(
            &mut ops,
            &mut engine,
            &mut state,
            &function.params,
            collection_slot_summaries,
            &block.ops,
        );
        collect_return_facts_from_terminator(
            &mut return_transfers,
            &mut return_slots,
            &mut return_ranges,
            &mut return_paths,
            &state,
            &engine,
            &function.params,
            &block_entry_state,
            &block.ops,
            &block.terminator,
        );
    }
    return_paths.retain(collection_return_path_has_lifecycle_facts);
    CollectionSlotLifecycleFunctionSummary {
        function: function.name.clone(),
        type_params: owner_summary_type_params(types, function),
        ops,
        return_transfers,
        return_slots,
        return_ranges,
        return_paths,
    }
}

fn collection_return_path_has_lifecycle_facts(path: &CollectionSlotLifecycleReturnPath) -> bool {
    !path.ops.is_empty()
        || !path.return_transfers.is_empty()
        || !path.return_slots.is_empty()
        || !path.return_ranges.is_empty()
}

#[cfg(test)]
mod tests {
    use alloc::string::{String, ToString};
    use alloc::vec;
    use alloc::vec::Vec;

    use crate::source_map::CompilerMemoryType;
    use crate::span::Span;
    use crate::types::{TypeCtx, TypeId, TypeKind};

    use super::super::collection_slot_lifecycle::CollectionSlotLifecycleEvent;
    use super::super::collection_slot_summary_model::{
        CollectionSlotInitializedRangeDropTraversalCertificate,
        CollectionSlotInitializedRangeDropTraversalProof,
        CollectionSlotLifecycleSummaryDropTraversalCoverage,
        CollectionSlotLifecycleSummaryI32Operand, CollectionSlotLifecycleSummaryOp,
    };
    use super::super::i32_scalar_return_facts::I32ScalarReturnFacts;
    use super::super::model::{
        Place, ResourceBlock, ResourceBlockId, ResourceFunction, ResourceLocal, ResourceModule,
        ResourceTerminator,
    };
    use super::super::resource_summary_value_cache::ResourceSummaryValueCache;
    use super::super::summary_projection::SummaryPlace;
    use super::*;

    fn identity_storage_function(storage_ty: TypeId) -> ResourceFunction {
        let span = Span::dummy();
        let param = Place::local("storage".to_string(), storage_ty);
        ResourceFunction {
            name: "identity_storage".to_string(),
            origin_name: "identity_storage".to_string(),
            type_params: Vec::new(),
            params: vec![ResourceLocal {
                name: "storage".to_string(),
                ty: storage_ty,
                mutable: false,
                place: param.clone(),
            }],
            result: storage_ty,
            effect: crate::ast::Effect::Pure,
            entry_block: ResourceBlockId(0),
            blocks: vec![ResourceBlock {
                id: ResourceBlockId(0),
                ops: Vec::new(),
                terminator: ResourceTerminator::Return {
                    value: Some(param),
                    span,
                },
                span,
            }],
            span,
        }
    }

    fn collection_storage_marker_function(
        storage_ty: TypeId,
        value_ty: TypeId,
    ) -> ResourceFunction {
        let span = Span::dummy();
        let storage = Place::local("storage".to_string(), storage_ty);
        ResourceFunction {
            name: "mark_collection_storage".to_string(),
            origin_name: "mark_collection_storage".to_string(),
            type_params: Vec::new(),
            params: Vec::new(),
            result: value_ty,
            effect: crate::ast::Effect::Pure,
            entry_block: ResourceBlockId(0),
            blocks: vec![ResourceBlock {
                id: ResourceBlockId(0),
                ops: vec![
                    super::super::model::ResourceOp::CollectionSlotLifecycle {
                        target: storage.clone(),
                        event: CollectionSlotLifecycleEvent::InitializeEmpty { value_ty },
                        span,
                    },
                    super::super::model::ResourceOp::Call {
                        output: Place::local("storage_out".to_string(), storage_ty),
                        target: super::super::model::ResourceCallTarget::User {
                            name: "identity_storage".to_string(),
                            type_args: Vec::new(),
                        },
                        args: vec![storage],
                        effect: super::super::model::EffectOp::Pure,
                        span,
                    },
                ],
                terminator: ResourceTerminator::Return { value: None, span },
                span,
            }],
            span,
        }
    }

    fn register_empty_struct(types: &mut TypeCtx, name: &str) -> TypeId {
        types.register_named(
            String::from(name),
            TypeKind::Struct {
                name: String::from(name),
                type_params: vec![],
                fields: vec![],
                field_names: vec![],
            },
        )
    }

    fn register_region_token(types: &mut TypeCtx) -> TypeId {
        let raw_ty = types.i32();
        let value_ty = types.fresh_var(Some("T".to_string()));
        let region_token_ty = types.register_named(
            "RegionToken".to_string(),
            TypeKind::Struct {
                name: "RegionToken".to_string(),
                type_params: vec![value_ty],
                fields: vec![raw_ty, raw_ty],
                field_names: vec!["raw".to_string(), "size".to_string()],
            },
        );
        types.mark_compiler_memory_type(region_token_ty, CompilerMemoryType::OwnerToken);
        region_token_ty
    }

    fn summary_place(parameter_index: usize, ty: TypeId) -> SummaryPlace {
        SummaryPlace {
            parameter_index,
            suffix: Vec::new(),
            ty,
        }
    }

    fn forall_drop_traversal_op() -> CollectionSlotLifecycleSummaryOp {
        CollectionSlotLifecycleSummaryOp::DropTraversal {
            storage: summary_place(0, TypeId(0)),
            initialized_count: CollectionSlotLifecycleSummaryI32Operand::KnownI32 {
                value: 1,
                ty: TypeId(1),
            },
            expected_ty: TypeId(2),
            coverage: CollectionSlotLifecycleSummaryDropTraversalCoverage::ForallInitializedRange(
                CollectionSlotInitializedRangeDropTraversalCertificate {
                    element_stride: 4,
                    drop_proof: CollectionSlotInitializedRangeDropTraversalProof::StateOnly,
                },
            ),
        }
    }

    /// Resource summary value cache の初期 MVP は、固定点収束後の top-level
    /// `DropTraversal + ForallInitializedRange` だけを store 候補として扱う。
    /// return path や control-flow container 内の leaf は、stable mirror の key/value
    /// を別途設計するまで候補数に含めない。
    #[test]
    fn resource_summary_value_bypass_counts_only_final_top_level_forall_drop_traversal() {
        let mut cache = ResourceSummaryValueCache::new();
        let types = TypeCtx::new();
        let function = identity_storage_function(types.i32());
        let module = ResourceModule {
            functions: vec![function],
            entry: None,
            string_literals: Vec::new(),
        };
        let summary = CollectionSlotLifecycleFunctionSummary {
            function: "identity_storage".to_string(),
            type_params: Vec::new(),
            ops: vec![
                forall_drop_traversal_op(),
                CollectionSlotLifecycleSummaryOp::DropTraversal {
                    storage: summary_place(0, TypeId(0)),
                    initialized_count: CollectionSlotLifecycleSummaryI32Operand::KnownI32 {
                        value: 1,
                        ty: TypeId(1),
                    },
                    expected_ty: TypeId(2),
                    coverage: CollectionSlotLifecycleSummaryDropTraversalCoverage::CertifiedSlots(
                        vec![summary_place(0, TypeId(2))],
                    ),
                },
                CollectionSlotLifecycleSummaryOp::Merge {
                    paths: vec![vec![forall_drop_traversal_op()]],
                },
                CollectionSlotLifecycleSummaryOp::Loop {
                    condition_ops: vec![forall_drop_traversal_op()],
                    body_ops: vec![forall_drop_traversal_op()],
                },
            ],
            return_transfers: Vec::new(),
            return_slots: Vec::new(),
            return_ranges: Vec::new(),
            return_paths: vec![CollectionSlotLifecycleReturnPath {
                return_variant: None,
                preconditions: Vec::new(),
                ops: vec![forall_drop_traversal_op()],
                return_transfers: Vec::new(),
                return_slots: Vec::new(),
                return_ranges: Vec::new(),
                i32_scalar_facts: I32ScalarReturnFacts::default(),
            }],
        };

        record_resource_summary_value_cache_bypass_candidates(
            &mut cache,
            &types,
            &module,
            &[summary],
        );

        let stats = cache.stats();
        assert_eq!(stats.resource_summary_value_bypasses, 1);
        assert_eq!(
            stats.resource_summary_value_drop_traversal_forall_bypasses,
            1
        );
    }

    /// Resource summary value cache の初期 store 候補は、summary value だけでなく
    /// function body hash も安定化できる場合に限る。raw body は本文が
    /// `ResourceFunction` に残らないため、raw source/body hash を key に追加するまで
    /// 候補数に含めない。
    #[test]
    fn resource_summary_value_bypass_rejects_unstable_function_body_hash() {
        let mut cache = ResourceSummaryValueCache::new();
        let types = TypeCtx::new();
        let span = Span::dummy();
        let module = ResourceModule {
            functions: vec![ResourceFunction {
                name: "raw_body".to_string(),
                origin_name: "raw_body".to_string(),
                type_params: Vec::new(),
                params: Vec::new(),
                result: types.i32(),
                effect: crate::ast::Effect::Pure,
                entry_block: ResourceBlockId(0),
                blocks: vec![ResourceBlock {
                    id: ResourceBlockId(0),
                    ops: Vec::new(),
                    terminator: ResourceTerminator::RawBody {
                        kind: super::super::model::RawBodyKind::Wasm,
                        span,
                    },
                    span,
                }],
                span,
            }],
            entry: None,
            string_literals: Vec::new(),
        };
        let summary = CollectionSlotLifecycleFunctionSummary {
            function: "raw_body".to_string(),
            type_params: Vec::new(),
            ops: vec![forall_drop_traversal_op()],
            return_transfers: Vec::new(),
            return_slots: Vec::new(),
            return_ranges: Vec::new(),
            return_paths: Vec::new(),
        };

        record_resource_summary_value_cache_bypass_candidates(
            &mut cache,
            &types,
            &module,
            &[summary],
        );

        let stats = cache.stats();
        assert_eq!(stats.resource_summary_value_bypasses, 0);
        assert_eq!(
            stats.resource_summary_value_drop_traversal_forall_bypasses,
            0
        );
    }

    /// function body と stable mirror value が保存可能でも、function-local type parameter
    /// boundary が再投影できない場合は Resource summary value key を安全に作れない。
    /// anonymous type variable は compile session の arena slot に依存するため、初期
    /// cache 候補から外す。
    #[test]
    fn resource_summary_value_bypass_rejects_unstable_type_parameter_boundary() {
        let mut cache = ResourceSummaryValueCache::new();
        let mut types = TypeCtx::new();
        let function = identity_storage_function(types.i32());
        let module = ResourceModule {
            functions: vec![function],
            entry: None,
            string_literals: Vec::new(),
        };
        let summary = CollectionSlotLifecycleFunctionSummary {
            function: "identity_storage".to_string(),
            type_params: vec![types.fresh_var(None)],
            ops: vec![forall_drop_traversal_op()],
            return_transfers: Vec::new(),
            return_slots: Vec::new(),
            return_ranges: Vec::new(),
            return_paths: Vec::new(),
        };

        record_resource_summary_value_cache_bypass_candidates(
            &mut cache,
            &types,
            &module,
            &[summary],
        );

        let stats = cache.stats();
        assert_eq!(stats.resource_summary_value_bypasses, 0);
        assert_eq!(
            stats.resource_summary_value_drop_traversal_forall_bypasses,
            0
        );
    }

    /// Resource summary value key は function identity を必須入力にする。
    /// `ResourceFunction.name` または `origin_name` が空なら、compile session 間で対応する
    /// callable 境界を特定できないため、stable mirror value が作れても候補数に含めない。
    #[test]
    fn resource_summary_value_bypass_rejects_empty_function_identity() {
        let mut cache = ResourceSummaryValueCache::new();
        let types = TypeCtx::new();
        let mut function = identity_storage_function(types.i32());
        function.name.clear();
        let module = ResourceModule {
            functions: vec![function],
            entry: None,
            string_literals: Vec::new(),
        };
        let summary = CollectionSlotLifecycleFunctionSummary {
            function: String::new(),
            type_params: Vec::new(),
            ops: vec![forall_drop_traversal_op()],
            return_transfers: Vec::new(),
            return_slots: Vec::new(),
            return_ranges: Vec::new(),
            return_paths: Vec::new(),
        };

        record_resource_summary_value_cache_bypass_candidates(
            &mut cache,
            &types,
            &module,
            &[summary],
        );

        let stats = cache.stats();
        assert_eq!(stats.resource_summary_value_bypasses, 0);
        assert_eq!(
            stats.resource_summary_value_drop_traversal_forall_bypasses,
            0
        );
    }

    #[test]
    fn collection_slot_summary_keeps_identity_transfer_for_non_copy_owner_token_storage() {
        let mut types = TypeCtx::new();
        types.set_copy_trait_enabled(true);
        types.register_copy_impl_target(types.unit());
        let payload_ty = register_empty_struct(&mut types, "OwnedPayload");
        let region_token = register_region_token(&mut types);
        let storage_ty = types.apply(region_token, vec![payload_ty]);
        let module = ResourceModule {
            functions: vec![
                collection_storage_marker_function(storage_ty, payload_ty),
                identity_storage_function(storage_ty),
            ],
            entry: None,
            string_literals: vec![],
        };

        let summaries = compute_collection_slot_lifecycle_function_summaries(
            &module,
            &types,
            &[],
            &[],
            &[],
            None,
        );

        let identity_summary = summaries
            .iter()
            .find(|summary| summary.function == "identity_storage")
            .expect("identity_storage should keep its return transfer summary");
        assert_eq!(identity_summary.return_transfers.len(), 1, "{summaries:#?}");
    }
}
