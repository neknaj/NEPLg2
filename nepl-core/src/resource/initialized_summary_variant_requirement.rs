extern crate alloc;

use alloc::vec::Vec;

use super::cell_state::raw_cell_suffix_after_address;
use super::initialized_alias::RawCellAddressAliases;
use super::initialized_summary_variant_model::RawCellInitializationVariantParamRequirement;
use super::model::{RawMemoryOp, ResourceLocal, ResourceOp};
use super::place_utils::raw_memory_cell_place;
use super::summary_projection::summary_suffix_for_params;
use super::variant_name::normalize_variant_name;

pub(super) fn collect_variant_param_required_raw_cells(
    out: &mut Vec<RawCellInitializationVariantParamRequirement>,
    variant: &str,
    path_ops: &[ResourceOp],
    raw_aliases: &RawCellAddressAliases,
    params: &[ResourceLocal],
) {
    for op in path_ops {
        let ResourceOp::RawMemory {
            operation: RawMemoryOp::Load,
            output,
            args,
            ..
        } = op
        else {
            continue;
        };
        let Some(address) = args.first() else {
            continue;
        };
        let address = raw_aliases.canonicalize(address);
        for address_alias in raw_aliases.aliases_for(&address) {
            let cell = raw_memory_cell_place(&address_alias, output.ty);
            for (param_index, param) in params.iter().enumerate() {
                for param_alias in raw_aliases.aliases_for(&param.place) {
                    let Some(suffix) = raw_cell_suffix_after_address(&cell, &param_alias) else {
                        continue;
                    };
                    let Some(suffix) = summary_suffix_for_params(params, &suffix) else {
                        continue;
                    };
                    push_unique_variant_param_requirement(
                        out,
                        RawCellInitializationVariantParamRequirement {
                            variant: normalize_variant_name(variant),
                            param_index,
                            suffix,
                            ty: output.ty,
                        },
                    );
                }
            }
        }
    }
}

fn push_unique_variant_param_requirement(
    cells: &mut Vec<RawCellInitializationVariantParamRequirement>,
    cell: RawCellInitializationVariantParamRequirement,
) {
    if !cells.iter().any(|existing| existing == &cell) {
        cells.push(cell);
    }
}
