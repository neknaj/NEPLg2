extern crate alloc;

use alloc::vec::Vec;

use crate::types::TypeCtx;

use super::collection_slot_summary_build_ops::collect_summary_ops_from_ops;
use super::collection_slot_summary_build_state::CollectionSlotSummaryBuildState;
use super::collection_slot_summary_model::{
    CollectionSlotLifecycleFunctionSummary, CollectionSlotLifecycleFunctionSummaryIndex,
    CollectionSlotLifecycleReturnPath,
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
use super::summary_worklist::SummaryWorklist;

pub(super) fn compute_collection_slot_lifecycle_function_summaries(
    module: &ResourceModule,
    types: &TypeCtx,
    raw_alias_summaries: &[RawCellAddressReturnSummary],
    i32_scalar_summaries: &[I32ScalarReturnSummary],
    raw_init_summaries: &[RawCellInitializationFunctionSummary],
) -> Vec<CollectionSlotLifecycleFunctionSummary> {
    let mut worklist = SummaryWorklist::new(module);
    let mut summaries = Vec::new();
    let relevant_functions = collection_slot_summary_relevant_functions(module, types);
    let raw_alias_summary_index = RawCellAddressReturnSummaryIndex::new(raw_alias_summaries);
    let i32_scalar_summary_index = I32ScalarReturnSummaryIndex::new(i32_scalar_summaries);
    let raw_init_summary_index = RawCellInitializationFunctionSummaryIndex::new(raw_init_summaries);
    while let Some(function_index) = worklist.pop() {
        let collection_summary_index = CollectionSlotLifecycleFunctionSummaryIndex::new(&summaries);
        let function = &module.functions[function_index];
        if !relevant_functions[function_index] {
            continue;
        }
        let summary = function_collection_slot_lifecycle_summary(
            function,
            types,
            &raw_alias_summary_index,
            &i32_scalar_summary_index,
            &raw_init_summary_index,
            &collection_summary_index,
        );
        if update_collection_slot_lifecycle_summary(&mut summaries, summary) {
            worklist.notify_changed(function_index);
        }
    }
    summaries
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

    use crate::span::Span;
    use crate::types::{TypeCtx, TypeId, TypeKind};

    use super::super::model::{
        Place, ResourceBlock, ResourceBlockId, ResourceFunction, ResourceLocal, ResourceModule,
        ResourceTerminator,
    };
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

    #[test]
    fn collection_slot_summary_keeps_identity_transfer_for_structural_copy_storage() {
        let mut types = TypeCtx::new();
        types.set_copy_trait_enabled(true);
        types.register_copy_impl_target(types.unit());
        let copied_payload_ty = register_empty_struct(&mut types, "Owned");
        types.register_copy_impl_target(copied_payload_ty);
        let storage_ty = register_empty_struct(&mut types, "CollectionStorage");
        let module = ResourceModule {
            functions: vec![identity_storage_function(storage_ty)],
            entry: None,
            string_literals: vec![],
        };

        let summaries =
            compute_collection_slot_lifecycle_function_summaries(&module, &types, &[], &[], &[]);

        assert_eq!(summaries.len(), 1, "summaries: {summaries:#?}");
        assert_eq!(summaries[0].return_transfers.len(), 1, "{summaries:#?}");
    }
}
