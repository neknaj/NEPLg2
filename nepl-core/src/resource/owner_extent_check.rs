use crate::span::Span;

use super::initialized_alias::RawCellAddressAliases;
use super::model::{OwnerState, OwnerStorageExtent, Place};
use super::owner_alias::resolve_owner_alias_place;
use super::owner_check::ResourceOwnerCheckEngine;
use super::owner_extent::{
    prove_owner_extent_matches_argument, OwnerExtentProof, PendingOwnerExtentRequirement,
};
use super::owner_state::OwnerTable;
use super::place_utils::region_token_size_field_for_raw_owner;
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
        let resolved_place = resolve_owner_alias_place(owners, raw_aliases, place);
        let extent = owners
            .live_extent(&resolved_place)
            .unwrap_or(OwnerStorageExtent::Unknown);
        let extent = comparable_owner_extent(&resolved_place, extent);
        match prove_owner_extent_matches_argument(raw_aliases, &extent, actual_extent) {
            OwnerExtentProof::Proven => true,
            OwnerExtentProof::Unknown => {
                self.owner_extent_requirements
                    .push(PendingOwnerExtentRequirement {
                        owner: resolved_place,
                        expected: OwnerStorageExtent::payload_bytes(actual_extent),
                        operation,
                    });
                true
            }
            OwnerExtentProof::Mismatch => false,
        }
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
                let Some(size) = region_token_size_field_for_raw_owner(&resolved_place) else {
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

fn comparable_owner_extent(owner: &Place, extent: OwnerStorageExtent) -> OwnerStorageExtent {
    match extent {
        OwnerStorageExtent::RegionTokenSize => region_token_size_field_for_raw_owner(owner)
            .map(|size| OwnerStorageExtent::payload_bytes(&size))
            .unwrap_or(OwnerStorageExtent::Unknown),
        other => other,
    }
}
