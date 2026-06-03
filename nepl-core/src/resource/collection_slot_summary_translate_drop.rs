extern crate alloc;

use alloc::vec::Vec;

use crate::types::TypeId;

use super::collection_slot_summary_model::{
    CollectionSlotLifecycleSummaryDropTraversalCoverage, CollectionSlotLifecycleSummaryI32Operand,
    CollectionSlotLifecycleSummaryOp, CollectionSlotLifecycleSummaryPlace,
};
use super::collection_slot_summary_i32_operand::summary_i32_operand_for_params_with_aliases;
use super::collection_slot_summary_target::{
    instantiate_summary_target_with_aliases, summary_place_for_params_with_aliases_and_types,
    translate_summary_target_for_params_with_aliases,
};
use super::initialized::ResourceCheckEngine;
use super::initialized_alias::RawCellAddressAliases;
use super::model::{Place, ResourceId, ResourceLocal};

pub(super) fn translate_drop_traversal_summary_op(
    out: &mut Vec<CollectionSlotLifecycleSummaryOp>,
    engine: &ResourceCheckEngine<'_>,
    args: &[Place],
    params: &[ResourceLocal],
    raw_aliases: &RawCellAddressAliases,
    storage: &CollectionSlotLifecycleSummaryPlace,
    initialized_count: &CollectionSlotLifecycleSummaryI32Operand,
    expected_ty: TypeId,
    coverage: &CollectionSlotLifecycleSummaryDropTraversalCoverage,
) {
    let Some(storage) = translate_summary_place(engine, args, params, raw_aliases, storage) else {
        return;
    };
    let Some(initialized_count) =
        translate_i32_operand(engine, args, params, raw_aliases, initialized_count)
    else {
        return;
    };
    let Some(coverage) =
        translate_drop_traversal_coverage(engine, args, params, raw_aliases, coverage)
    else {
        return;
    };
    out.push(CollectionSlotLifecycleSummaryOp::DropTraversal {
        storage,
        initialized_count,
        expected_ty,
        coverage,
    });
}

fn translate_i32_operand(
    engine: &ResourceCheckEngine<'_>,
    args: &[Place],
    params: &[ResourceLocal],
    raw_aliases: &RawCellAddressAliases,
    operand: &CollectionSlotLifecycleSummaryI32Operand,
) -> Option<CollectionSlotLifecycleSummaryI32Operand> {
    match operand {
        CollectionSlotLifecycleSummaryI32Operand::KnownI32 { value, ty } => {
            Some(CollectionSlotLifecycleSummaryI32Operand::KnownI32 {
                value: *value,
                ty: *ty,
            })
        }
        CollectionSlotLifecycleSummaryI32Operand::Place(place) => {
            let actual = instantiate_summary_target_with_aliases(engine, args, raw_aliases, place)?;
            summary_i32_operand_for_params_with_aliases(params, raw_aliases, &actual)
                .or_else(|| {
                    translate_summary_place(engine, args, params, raw_aliases, place)
                        .map(CollectionSlotLifecycleSummaryI32Operand::Place)
                })
        }
        CollectionSlotLifecycleSummaryI32Operand::Offset { base, offset, ty } => {
            let actual_base =
                instantiate_summary_target_with_aliases(engine, args, raw_aliases, base)?;
            let operand_place = translated_i32_offset_operand_place(&actual_base, *offset, *ty);
            let mut operand_aliases = raw_aliases.clone();
            operand_aliases.add_i32_offset(&actual_base, &operand_place, *offset);
            summary_i32_operand_for_params_with_aliases(params, &operand_aliases, &operand_place)
        }
    }
}

fn translated_i32_offset_operand_place(base: &Place, offset: i64, ty: TypeId) -> Place {
    let synthetic_id = usize::MAX
        .saturating_sub(base.projections.len())
        .saturating_sub(offset.unsigned_abs() as usize);
    Place::temporary(ResourceId(synthetic_id), ty)
}

fn translate_drop_traversal_coverage(
    engine: &ResourceCheckEngine<'_>,
    args: &[Place],
    params: &[ResourceLocal],
    raw_aliases: &RawCellAddressAliases,
    coverage: &CollectionSlotLifecycleSummaryDropTraversalCoverage,
) -> Option<CollectionSlotLifecycleSummaryDropTraversalCoverage> {
    match coverage {
        CollectionSlotLifecycleSummaryDropTraversalCoverage::CertifiedSlots(certified_slots) => {
            Some(
                CollectionSlotLifecycleSummaryDropTraversalCoverage::CertifiedSlots(
                    translate_summary_places(engine, args, params, raw_aliases, certified_slots)?,
                ),
            )
        }
        CollectionSlotLifecycleSummaryDropTraversalCoverage::ForallInitializedRange(
            certificate,
        ) => Some(
            CollectionSlotLifecycleSummaryDropTraversalCoverage::ForallInitializedRange(
                *certificate,
            ),
        ),
    }
}

fn translate_summary_places(
    engine: &ResourceCheckEngine<'_>,
    args: &[Place],
    params: &[ResourceLocal],
    raw_aliases: &RawCellAddressAliases,
    places: &[CollectionSlotLifecycleSummaryPlace],
) -> Option<Vec<CollectionSlotLifecycleSummaryPlace>> {
    let mut translated = Vec::new();
    for place in places {
        translated.push(translate_summary_place(
            engine,
            args,
            params,
            raw_aliases,
            place,
        )?);
    }
    Some(translated)
}

fn translate_summary_place(
    engine: &ResourceCheckEngine<'_>,
    args: &[Place],
    params: &[ResourceLocal],
    raw_aliases: &RawCellAddressAliases,
    place: &CollectionSlotLifecycleSummaryPlace,
) -> Option<CollectionSlotLifecycleSummaryPlace> {
    if let Some(place) =
        translate_summary_target_for_params_with_aliases(engine, args, params, raw_aliases, place)
    {
        return Some(place);
    }
    let actual = instantiate_summary_target_with_aliases(engine, args, raw_aliases, place)?;
    let actual = raw_aliases.canonicalize_owner_cell_address(&actual);
    summary_place_for_params_with_aliases_and_types(
        params,
        Some(engine.types),
        raw_aliases,
        &actual,
    )
}
