use super::initialized_alias::RawCellAddressAliases;
use super::model::{OwnerState, Place};
use super::owner_state::OwnerTable;
use super::place_utils::should_track;
use super::storage_origin::StorageOriginTable;

pub(super) fn transfer_owner_state(
    owners: &mut OwnerTable,
    raw_aliases: &mut RawCellAddressAliases,
    storage_origins: &mut StorageOriginTable,
    resolved_source: &Place,
    source: &Place,
    target: &Place,
    state: OwnerState,
) {
    owners.set_state(resolved_source, OwnerState::Moved);
    if should_track(target) {
        owners.set_state(target, state);
        storage_origins.move_origin(resolved_source, target);
        raw_aliases.clear(source);
        raw_aliases.clear(resolved_source);
        raw_aliases.mark(target);
    } else {
        storage_origins.clear(resolved_source);
    }
}
