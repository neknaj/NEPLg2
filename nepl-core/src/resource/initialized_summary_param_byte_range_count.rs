extern crate alloc;

use alloc::vec::Vec;

use super::cell_state::{place_suffix_after_address_prefix, CellTable};
use super::initialized_alias::RawCellAddressAliases;
use super::initialized_summary_byte_range_model::RawCellInitializationParamCount;
use super::model::{CellState, Place, ResourceLocal};

pub(super) fn collect_param_count_sources(
    cells: &CellTable,
    raw_aliases: &RawCellAddressAliases,
    count: &Place,
    params: &[ResourceLocal],
) -> Vec<RawCellInitializationParamCount> {
    let count = raw_aliases.canonicalize_scalar(count);
    let mut out = Vec::new();
    if let Some(value) = raw_aliases.i32_value(&count) {
        push_unique_param_count(
            &mut out,
            RawCellInitializationParamCount::KnownI32 {
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
        for entry_alias in raw_aliases.scalar_aliases_for_value(&entry.place) {
            for (param_index, param) in params.iter().enumerate() {
                for param_alias in raw_aliases.scalar_aliases_for_value(&param.place) {
                    let Some(suffix) =
                        place_suffix_after_address_prefix(&entry_alias, &param_alias)
                    else {
                        continue;
                    };
                    push_unique_param_count(
                        &mut out,
                        RawCellInitializationParamCount::ParamProjection {
                            param_index,
                            suffix,
                            ty: entry_alias.ty,
                        },
                    );
                }
            }
        }
    }
    out
}

fn push_unique_param_count(
    counts: &mut Vec<RawCellInitializationParamCount>,
    count: RawCellInitializationParamCount,
) {
    if !counts.iter().any(|existing| existing == &count) {
        counts.push(count);
    }
}
