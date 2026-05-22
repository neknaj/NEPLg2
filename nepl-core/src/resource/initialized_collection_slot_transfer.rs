extern crate alloc;

use alloc::string::ToString;

use crate::span::Span;

use super::collection_slot_owner_carrier::type_carries_collection_slot_owner;
use super::collection_slot_state_table::CollectionSlotStateTable;
use super::initialized::ResourceCheckEngine;
use super::initialized_alias::RawCellAddressAliases;
use super::model::Place;
use super::report::ResourceCheckDiagnostic;

impl ResourceCheckEngine<'_> {
    pub(super) fn transfer_slot_state_if_moved(
        &mut self,
        collection_slots: &mut CollectionSlotStateTable,
        source: &Place,
        target: &Place,
        span: Span,
    ) {
        if self.types.is_copy(source.ty)
            || !type_carries_collection_slot_owner(self.types, source.ty)
        {
            return;
        }
        self.transfer_slot_state(collection_slots, source, target, span);
    }

    pub(super) fn transfer_slot_state_if_moved_with_aliases(
        &mut self,
        collection_slots: &mut CollectionSlotStateTable,
        source: &Place,
        target: &Place,
        raw_aliases: &RawCellAddressAliases,
        span: Span,
    ) {
        if self.types.is_copy(source.ty)
            || !type_carries_collection_slot_owner(self.types, source.ty)
        {
            return;
        }
        self.transfer_slot_state_with_aliases(collection_slots, source, target, raw_aliases, span);
    }

    pub(super) fn transfer_slot_state(
        &mut self,
        collection_slots: &mut CollectionSlotStateTable,
        source: &Place,
        target: &Place,
        span: Span,
    ) {
        if !type_carries_collection_slot_owner(self.types, source.ty) {
            return;
        }
        if let Err(refutation) = collection_slots.transfer_storage_prefix(source, target) {
            self.diagnostics
                .push(ResourceCheckDiagnostic::CollectionSlotRefuted {
                    function: self.function.to_string(),
                    target: refutation.slot,
                    reason: refutation.reason,
                    span,
                });
        }
    }

    pub(super) fn transfer_slot_state_with_aliases(
        &mut self,
        collection_slots: &mut CollectionSlotStateTable,
        source: &Place,
        target: &Place,
        raw_aliases: &RawCellAddressAliases,
        span: Span,
    ) {
        if !type_carries_collection_slot_owner(self.types, source.ty) {
            return;
        }
        if let Err(refutation) =
            collection_slots.transfer_storage_prefix_with_aliases(source, target, raw_aliases)
        {
            self.diagnostics
                .push(ResourceCheckDiagnostic::CollectionSlotRefuted {
                    function: self.function.to_string(),
                    target: refutation.slot,
                    reason: refutation.reason,
                    span,
                });
        }
    }
}
