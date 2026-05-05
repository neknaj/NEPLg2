extern crate alloc;

use alloc::vec::Vec;

use crate::types::{TypeCtx, TypeId};

use super::cell_state::raw_address_suffix_after_address;
use super::cell_state::CellTable;
use super::initialized_alias::RawCellAddressAliases;
use super::initialized_summary::{
    RawCellDestructionParamAddress, RawCellInitializationFunctionSummary, RawCellMoveParamAddress,
};
use super::model::{
    CellState, Place, PlaceProjection, RawMemoryOp, ResourceCallTarget, ResourceLocal,
    ResourceOffset,
};
use super::place_utils::{place_with_suffix, raw_memory_cell_place};
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

pub(super) fn collect_param_moves_from_raw_memory(
    out: &mut Vec<RawCellMoveParamAddress>,
    cells: &CellTable,
    raw_aliases: &RawCellAddressAliases,
    params: &[ResourceLocal],
    types: &TypeCtx,
    operation: &RawMemoryOp,
    output: &Place,
    args: &[Place],
) {
    match operation {
        RawMemoryOp::Load if !types.is_copy(output.ty) => {
            collect_param_moves_for_arg(out, cells, raw_aliases, params, types, args, 0, output.ty)
        }
        RawMemoryOp::Alloc
        | RawMemoryOp::Dealloc
        | RawMemoryOp::Realloc
        | RawMemoryOp::Store
        | RawMemoryOp::BulkCopy
        | RawMemoryOp::BulkMove
        | RawMemoryOp::MemorySize
        | RawMemoryOp::MemoryGrow
        | RawMemoryOp::Fill
        | RawMemoryOp::Load
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

pub(super) fn collect_param_moves_from_direct_call(
    out: &mut Vec<RawCellMoveParamAddress>,
    cells: &CellTable,
    raw_aliases: &RawCellAddressAliases,
    params: &[ResourceLocal],
    types: &TypeCtx,
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
    collect_param_moves_from_summary(out, cells, raw_aliases, params, types, args, summary);
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

pub(super) fn collect_param_moves_from_summary(
    out: &mut Vec<RawCellMoveParamAddress>,
    cells: &CellTable,
    raw_aliases: &RawCellAddressAliases,
    params: &[ResourceLocal],
    types: &TypeCtx,
    args: &[Place],
    summary: &RawCellInitializationFunctionSummary,
) {
    for moved in &summary.param_moves {
        let Some(arg) = args.get(moved.param_index) else {
            continue;
        };
        let address = place_with_suffix(arg, &moved.suffix, moved.address_ty);
        let address = raw_aliases.canonicalize(&address);
        collect_param_moves_for_address(
            out,
            cells,
            raw_aliases,
            params,
            types,
            &address,
            moved.cell_ty,
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

fn collect_param_moves_for_arg(
    out: &mut Vec<RawCellMoveParamAddress>,
    cells: &CellTable,
    raw_aliases: &RawCellAddressAliases,
    params: &[ResourceLocal],
    types: &TypeCtx,
    args: &[Place],
    arg_index: usize,
    cell_ty: TypeId,
) {
    let Some(address) = args.get(arg_index) else {
        return;
    };
    collect_param_moves_for_address(out, cells, raw_aliases, params, types, address, cell_ty);
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

fn collect_param_moves_for_address(
    out: &mut Vec<RawCellMoveParamAddress>,
    cells: &CellTable,
    raw_aliases: &RawCellAddressAliases,
    params: &[ResourceLocal],
    types: &TypeCtx,
    address: &Place,
    cell_ty: TypeId,
) {
    let address = raw_aliases.canonicalize(address);
    for address_alias in raw_aliases.aliases_for(&address) {
        if address_has_unknown_offset(&address_alias) {
            continue;
        }
        let cell = raw_memory_cell_place(&address_alias, cell_ty);
        if matches!(
            cells.availability_state(&cell, types),
            CellState::Initialized(_)
        ) {
            continue;
        }
        for (param_index, param) in params.iter().enumerate() {
            for param_alias in raw_aliases.aliases_for(&param.place) {
                let Some(suffix) = raw_address_suffix_after_address(&address_alias, &param_alias)
                else {
                    continue;
                };
                push_unique_param_move(
                    out,
                    RawCellMoveParamAddress {
                        param_index,
                        suffix,
                        address_ty: address_alias.ty,
                        cell_ty,
                        operation: ResourceCheckOperation::RawMemoryLoadCell,
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

fn push_unique_param_move(cells: &mut Vec<RawCellMoveParamAddress>, cell: RawCellMoveParamAddress) {
    if !cells.iter().any(|existing| existing == &cell) {
        cells.push(cell);
    }
}

fn address_has_unknown_offset(place: &Place) -> bool {
    place.projections.iter().any(|projection| {
        matches!(
            projection,
            PlaceProjection::StorageOffset(ResourceOffset { bytes: None })
        )
    })
}
