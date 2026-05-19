use crate::span::Span;
use crate::types::{TypeId, TypeKind};

use super::cell_state::CellTable;
use super::initialized::ResourceCheckEngine;
use super::initialized_alias::RawCellAddressAliases;
use super::model::Place;
use super::place_utils::raw_memory_cell_place;
use super::report::ResourceCheckOperation;

impl ResourceCheckEngine<'_> {
    pub(super) fn check_raw_memory_load(
        &mut self,
        cells: &mut CellTable,
        raw_aliases: &mut RawCellAddressAliases,
        output: &Place,
        args: &[Place],
        cell_ty: TypeId,
        span: Span,
    ) {
        let Some(address) = args.first() else {
            cells.mark_initialized(output);
            raw_aliases.clear(output);
            return;
        };
        let address = raw_aliases.canonicalize(address);
        let address_available = self.ensure_available(
            cells,
            &address,
            ResourceCheckOperation::RawMemoryLoadAddress,
            span,
        );
        let cell = raw_memory_cell_place(&address, cell_ty);
        let loaded_from_untracked_external = raw_aliases
            .aliases_for(&address)
            .iter()
            .any(|alias| cells.raw_cell_is_untracked_external(alias));
        let loaded_from_zero_initialized_runtime =
            raw_memory_load_reads_zero_initialized_runtime_cell(cells, raw_aliases, &address);
        let loaded_from_untracked_source =
            loaded_from_untracked_external || loaded_from_zero_initialized_runtime;
        let cell_available = loaded_from_untracked_source
            || cells.raw_cell_initialized_by_byte_range(&address, cell_ty, raw_aliases, self.types)
            || self.ensure_available(
                cells,
                &cell,
                ResourceCheckOperation::RawMemoryLoadCell,
                span,
            );
        if address_available && cell_available {
            if !self.types.is_copy(cell_ty) {
                cells.mark_raw_cell_moved(&address, cell_ty);
            }
            cells.mark_initialized(output);
            if raw_aliases.value_is_known_raw_address(&cell) {
                self.copy_raw_alias_and_rekey_cells_preferring_target(
                    cells,
                    raw_aliases,
                    &cell,
                    output,
                );
            } else if loaded_from_untracked_source && self.output_can_hold_raw_address(output.ty) {
                cells.mark_external_raw_storage_root(output);
                raw_aliases.mark(output);
            } else {
                raw_aliases.copy_alias_if_tracked(&cell, output);
            }
            cells.copy_initialized_raw_byte_ranges_through_value_aliases(
                &cell,
                output,
                raw_aliases,
            );
        }
    }

    pub(super) fn check_raw_memory_store(
        &mut self,
        cells: &mut CellTable,
        raw_aliases: &mut RawCellAddressAliases,
        output: &Place,
        args: &[Place],
        cell_ty: Option<TypeId>,
        span: Span,
    ) {
        let Some(address) = args.first() else {
            cells.mark_initialized(output);
            raw_aliases.clear(output);
            return;
        };
        let address = raw_aliases.canonicalize(address);
        let address_available = self.ensure_available(
            cells,
            &address,
            ResourceCheckOperation::RawMemoryStoreAddress,
            span,
        );
        let cell_available = self.ensure_no_live_non_copy_raw_cells(
            cells,
            &address,
            ResourceCheckOperation::RawMemoryStoreCell,
            span,
        );
        let value_available = if address_available && cell_available {
            args.get(1).is_none_or(|value| {
                self.consume_by_value(
                    cells,
                    value,
                    ResourceCheckOperation::RawMemoryStoreValue,
                    span,
                )
            })
        } else {
            false
        };
        if address_available && cell_available && value_available {
            if let Some(value) = args.get(1) {
                let stored_ty = cell_ty.unwrap_or(value.ty);
                let cell = raw_memory_cell_place(&address, stored_ty);
                cells.clear_raw_cells_overwritten_by_store(&address, stored_ty, self.types);
                cells.clear_initialized_raw_byte_ranges_through_value(&cell);
                cells.mark_initialized(&cell);
                raw_aliases.clear(&cell);
                raw_aliases.copy_alias_if_tracked(value, &cell);
                cells.copy_initialized_raw_byte_ranges_through_value_aliases(
                    value,
                    &cell,
                    raw_aliases,
                );
            }
            cells.mark_initialized(output);
            raw_aliases.clear(output);
        }
    }

    fn output_can_hold_raw_address(&self, ty: TypeId) -> bool {
        matches!(self.types.get_ref(self.types.resolve_id(ty)), TypeKind::I32)
    }
}

pub(super) fn raw_memory_load_reads_zero_initialized_runtime_cell(
    cells: &CellTable,
    raw_aliases: &RawCellAddressAliases,
    address: &Place,
) -> bool {
    let aliases = raw_aliases.aliases_for(address);
    aliases
        .iter()
        .any(|alias| raw_aliases.i32_value(alias).is_some_and(|value| value >= 0))
        && aliases
            .iter()
            .all(|alias| !cells.raw_address_has_tracked_storage(alias))
}
