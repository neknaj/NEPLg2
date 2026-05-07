extern crate alloc;

use alloc::vec::Vec;

use super::cell_state::{place_suffix_after_address_prefix, CellTable};
use super::initialized_alias::RawCellAddressAliases;
use super::initialized_summary_byte_range_model::RawCellInitializationReturnCount;
use super::model::{CellState, Place};

pub(super) fn collect_return_count_sources(
    cells: &CellTable,
    raw_aliases: &RawCellAddressAliases,
    count: &Place,
    return_aliases: &[Place],
) -> Vec<RawCellInitializationReturnCount> {
    let count = raw_aliases.canonicalize_scalar(count);
    let mut out = Vec::new();
    if let Some(value) = raw_aliases.i32_value(&count) {
        push_unique_return_count(
            &mut out,
            RawCellInitializationReturnCount::KnownI32 {
                value,
                ty: count.ty,
            },
        );
    }
    for entry in cells.entries() {
        if !matches!(entry.state, CellState::Initialized(_)) {
            continue;
        }
        if raw_aliases.canonicalize_scalar(&entry.place) != count {
            continue;
        }
        for entry_alias in raw_aliases.aliases_for(&entry.place) {
            for return_alias in return_aliases {
                let Some(suffix) = place_suffix_after_address_prefix(&entry_alias, return_alias)
                else {
                    continue;
                };
                push_unique_return_count(
                    &mut out,
                    RawCellInitializationReturnCount::ReturnValueProjection {
                        suffix,
                        ty: entry_alias.ty,
                    },
                );
            }
        }
    }
    out
}

fn push_unique_return_count(
    counts: &mut Vec<RawCellInitializationReturnCount>,
    count: RawCellInitializationReturnCount,
) {
    if !counts.iter().any(|existing| existing == &count) {
        counts.push(count);
    }
}
