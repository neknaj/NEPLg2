extern crate alloc;

use alloc::vec::Vec;

use super::collection_slot_summary_model::CollectionSlotLifecycleFunctionSummary;
use super::collection_slot_summary_projection::compose_translated_summary_suffix_for_params;
use super::collection_slot_summary_return_model::CollectionSlotLifecycleReturnTransfer;
use super::collection_slot_summary_return_unique::push_return_transfer;
use super::collection_slot_summary_target::{
    instantiate_summary_target_with_aliases, summary_place_for_params_with_aliases,
};
use super::function_alias::FunctionAliasTable;
use super::initialized::ResourceCheckEngine;
use super::initialized_alias::RawCellAddressAliases;
use super::model::{Place, PlaceProjection, ResourceCallTarget, ResourceLocal};

pub(super) fn collect_return_transfers_from_call_summary(
    out: &mut Vec<CollectionSlotLifecycleReturnTransfer>,
    engine: &ResourceCheckEngine<'_>,
    params: &[ResourceLocal],
    raw_aliases: &RawCellAddressAliases,
    args: &[Place],
    target: &ResourceCallTarget,
    target_suffix: &[PlaceProjection],
) {
    let ResourceCallTarget::User { name, .. } = target else {
        return;
    };
    if let Some(summary) = engine.collection_slot_summaries.get(name) {
        collect_return_transfers_from_summary(
            out,
            engine,
            params,
            raw_aliases,
            args,
            summary,
            target_suffix,
        );
    }
}

pub(super) fn collect_return_transfers_from_indirect_call_summary(
    out: &mut Vec<CollectionSlotLifecycleReturnTransfer>,
    engine: &ResourceCheckEngine<'_>,
    params: &[ResourceLocal],
    raw_aliases: &RawCellAddressAliases,
    function_aliases: &FunctionAliasTable,
    callee: &Place,
    args: &[Place],
    target_suffix: &[PlaceProjection],
) {
    for function in function_aliases.functions(callee) {
        if let Some(summary) = engine.collection_slot_summaries.get(function) {
            collect_return_transfers_from_summary(
                out,
                engine,
                params,
                raw_aliases,
                args,
                summary,
                target_suffix,
            );
        }
    }
}

fn collect_return_transfers_from_summary(
    out: &mut Vec<CollectionSlotLifecycleReturnTransfer>,
    engine: &ResourceCheckEngine<'_>,
    params: &[ResourceLocal],
    raw_aliases: &RawCellAddressAliases,
    args: &[Place],
    summary: &CollectionSlotLifecycleFunctionSummary,
    target_suffix: &[PlaceProjection],
) {
    for transfer in &summary.return_transfers {
        let Some(source) =
            instantiate_summary_target_with_aliases(engine, args, raw_aliases, &transfer.source)
        else {
            continue;
        };
        let source = raw_aliases.canonicalize_owner_cell_address(&source);
        let Some(source) = summary_place_for_params_with_aliases(params, raw_aliases, &source)
        else {
            continue;
        };
        let Some(composed_target_suffix) = compose_translated_summary_suffix_for_params(
            engine,
            args,
            params,
            target_suffix,
            &transfer.target_suffix,
        ) else {
            continue;
        };
        push_return_transfer(
            out,
            CollectionSlotLifecycleReturnTransfer {
                source,
                target_suffix: composed_target_suffix,
                target_ty: transfer.target_ty,
            },
        );
    }
}
