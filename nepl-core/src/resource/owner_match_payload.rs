use super::initialized_alias::RawCellAddressAliases;
use super::model::{OwnerState, Place};
use super::owner_alias::resolve_owner_alias_place;
use super::owner_raw_view::RawAddressViewTable;
use super::owner_state::OwnerTable;
use super::storage_origin::StorageOriginTable;

/// Retires owner obligations for enum payloads excluded by a selected match arm.
pub(super) fn retire_inactive_enum_payload_owners(
    owners: &mut OwnerTable,
    raw_aliases: &mut RawCellAddressAliases,
    raw_views: &mut RawAddressViewTable,
    storage_origins: &mut StorageOriginTable,
    scrutinee: &Place,
    selected_variant: &str,
) {
    let mut inactive_payloads = owners.sibling_enum_payload_places(scrutinee, selected_variant);
    let resolved_scrutinee = resolve_owner_alias_place(owners, raw_aliases, scrutinee);
    if resolved_scrutinee != *scrutinee {
        for inactive_payload in
            owners.sibling_enum_payload_places(&resolved_scrutinee, selected_variant)
        {
            if !inactive_payloads.contains(&inactive_payload) {
                inactive_payloads.push(inactive_payload);
            }
        }
    }
    for inactive_payload in inactive_payloads {
        owners.set_state(&inactive_payload, OwnerState::NoFreeObligation);
        raw_aliases.clear(&inactive_payload);
        raw_views.clear(&inactive_payload);
        storage_origins.clear(&inactive_payload);
    }
}
