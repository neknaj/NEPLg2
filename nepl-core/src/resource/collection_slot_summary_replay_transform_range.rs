use crate::span::Span;
use crate::types::TypeId;

use super::cell_state::CellTable;
use super::collection_slot_state_table::CollectionSlotStateTable;
use super::collection_slot_summary_model::{
    CollectionSlotLifecycleSummaryPlace, CollectionSlotTransformRangeCertificate,
};
use super::collection_slot_summary_target::instantiate_summary_target_with_aliases;
use super::initialized::ResourceCheckEngine;
use super::initialized_alias::RawCellAddressAliases;
use super::model::Place;

pub(super) fn apply_transform_range_summary_op(
    engine: &mut ResourceCheckEngine<'_>,
    cells: &mut CellTable,
    collection_slots: &mut CollectionSlotStateTable,
    raw_aliases: &RawCellAddressAliases,
    args: &[Place],
    source_storage: &CollectionSlotLifecycleSummaryPlace,
    source_initialized_count: &CollectionSlotLifecycleSummaryPlace,
    output_storage: &CollectionSlotLifecycleSummaryPlace,
    output_initialized_count: &CollectionSlotLifecycleSummaryPlace,
    expected_ty: TypeId,
    certificate: CollectionSlotTransformRangeCertificate,
    span: Span,
) {
    let Some(source_storage) =
        instantiate_summary_target_with_aliases(engine, args, raw_aliases, source_storage)
    else {
        return;
    };
    let Some(source_initialized_count) = instantiate_summary_target_with_aliases(
        engine,
        args,
        raw_aliases,
        source_initialized_count,
    ) else {
        return;
    };
    let Some(output_storage) =
        instantiate_summary_target_with_aliases(engine, args, raw_aliases, output_storage)
    else {
        return;
    };
    let Some(output_initialized_count) = instantiate_summary_target_with_aliases(
        engine,
        args,
        raw_aliases,
        output_initialized_count,
    ) else {
        return;
    };
    engine.apply_certified_collection_slot_transform_range_with_aliases(
        cells,
        collection_slots,
        raw_aliases,
        &source_storage,
        &source_initialized_count,
        &output_storage,
        &output_initialized_count,
        expected_ty,
        certificate,
        span,
    );
}
