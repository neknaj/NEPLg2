use crate::span::Span;

use super::initialized_alias::RawCellAddressAliases;
use super::model::{OwnerState, OwnerStorageExtent, Place};
use super::owner_alias::resolve_owner_alias_place;
use super::owner_check::ResourceOwnerCheckEngine;
use super::owner_extent_compare::region_token_size_for_raw_owner;
use super::owner_state::OwnerTable;
use super::report::ResourceOwnerOperation;

impl ResourceOwnerCheckEngine<'_> {
    pub(super) fn ensure_owner_extent_matches_argument(
        &mut self,
        owners: &OwnerTable,
        raw_aliases: &RawCellAddressAliases,
        place: &Place,
        actual_extent: &Place,
        operation: ResourceOwnerOperation,
        _span: Span,
    ) -> bool {
        self.ensure_owner_extent_matches_expected_storage(
            owners,
            raw_aliases,
            place,
            &OwnerStorageExtent::payload_bytes(actual_extent),
            operation,
        )
    }

    pub(super) fn ensure_owner_extent_matches_summary(
        &mut self,
        owners: &OwnerTable,
        raw_aliases: &RawCellAddressAliases,
        place: &Place,
        expected_extent: &OwnerStorageExtent,
        operation: ResourceOwnerOperation,
        span: Span,
    ) -> bool {
        match expected_extent {
            OwnerStorageExtent::Unknown => true,
            OwnerStorageExtent::RegionTokenSize => {
                let resolved_place = resolve_owner_alias_place(owners, raw_aliases, place);
                let Some(size) = region_token_size_for_raw_owner(&resolved_place) else {
                    return true;
                };
                self.ensure_owner_extent_matches_argument(
                    owners,
                    raw_aliases,
                    place,
                    &size,
                    operation,
                    span,
                )
            }
            OwnerStorageExtent::PayloadBytes { bytes } => self
                .ensure_owner_extent_matches_argument(
                    owners,
                    raw_aliases,
                    place,
                    bytes,
                    operation,
                    span,
                ),
            OwnerStorageExtent::PayloadBytesScaled { .. } => self
                .ensure_owner_extent_matches_expected_storage(
                    owners,
                    raw_aliases,
                    place,
                    expected_extent,
                    operation,
                ),
        }
    }

    pub(super) fn push_extent_unavailable(
        &mut self,
        owners: &OwnerTable,
        raw_aliases: &RawCellAddressAliases,
        place: &Place,
        operation: ResourceOwnerOperation,
        span: Span,
    ) {
        let resolved_place = resolve_owner_alias_place(owners, raw_aliases, place);
        let state = owners
            .state(&resolved_place)
            .unwrap_or(OwnerState::NoFreeObligation);
        self.push_unavailable(operation, &resolved_place, state, span);
    }
}
