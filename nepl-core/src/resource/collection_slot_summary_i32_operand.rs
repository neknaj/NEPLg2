use super::collection_slot_summary_model::CollectionSlotLifecycleSummaryI32Operand;
use super::collection_slot_summary_target::summary_place_for_params_with_aliases;
use super::initialized_alias::RawCellAddressAliases;
use super::model::{Place, ResourceLocal};

pub(super) fn summary_i32_operand_for_params_with_aliases(
    params: &[ResourceLocal],
    raw_aliases: &RawCellAddressAliases,
    place: &Place,
) -> Option<CollectionSlotLifecycleSummaryI32Operand> {
    if let Some(operand) = summary_i32_offset_operand_for_params(params, raw_aliases, place) {
        return Some(operand);
    }
    if let Some(summary) = summary_place_for_params_with_aliases(params, raw_aliases, place) {
        return Some(CollectionSlotLifecycleSummaryI32Operand::Place(summary));
    }
    raw_aliases
        .i32_value(place)
        .map(|value| CollectionSlotLifecycleSummaryI32Operand::KnownI32 {
            value,
            ty: place.ty,
        })
}

fn summary_i32_offset_operand_for_params(
    params: &[ResourceLocal],
    raw_aliases: &RawCellAddressAliases,
    place: &Place,
) -> Option<CollectionSlotLifecycleSummaryI32Operand> {
    for (source, offset) in raw_aliases.i32_offset_sources(place) {
        let Some(base) = summary_place_for_params_with_aliases(params, raw_aliases, &source) else {
            continue;
        };
        if offset == 0 {
            return Some(CollectionSlotLifecycleSummaryI32Operand::Place(base));
        }
        return Some(CollectionSlotLifecycleSummaryI32Operand::Offset {
            base,
            offset,
            ty: place.ty,
        });
    }
    None
}
