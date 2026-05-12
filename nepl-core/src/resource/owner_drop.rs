extern crate alloc;

use crate::span::Span;

use super::initialized_alias::RawCellAddressAliases;
use super::model::Place;
use super::owner_check::ResourceOwnerCheckEngine;
use super::owner_raw_view::RawAddressViewTable;
use super::owner_state::OwnerTable;
use super::raw_realloc::PendingRawReallocs;
use super::report::ResourceOwnerOperation;
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
        span: Span,
    ) {
        if self.has_transferable_owner(owners, raw_aliases, place) {
            self.move_owner_out(
                owners,
                raw_aliases,
                storage_origins,
                place,
                ResourceOwnerOperation::Drop,
                span,
            );
        }
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
}
