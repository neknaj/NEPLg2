use alloc::vec::Vec;

use super::initialized_alias::RawCellAddressAliases;
use super::model::Place;
use super::place_utils::{place_suffix_after_prefix, push_unique_place};
use super::summary::OwnerHostSizeReturn;

pub(super) fn record_host_size_returns(
    out: &mut Vec<OwnerHostSizeReturn>,
    raw_aliases: &RawCellAddressAliases,
    value: &Place,
) {
    for (fact_place, kind) in raw_aliases.host_size_fact_places() {
        let mut candidates = raw_aliases.scalar_aliases_for_value(&fact_place);
        push_unique_place(&mut candidates, &fact_place);
        for candidate in candidates {
            let Some(suffix) = place_suffix_after_prefix(&candidate, value) else {
                continue;
            };
            push_unique_host_size_return(
                out,
                OwnerHostSizeReturn {
                    suffix,
                    ty: candidate.ty,
                    kind,
                },
            );
        }
    }
}

fn push_unique_host_size_return(out: &mut Vec<OwnerHostSizeReturn>, entry: OwnerHostSizeReturn) {
    if !out.iter().any(|existing| existing == &entry) {
        out.push(entry);
    }
}
