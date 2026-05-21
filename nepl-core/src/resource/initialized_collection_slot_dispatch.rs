use super::cell_state::CellTable;
use super::collection_slot_state_table::CollectionSlotStateTable;
use super::initialized::ResourceCheckEngine;
use super::initialized_alias::RawCellAddressAliases;
use super::model::ResourceOp;
use super::raw_realloc::PendingRawReallocs;

pub(super) fn check_initialized_collection_slot_op(
    engine: &mut ResourceCheckEngine<'_>,
    cells: &mut CellTable,
    collection_slots: &mut CollectionSlotStateTable,
    raw_aliases: &mut RawCellAddressAliases,
    pending_reallocs: &mut PendingRawReallocs,
    op: &ResourceOp,
) -> bool {
    match op {
        ResourceOp::CollectionSlotLifecycle {
            target,
            event,
            span,
        } => {
            engine.apply_collection_slot_lifecycle_with_aliases(
                cells,
                collection_slots,
                raw_aliases,
                pending_reallocs,
                target,
                *event,
                *span,
            );
            true
        }
        ResourceOp::CollectionStorageRelocate {
            old_storage,
            new_storage,
            span,
        } => {
            engine.apply_collection_storage_relocate_with_aliases(
                collection_slots,
                raw_aliases,
                pending_reallocs,
                old_storage,
                new_storage,
                *span,
            );
            true
        }
        ResourceOp::CollectionSlotDropTraversal {
            storage,
            expected_ty,
            span,
        } => {
            engine.apply_local_collection_slot_drop_traversal(
                cells,
                collection_slots,
                raw_aliases,
                storage,
                *expected_ty,
                *span,
            );
            true
        }
        ResourceOp::Expr { .. }
        | ResourceOp::DeclareLocal { .. }
        | ResourceOp::Read { .. }
        | ResourceOp::Assign { .. }
        | ResourceOp::Borrow { .. }
        | ResourceOp::Move { .. }
        | ResourceOp::Drop { .. }
        | ResourceOp::EndScope { .. }
        | ResourceOp::CallEffect { .. }
        | ResourceOp::FunctionValue { .. }
        | ResourceOp::Call { .. }
        | ResourceOp::IndirectCall { .. }
        | ResourceOp::RawMemory { .. }
        | ResourceOp::RawAddressAlias { .. }
        | ResourceOp::RawAddressView { .. }
        | ResourceOp::StorageOrigin { .. }
        | ResourceOp::Construct { .. }
        | ResourceOp::Branch { .. }
        | ResourceOp::Loop { .. }
        | ResourceOp::Match { .. } => false,
    }
}
