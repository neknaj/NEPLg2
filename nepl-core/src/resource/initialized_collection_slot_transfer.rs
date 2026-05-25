extern crate alloc;

use alloc::string::ToString;

use crate::span::Span;

use super::cell_state::CellTable;
use super::collection_slot_state_table::CollectionSlotStateTable;
use super::collection_slot_storage_carrier::type_can_carry_collection_slot_storage;
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
            || !type_can_carry_collection_slot_storage(self.types, source.ty)
        {
            return;
        }
        self.transfer_slot_state(collection_slots, source, target, span);
    }

    pub(super) fn transfer_slot_state_if_moved_with_aliases(
        &mut self,
        cells: &CellTable,
        collection_slots: &mut CollectionSlotStateTable,
        source: &Place,
        target: &Place,
        raw_aliases: &RawCellAddressAliases,
        span: Span,
    ) {
        if self.types.is_copy(source.ty)
            || !type_can_carry_collection_slot_storage(self.types, source.ty)
        {
            return;
        }
        // raw memory から load された payload は、後続の drop traversal に対して
        // 「その raw cell の値が drop された」という証明を運ぶ値であり、
        // collection storage 自体の所有者ではない。ここで slot state を local へ
        // 移すと、storage 側の traversal 証明と local 上書き時の vacancy 検査が
        // 同じ slot を別々の所有者として扱ってしまう。
        if cells.value_has_raw_cell_loaded_origin(source, self.types)
            || cells.value_has_raw_cell_loaded_origin(target, self.types)
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
        if !type_can_carry_collection_slot_storage(self.types, source.ty) {
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
        if !type_can_carry_collection_slot_storage(self.types, source.ty) {
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
