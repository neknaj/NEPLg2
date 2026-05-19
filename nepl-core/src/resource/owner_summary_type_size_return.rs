use alloc::vec::Vec;

use super::initialized_alias::RawCellAddressAliases;
use super::model::Place;
use super::place_utils::{place_suffix_after_prefix, push_unique_place};
use super::summary::OwnerTypeSizeReturn;

pub(super) fn record_type_size_returns(
    out: &mut Vec<OwnerTypeSizeReturn>,
    raw_aliases: &RawCellAddressAliases,
    value: &Place,
) {
    for (fact_place, element_ty) in raw_aliases.i32_type_size_fact_places() {
        let mut candidates = raw_aliases.scalar_aliases_for_value(&fact_place);
        push_unique_place(&mut candidates, &fact_place);
        for candidate in candidates {
            let Some(suffix) = place_suffix_after_prefix(&candidate, value) else {
                continue;
            };
            push_unique_type_size_return(
                out,
                OwnerTypeSizeReturn {
                    suffix,
                    ty: candidate.ty,
                    element_ty,
                },
            );
        }
    }
}

fn push_unique_type_size_return(out: &mut Vec<OwnerTypeSizeReturn>, entry: OwnerTypeSizeReturn) {
    if !out.iter().any(|existing| existing == &entry) {
        out.push(entry);
    }
}
