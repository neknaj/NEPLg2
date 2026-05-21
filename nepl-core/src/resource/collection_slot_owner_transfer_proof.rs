use crate::types::{TypeCtx, TypeId};

use super::cell_state::CellTable;
use super::collection_slot_owner_transfer::CollectionSlotOwnerTransferObligation;
use super::initialized_alias::RawCellAddressAliases;
use super::model::Place;
use super::raw_cell_value_flow::RawCellValueFlowKind;
use super::raw_cell_value_flow_proof::{
    consume_raw_cell_value_flow, raw_cell_value_flow_available,
};
use super::type_pattern::type_pattern_matches;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum CollectionSlotOwnerTransferProof {
    LocalRawValueFlow,
    SummaryStateOnly,
    SummaryCertified(CollectionSlotOwnerTransferObligation),
}

pub(super) fn consume_collection_slot_owner_transfer_proof(
    cells: &mut CellTable,
    raw_aliases: Option<&RawCellAddressAliases>,
    target: &Place,
    obligation: CollectionSlotOwnerTransferObligation,
    proof: CollectionSlotOwnerTransferProof,
    types: &TypeCtx,
) -> bool {
    match proof {
        CollectionSlotOwnerTransferProof::LocalRawValueFlow => {
            consume_local_raw_value_flow_proof(cells, raw_aliases, target, obligation, types)
        }
        CollectionSlotOwnerTransferProof::SummaryStateOnly => false,
        CollectionSlotOwnerTransferProof::SummaryCertified(certified) => {
            owner_transfer_obligation_matches(types, certified, obligation)
        }
    }
}

pub(super) fn collection_slot_owner_transfer_proof_satisfied(
    cells: &CellTable,
    raw_aliases: Option<&RawCellAddressAliases>,
    target: &Place,
    obligation: CollectionSlotOwnerTransferObligation,
    proof: CollectionSlotOwnerTransferProof,
    types: &TypeCtx,
) -> bool {
    match proof {
        CollectionSlotOwnerTransferProof::LocalRawValueFlow => {
            collection_slot_owner_transfer_proof_available(
                cells,
                raw_aliases,
                target,
                obligation,
                types,
            )
        }
        CollectionSlotOwnerTransferProof::SummaryStateOnly => false,
        CollectionSlotOwnerTransferProof::SummaryCertified(certified) => {
            owner_transfer_obligation_matches(types, certified, obligation)
        }
    }
}

pub(super) fn collection_slot_owner_transfer_proof_available(
    cells: &CellTable,
    raw_aliases: Option<&RawCellAddressAliases>,
    target: &Place,
    obligation: CollectionSlotOwnerTransferObligation,
    types: &TypeCtx,
) -> bool {
    match obligation {
        CollectionSlotOwnerTransferObligation::StoreValue {
            operation: _,
            value_ty,
        } => raw_cell_value_flow_available(
            cells,
            raw_aliases,
            target,
            value_ty,
            RawCellValueFlowKind::StoreValue,
            types,
        ),
        CollectionSlotOwnerTransferObligation::MoveOutValue {
            operation: _,
            value_ty,
        } => raw_cell_value_flow_available(
            cells,
            raw_aliases,
            target,
            value_ty,
            RawCellValueFlowKind::MoveOutLoadedCell,
            types,
        ),
        CollectionSlotOwnerTransferObligation::MoveOutAndStoreValue {
            operation: _,
            old_ty,
            new_ty,
        } => {
            raw_cell_value_flow_available(
                cells,
                raw_aliases,
                target,
                old_ty,
                RawCellValueFlowKind::MoveOutLoadedCell,
                types,
            ) && raw_cell_value_flow_available(
                cells,
                raw_aliases,
                target,
                new_ty,
                RawCellValueFlowKind::StoreValue,
                types,
            )
        }
    }
}

fn consume_local_raw_value_flow_proof(
    cells: &mut CellTable,
    raw_aliases: Option<&RawCellAddressAliases>,
    target: &Place,
    obligation: CollectionSlotOwnerTransferObligation,
    types: &TypeCtx,
) -> bool {
    if !collection_slot_owner_transfer_proof_available(
        cells,
        raw_aliases,
        target,
        obligation,
        types,
    ) {
        return false;
    }
    match obligation {
        CollectionSlotOwnerTransferObligation::StoreValue {
            operation: _,
            value_ty,
        } => consume_raw_cell_value_flow(
            cells,
            raw_aliases,
            target,
            value_ty,
            RawCellValueFlowKind::StoreValue,
            types,
        ),
        CollectionSlotOwnerTransferObligation::MoveOutValue {
            operation: _,
            value_ty,
        } => consume_raw_cell_value_flow(
            cells,
            raw_aliases,
            target,
            value_ty,
            RawCellValueFlowKind::MoveOutLoadedCell,
            types,
        ),
        CollectionSlotOwnerTransferObligation::MoveOutAndStoreValue {
            operation: _,
            old_ty,
            new_ty,
        } => {
            let mut committed = cells.clone();
            if consume_raw_cell_value_flow(
                &mut committed,
                raw_aliases,
                target,
                old_ty,
                RawCellValueFlowKind::MoveOutLoadedCell,
                types,
            ) && consume_raw_cell_value_flow(
                &mut committed,
                raw_aliases,
                target,
                new_ty,
                RawCellValueFlowKind::StoreValue,
                types,
            ) {
                *cells = committed;
                true
            } else {
                false
            }
        }
    }
}

fn owner_transfer_obligation_matches(
    types: &TypeCtx,
    certified: CollectionSlotOwnerTransferObligation,
    required: CollectionSlotOwnerTransferObligation,
) -> bool {
    match (certified, required) {
        (
            CollectionSlotOwnerTransferObligation::StoreValue {
                operation: certified_op,
                value_ty: certified_ty,
            },
            CollectionSlotOwnerTransferObligation::StoreValue {
                operation: required_op,
                value_ty: required_ty,
            },
        )
        | (
            CollectionSlotOwnerTransferObligation::MoveOutValue {
                operation: certified_op,
                value_ty: certified_ty,
            },
            CollectionSlotOwnerTransferObligation::MoveOutValue {
                operation: required_op,
                value_ty: required_ty,
            },
        ) => {
            certified_op == required_op
                && owner_transfer_type_matches(types, certified_ty, required_ty)
        }
        (
            CollectionSlotOwnerTransferObligation::MoveOutAndStoreValue {
                operation: certified_op,
                old_ty: certified_old,
                new_ty: certified_new,
            },
            CollectionSlotOwnerTransferObligation::MoveOutAndStoreValue {
                operation: required_op,
                old_ty: required_old,
                new_ty: required_new,
            },
        ) => {
            certified_op == required_op
                && owner_transfer_type_matches(types, certified_old, required_old)
                && owner_transfer_type_matches(types, certified_new, required_new)
        }
        _ => false,
    }
}

fn owner_transfer_type_matches(types: &TypeCtx, left: TypeId, right: TypeId) -> bool {
    type_pattern_matches(types, left, right) || type_pattern_matches(types, right, left)
}
