extern crate alloc;

use alloc::vec::Vec;

use super::cell_state::raw_address_suffix_after_address;
use super::initialized_alias::RawCellAddressAliases;
use super::initialized_summary::{
    RawCellDestructionParamAddress, RawCellInitializationFunctionSummary,
};
use super::model::{Place, RawMemoryOp, ResourceCallTarget, ResourceLocal};
use super::place_utils::place_with_suffix;
use super::report::ResourceCheckOperation;

pub(super) fn collect_param_destructions_from_raw_memory(
    out: &mut Vec<RawCellDestructionParamAddress>,
    raw_aliases: &RawCellAddressAliases,
    params: &[ResourceLocal],
    operation: &RawMemoryOp,
    args: &[Place],
) {
    match operation {
        RawMemoryOp::Store => collect_param_destructions_for_arg(
            out,
            raw_aliases,
            params,
            args,
            0,
            ResourceCheckOperation::RawMemoryStoreCell,
        ),
        RawMemoryOp::Dealloc => collect_param_destructions_for_arg(
            out,
            raw_aliases,
            params,
            args,
            0,
            ResourceCheckOperation::RawMemoryDeallocCell,
        ),
        RawMemoryOp::Realloc => collect_param_destructions_for_arg(
            out,
            raw_aliases,
            params,
            args,
            0,
            ResourceCheckOperation::RawMemoryReallocCell,
        ),
        RawMemoryOp::Fill => collect_param_destructions_for_arg(
            out,
            raw_aliases,
            params,
            args,
            0,
            ResourceCheckOperation::RawMemoryFillCell,
        ),
        RawMemoryOp::BulkCopy | RawMemoryOp::BulkMove => {
            collect_param_destructions_for_arg(
                out,
                raw_aliases,
                params,
                args,
                0,
                ResourceCheckOperation::RawMemoryBulkDestinationCell,
            );
            collect_param_destructions_for_arg(
                out,
                raw_aliases,
                params,
                args,
                1,
                ResourceCheckOperation::RawMemoryBulkSourceCell,
            );
        }
        RawMemoryOp::Alloc
        | RawMemoryOp::Load
        | RawMemoryOp::MemorySize
        | RawMemoryOp::MemoryGrow
        | RawMemoryOp::Other { .. } => {}
    }
}

pub(super) fn collect_param_destructions_from_direct_call(
    out: &mut Vec<RawCellDestructionParamAddress>,
    raw_aliases: &RawCellAddressAliases,
    params: &[ResourceLocal],
    target: &ResourceCallTarget,
    args: &[Place],
    raw_init_summaries: &[RawCellInitializationFunctionSummary],
) {
    let ResourceCallTarget::User { name, .. } = target else {
        return;
    };
    let Some(summary) = raw_init_summaries
        .iter()
        .find(|summary| summary.function == name.as_str())
    else {
        return;
    };
    collect_param_destructions_from_summary(out, raw_aliases, params, args, summary);
}

pub(super) fn collect_param_destructions_from_summary(
    out: &mut Vec<RawCellDestructionParamAddress>,
    raw_aliases: &RawCellAddressAliases,
    params: &[ResourceLocal],
    args: &[Place],
    summary: &RawCellInitializationFunctionSummary,
) {
    for destruction in &summary.param_destructions {
        let Some(arg) = args.get(destruction.param_index) else {
            continue;
        };
        let address = place_with_suffix(arg, &destruction.suffix, destruction.ty);
        let address = raw_aliases.canonicalize(&address);
        collect_param_destructions_for_address(
            out,
            raw_aliases,
            params,
            &address,
            destruction.operation,
        );
    }
}

fn collect_param_destructions_for_arg(
    out: &mut Vec<RawCellDestructionParamAddress>,
    raw_aliases: &RawCellAddressAliases,
    params: &[ResourceLocal],
    args: &[Place],
    arg_index: usize,
    operation: ResourceCheckOperation,
) {
    let Some(address) = args.get(arg_index) else {
        return;
    };
    collect_param_destructions_for_address(out, raw_aliases, params, address, operation);
}

fn collect_param_destructions_for_address(
    out: &mut Vec<RawCellDestructionParamAddress>,
    raw_aliases: &RawCellAddressAliases,
    params: &[ResourceLocal],
    address: &Place,
    operation: ResourceCheckOperation,
) {
    let address = raw_aliases.canonicalize(address);
    for address_alias in raw_aliases.aliases_for(&address) {
        for (param_index, param) in params.iter().enumerate() {
            for param_alias in raw_aliases.aliases_for(&param.place) {
                let Some(suffix) = raw_address_suffix_after_address(&address_alias, &param_alias)
                else {
                    continue;
                };
                push_unique_param_destruction(
                    out,
                    RawCellDestructionParamAddress {
                        param_index,
                        suffix,
                        ty: address_alias.ty,
                        operation,
                    },
                );
            }
        }
    }
}

fn push_unique_param_destruction(
    cells: &mut Vec<RawCellDestructionParamAddress>,
    cell: RawCellDestructionParamAddress,
) {
    if !cells.iter().any(|existing| existing == &cell) {
        cells.push(cell);
    }
}
