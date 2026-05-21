use crate::types::{TypeCtx, TypeId};

use super::cell_state::CellTable;
use super::collection_slot_lifecycle::{
    CollectionSlotLifecycleEvent, CollectionSlotLifecycleOp, CollectionSlotReplacement,
};
use super::drop_requirement::resource_type_needs_drop_code;
use super::model::Place;
use super::raw_cell_value_flow::RawCellValueFlowKind;
use super::type_pattern::type_pattern_matches;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum CollectionSlotDropObligation {
    DropLoadedValue {
        operation: CollectionSlotLifecycleOp,
        value_ty: TypeId,
    },
}

impl CollectionSlotDropObligation {
    pub(super) fn primary_refutation(self) -> (CollectionSlotLifecycleOp, TypeId) {
        match self {
            CollectionSlotDropObligation::DropLoadedValue {
                operation,
                value_ty,
            } => (operation, value_ty),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum CollectionSlotDropProof {
    LocalLoadedValueDrop,
    SummaryStateOnly,
    SummaryCertified(CollectionSlotDropObligation),
}

pub(super) fn collection_slot_drop_obligation(
    types: &TypeCtx,
    event: CollectionSlotLifecycleEvent,
) -> Option<CollectionSlotDropObligation> {
    let (operation, value_ty) = match event {
        CollectionSlotLifecycleEvent::DropInitialized { expected_ty } => {
            (CollectionSlotLifecycleOp::DropInitialized, expected_ty)
        }
        CollectionSlotLifecycleEvent::ReplaceInitialized {
            old_ty,
            new_ty: _,
            old_owner: CollectionSlotReplacement::DropOldOwner,
        } => (CollectionSlotLifecycleOp::ReplaceInitialized, old_ty),
        CollectionSlotLifecycleEvent::ReplaceInitialized {
            old_ty: _,
            new_ty: _,
            old_owner: CollectionSlotReplacement::ReturnOldOwner,
        }
        | CollectionSlotLifecycleEvent::InitializeEmpty { .. }
        | CollectionSlotLifecycleEvent::BorrowRead { .. }
        | CollectionSlotLifecycleEvent::MoveOut { .. }
        | CollectionSlotLifecycleEvent::StorageDealloc => return None,
    };
    resource_type_needs_drop_code(types, value_ty).then_some(
        CollectionSlotDropObligation::DropLoadedValue {
            operation,
            value_ty,
        },
    )
}

pub(super) fn collection_slot_drop_proof_available(
    cells: &CellTable,
    target: &Place,
    obligation: CollectionSlotDropObligation,
    types: &TypeCtx,
) -> bool {
    match obligation {
        CollectionSlotDropObligation::DropLoadedValue {
            operation: _,
            value_ty,
        } => cells.raw_cell_value_flow_available(
            target,
            value_ty,
            RawCellValueFlowKind::DropLoadedCell,
            types,
        ),
    }
}

pub(super) fn consume_collection_slot_drop_proof(
    cells: &mut CellTable,
    target: &Place,
    obligation: CollectionSlotDropObligation,
    proof: CollectionSlotDropProof,
    types: &TypeCtx,
) -> bool {
    match proof {
        CollectionSlotDropProof::LocalLoadedValueDrop => {
            consume_local_loaded_value_drop_proof(cells, target, obligation, types)
        }
        CollectionSlotDropProof::SummaryStateOnly => false,
        CollectionSlotDropProof::SummaryCertified(certified) => {
            drop_obligation_matches(types, certified, obligation)
        }
    }
}

fn consume_local_loaded_value_drop_proof(
    cells: &mut CellTable,
    target: &Place,
    obligation: CollectionSlotDropObligation,
    types: &TypeCtx,
) -> bool {
    if !collection_slot_drop_proof_available(cells, target, obligation, types) {
        return false;
    }
    match obligation {
        CollectionSlotDropObligation::DropLoadedValue {
            operation: _,
            value_ty,
        } => cells.consume_raw_cell_value_flow(
            target,
            value_ty,
            RawCellValueFlowKind::DropLoadedCell,
            types,
        ),
    }
}

fn drop_obligation_matches(
    types: &TypeCtx,
    certified: CollectionSlotDropObligation,
    required: CollectionSlotDropObligation,
) -> bool {
    match (certified, required) {
        (
            CollectionSlotDropObligation::DropLoadedValue {
                operation: certified_op,
                value_ty: certified_ty,
            },
            CollectionSlotDropObligation::DropLoadedValue {
                operation: required_op,
                value_ty: required_ty,
            },
        ) => {
            certified_op == required_op
                && (type_pattern_matches(types, certified_ty, required_ty)
                    || type_pattern_matches(types, required_ty, certified_ty))
        }
    }
}
