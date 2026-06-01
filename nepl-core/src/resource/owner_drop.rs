extern crate alloc;

use crate::span::Span;

use super::initialized_alias::RawCellAddressAliases;
use super::model::{OwnerState, Place};
use super::owner_alias::resolve_owner_alias_place;
use super::owner_check::ResourceOwnerCheckEngine;
use super::owner_raw_view::RawAddressViewTable;
use super::owner_state::OwnerTable;
use super::owner_transfer::move_owner_state_out;
use super::raw_realloc::PendingRawReallocs;
use super::storage_origin::StorageOriginTable;

impl ResourceOwnerCheckEngine<'_> {
    pub(super) fn drop_owner_obligation(
        &mut self,
        owners: &mut OwnerTable,
        raw_aliases: &mut RawCellAddressAliases,
        raw_views: &mut RawAddressViewTable,
        storage_origins: &mut StorageOriginTable,
        pending_reallocs: &mut PendingRawReallocs,
        place: &Place,
        _span: Span,
    ) {
        self.close_owner_obligations_for_drop(owners, raw_aliases, storage_origins, place);
        raw_aliases.clear(place);
        raw_views.clear(place);
        storage_origins.clear(place);
        pending_reallocs.clear_result(place);
    }

    pub(super) fn auto_drop_scope_owner_obligations(
        &mut self,
        owners: &mut OwnerTable,
        raw_aliases: &mut RawCellAddressAliases,
        raw_views: &mut RawAddressViewTable,
        storage_origins: &mut StorageOriginTable,
        pending_reallocs: &mut PendingRawReallocs,
        locals: &[Place],
        result: Option<&Place>,
        span: Span,
    ) {
        for place in self.scope_auto_drop_owner_obligation_places(
            owners,
            raw_aliases,
            storage_origins,
            locals,
            result,
            span,
        ) {
            self.drop_owner_obligation(
                owners,
                raw_aliases,
                raw_views,
                storage_origins,
                pending_reallocs,
                &place,
                span,
            );
        }
    }

    fn close_owner_obligations_for_drop(
        &mut self,
        owners: &mut OwnerTable,
        raw_aliases: &mut RawCellAddressAliases,
        storage_origins: &mut StorageOriginTable,
        place: &Place,
    ) {
        let resolved_place = resolve_owner_alias_place(owners, raw_aliases, place);
        let mut entries = owners.live_entries_under(&resolved_place);
        if entries.is_empty() && resolved_place != *place {
            entries = owners.live_entries_under(place);
        }
        if entries.is_empty() {
            return;
        }

        for entry in entries {
            match entry.state {
                OwnerState::Live { .. } | OwnerState::MaybeFreed { .. } => {
                    move_owner_state_out(owners, raw_aliases, storage_origins, &entry.place);
                    raw_aliases.clear(&entry.place);
                }
                OwnerState::Reserved { .. } => {
                    owners.set_state(&entry.place, OwnerState::Moved);
                    raw_aliases.clear(&entry.place);
                    storage_origins.clear(&entry.place);
                }
                OwnerState::NoFreeObligation | OwnerState::Moved | OwnerState::Freed => {}
            }
        }
    }
}
