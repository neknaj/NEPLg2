use super::initialized_alias::RawCellAddressAliases;
use super::model::{OwnerStorageExtent, Place};
use super::owner_alias::resolve_owner_alias_place;
use super::owner_check::ResourceOwnerCheckEngine;
use super::owner_extent::{
    prove_owner_extent_matches_storage, OwnerExtentProof, PendingOwnerExtentRequirement,
};
use super::owner_extent_compare::comparable_owner_extent;
use super::owner_state::OwnerTable;
use super::report::ResourceOwnerOperation;

impl ResourceOwnerCheckEngine<'_> {
    pub(super) fn ensure_owner_extent_matches_expected_storage(
        &mut self,
        owners: &OwnerTable,
        raw_aliases: &RawCellAddressAliases,
        place: &Place,
        expected_extent: &OwnerStorageExtent,
        operation: ResourceOwnerOperation,
    ) -> bool {
        let resolved_place = resolve_owner_alias_place(owners, raw_aliases, place);
        let extent = owners
            .live_extent(&resolved_place)
            .unwrap_or(OwnerStorageExtent::Unknown);
        let extent = comparable_owner_extent(&resolved_place, extent);
        match prove_owner_extent_matches_storage(raw_aliases, &extent, expected_extent) {
            OwnerExtentProof::Proven => true,
            OwnerExtentProof::Unknown => {
                self.owner_extent_requirements
                    .push(PendingOwnerExtentRequirement {
                        owner: resolved_place,
                        expected: expected_extent.clone(),
                        operation,
                    });
                true
            }
            OwnerExtentProof::Mismatch => false,
        }
    }
}
