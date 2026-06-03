extern crate alloc;

use alloc::vec::Vec;

use crate::span::Span;
use crate::types::TypeId;

use super::cell_state::CellTable;
use super::collection_slot_state_table::CollectionSlotStateTable;
use super::collection_slot_summary_model::{
    CollectionSlotLifecycleSummaryDropTraversalCoverage, CollectionSlotLifecycleSummaryI32Operand,
    CollectionSlotLifecycleSummaryPlace,
};
use super::collection_slot_summary_target::instantiate_summary_target_with_aliases;
use super::initialized::ResourceCheckEngine;
use super::initialized_alias::RawCellAddressAliases;
use super::model::{Place, ResourceId};

pub(super) fn apply_drop_traversal_summary_op(
    engine: &mut ResourceCheckEngine<'_>,
    cells: &mut CellTable,
    collection_slots: &mut CollectionSlotStateTable,
    raw_aliases: &RawCellAddressAliases,
    args: &[Place],
    storage: &CollectionSlotLifecycleSummaryPlace,
    initialized_count: &CollectionSlotLifecycleSummaryI32Operand,
    expected_ty: TypeId,
    coverage: &CollectionSlotLifecycleSummaryDropTraversalCoverage,
    span: Span,
) {
    let Some(storage) = instantiate_summary_target_with_aliases(engine, args, raw_aliases, storage)
    else {
        return;
    };
    let Some((initialized_count, operand_aliases)) =
        instantiate_summary_i32_operand(engine, args, raw_aliases, initialized_count)
    else {
        return;
    };
    let effective_aliases = operand_aliases.as_ref().unwrap_or(raw_aliases);
    match coverage {
        CollectionSlotLifecycleSummaryDropTraversalCoverage::CertifiedSlots(certified_slots) => {
            let mut slots = Vec::new();
            for slot in certified_slots {
                if let Some(slot) =
                    instantiate_summary_target_with_aliases(engine, args, raw_aliases, slot)
                {
                    slots.push(slot);
                }
            }
            if slots.len() != certified_slots.len() {
                return;
            }
            engine.apply_certified_collection_slot_drop_traversal_slots_with_aliases(
                cells,
                collection_slots,
                effective_aliases,
                &storage,
                &initialized_count,
                expected_ty,
                &slots,
                span,
            );
        }
        CollectionSlotLifecycleSummaryDropTraversalCoverage::ForallInitializedRange(
            certificate,
        ) => {
            engine.apply_certified_collection_slot_drop_traversal_range_with_aliases(
                cells,
                collection_slots,
                effective_aliases,
                &storage,
                &initialized_count,
                expected_ty,
                *certificate,
                span,
            );
        }
    }
}

fn instantiate_summary_i32_operand(
    engine: &ResourceCheckEngine<'_>,
    args: &[Place],
    raw_aliases: &RawCellAddressAliases,
    operand: &CollectionSlotLifecycleSummaryI32Operand,
) -> Option<(Place, Option<RawCellAddressAliases>)> {
    match operand {
        CollectionSlotLifecycleSummaryI32Operand::Place(place) => {
            instantiate_summary_target_with_aliases(engine, args, raw_aliases, place)
                .map(|place| (place, None))
        }
        CollectionSlotLifecycleSummaryI32Operand::Offset { base, offset, ty } => {
            let base = instantiate_summary_target_with_aliases(engine, args, raw_aliases, base)?;
            let operand = replay_i32_offset_operand_place(&base, *offset, *ty);
            let mut operand_aliases = raw_aliases.clone();
            operand_aliases.add_i32_offset(&base, &operand, *offset);
            Some((operand, Some(operand_aliases)))
        }
        CollectionSlotLifecycleSummaryI32Operand::KnownI32 { value, ty } => {
            Some((Place::i32_constant(*value, *ty), None))
        }
    }
}

fn replay_i32_offset_operand_place(base: &Place, offset: i64, ty: TypeId) -> Place {
    let synthetic_id = usize::MAX
        .saturating_sub(base.projections.len())
        .saturating_sub(offset.unsigned_abs() as usize);
    Place::temporary(ResourceId(synthetic_id), ty)
}
