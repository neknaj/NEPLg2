extern crate alloc;

use alloc::vec::Vec;

use crate::types::TypeCtx;

use super::collection_slot_summary_build_ops::collect_summary_ops_from_ops;
use super::collection_slot_summary_build_state::CollectionSlotSummaryBuildState;
use super::collection_slot_summary_model::{
    CollectionSlotLifecycleFunctionSummary, CollectionSlotLifecycleFunctionSummaryIndex,
};
use super::collection_slot_summary_return_build::collect_return_facts_from_terminator;
use super::initialized::ResourceCheckEngine;
use super::initialized_alias_flow::{
    RawCellAddressReturnSummary, RawCellAddressReturnSummaryIndex,
};
use super::initialized_scalar_flow::{I32ScalarReturnSummary, I32ScalarReturnSummaryIndex};
use super::initialized_summary::RawCellInitializationFunctionSummary;
use super::initialized_summary::RawCellInitializationFunctionSummaryIndex;
use super::model::{ResourceFunction, ResourceModule};
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
    let raw_alias_summary_index = RawCellAddressReturnSummaryIndex::new(raw_alias_summaries);
    let i32_scalar_summary_index = I32ScalarReturnSummaryIndex::new(i32_scalar_summaries);
    let raw_init_summary_index = RawCellInitializationFunctionSummaryIndex::new(raw_init_summaries);
    while let Some(function_index) = worklist.pop() {
        let collection_summary_index = CollectionSlotLifecycleFunctionSummaryIndex::new(&summaries);
        let summary = function_collection_slot_lifecycle_summary(
            &module.functions[function_index],
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
        || !summary.return_slots.is_empty();
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
            &state.collection_slots,
            &engine,
            &function.params,
            &block_entry_state,
            &block.ops,
            &block.terminator,
        );
    }
    CollectionSlotLifecycleFunctionSummary {
        function: function.name.clone(),
        ops,
        return_transfers,
        return_slots,
    }
}
