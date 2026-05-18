use crate::span::Span;

use super::initialized_alias::RawCellAddressAliases;
use super::model::{OwnerState, Place};
use super::owner_alias::resolve_owner_alias_place;
use super::owner_check::ResourceOwnerCheckEngine;
use super::owner_raw_view::RawAddressViewTable;
use super::owner_state::OwnerTable;
use super::owner_transfer::free_owner_state;
use super::place_utils::should_track;
use super::report::ResourceOwnerOperation;
use super::storage_origin::StorageOriginTable;

impl ResourceOwnerCheckEngine<'_> {
    pub(super) fn release_owner(
        &mut self,
        owners: &mut OwnerTable,
        raw_aliases: &mut RawCellAddressAliases,
        raw_views: &RawAddressViewTable,
        storage_origins: &mut StorageOriginTable,
        place: &Place,
        operation: ResourceOwnerOperation,
        span: Span,
    ) -> bool {
        if !should_track(place) {
            return false;
        }
        if self.place_is_non_owning_raw_address_view(owners, raw_aliases, raw_views, place) {
            self.push_unavailable(operation, place, OwnerState::NoFreeObligation, span);
            return false;
        }
        let resolved_place = resolve_owner_alias_place(owners, raw_aliases, place);
        match owners.state(&resolved_place) {
            Some(OwnerState::Live { .. }) => {
                free_owner_state(owners, raw_aliases, storage_origins, &resolved_place);
                true
            }
            Some(state) => {
                self.push_unavailable(operation, &resolved_place, state, span);
                false
            }
            None => {
                if self.storage_origin_expects_owned(storage_origins, raw_aliases, place) {
                    self.push_unavailable(operation, place, OwnerState::NoFreeObligation, span);
                }
                false
            }
        }
    }

    pub(super) fn release_owner_with_extent(
        &mut self,
        owners: &mut OwnerTable,
        raw_aliases: &mut RawCellAddressAliases,
        raw_views: &RawAddressViewTable,
        storage_origins: &mut StorageOriginTable,
        place: &Place,
        expected_extent: &Place,
        operation: ResourceOwnerOperation,
        extent_operation: ResourceOwnerOperation,
        span: Span,
    ) -> bool {
        if !self.ensure_owner_extent_matches_argument(
            owners,
            raw_aliases,
            place,
            expected_extent,
            extent_operation,
            span,
        ) {
            self.push_extent_unavailable(owners, raw_aliases, place, extent_operation, span);
            return false;
        }
        self.release_owner(
            owners,
            raw_aliases,
            raw_views,
            storage_origins,
            place,
            operation,
            span,
        )
    }

    pub(super) fn ensure_owner_available_with_extent(
        &mut self,
        owners: &OwnerTable,
        raw_aliases: &RawCellAddressAliases,
        raw_views: &RawAddressViewTable,
        storage_origins: &StorageOriginTable,
        place: &Place,
        expected_extent: &Place,
        operation: ResourceOwnerOperation,
        extent_operation: ResourceOwnerOperation,
        span: Span,
    ) -> bool {
        if !self.ensure_owner_extent_matches_argument(
            owners,
            raw_aliases,
            place,
            expected_extent,
            extent_operation,
            span,
        ) {
            self.push_extent_unavailable(owners, raw_aliases, place, extent_operation, span);
            return false;
        }
        self.ensure_owner_available(
            owners,
            raw_aliases,
            raw_views,
            storage_origins,
            place,
            operation,
            span,
        )
    }

    pub(super) fn ensure_owner_available(
        &mut self,
        owners: &OwnerTable,
        raw_aliases: &RawCellAddressAliases,
        raw_views: &RawAddressViewTable,
        storage_origins: &StorageOriginTable,
        place: &Place,
        operation: ResourceOwnerOperation,
        span: Span,
    ) -> bool {
        if !should_track(place) {
            return false;
        }
        if self.place_is_non_owning_raw_address_view(owners, raw_aliases, raw_views, place) {
            self.push_unavailable(operation, place, OwnerState::NoFreeObligation, span);
            return false;
        }
        let resolved_place = resolve_owner_alias_place(owners, raw_aliases, place);
        match owners.state(&resolved_place) {
            Some(OwnerState::Live { .. }) => true,
            Some(state) => {
                self.push_unavailable(operation, &resolved_place, state, span);
                false
            }
            None => {
                if self.storage_origin_expects_owned(storage_origins, raw_aliases, place) {
                    self.push_unavailable(operation, place, OwnerState::NoFreeObligation, span);
                }
                false
            }
        }
    }

    pub(super) fn storage_origin_expects_owned(
        &self,
        storage_origins: &StorageOriginTable,
        raw_aliases: &RawCellAddressAliases,
        place: &Place,
    ) -> bool {
        storage_origins.expects_owned(place)
            || storage_origins.expects_owned_under(place)
            || raw_aliases.aliases_for(place).iter().any(|alias| {
                storage_origins.expects_owned(alias) || storage_origins.expects_owned_under(alias)
            })
    }
}
