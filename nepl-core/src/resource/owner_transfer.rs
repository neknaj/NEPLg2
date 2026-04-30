use super::initialized_alias::RawCellAddressAliases;
use super::model::{OwnerState, Place};
use super::owner_state::OwnerTable;
use super::place_utils::{places_overlap, should_track};
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
    retire_transferred_aliases(
        owners,
        raw_aliases,
        storage_origins,
        resolved_source,
        target,
    );
    owners.set_state(resolved_source, OwnerState::Moved);
    if should_track(target) {
        owners.set_state(target, state);
        storage_origins.move_origin(resolved_source, target);
        raw_aliases.move_owner_aliases(resolved_source, target);
        if !same_owner_path(source, resolved_source) {
            raw_aliases.clear(source);
        }
    } else {
        storage_origins.clear(resolved_source);
        raw_aliases.clear(source);
        raw_aliases.clear(resolved_source);
    }
}

pub(super) fn move_owner_state_out(
    owners: &mut OwnerTable,
    raw_aliases: &mut RawCellAddressAliases,
    storage_origins: &mut StorageOriginTable,
    place: &Place,
) {
    retire_transferred_aliases(owners, raw_aliases, storage_origins, place, place);
    owners.set_state(place, OwnerState::Moved);
    storage_origins.clear(place);
}

pub(super) fn move_owner_state_out_protecting_aliases(
    owners: &mut OwnerTable,
    raw_aliases: &mut RawCellAddressAliases,
    storage_origins: &mut StorageOriginTable,
    place: &Place,
    protected_aliases: &[Place],
) {
    retire_transferred_aliases_except(
        owners,
        raw_aliases,
        storage_origins,
        place,
        place,
        protected_aliases,
    );
    owners.set_state(place, OwnerState::Moved);
    storage_origins.clear(place);
}

pub(super) fn free_owner_state(
    owners: &mut OwnerTable,
    raw_aliases: &mut RawCellAddressAliases,
    storage_origins: &mut StorageOriginTable,
    place: &Place,
) {
    retire_transferred_aliases(owners, raw_aliases, storage_origins, place, place);
    owners.set_state(place, OwnerState::Freed);
    storage_origins.clear(place);
}

fn retire_transferred_aliases(
    owners: &mut OwnerTable,
    raw_aliases: &RawCellAddressAliases,
    storage_origins: &mut StorageOriginTable,
    source: &Place,
    target: &Place,
) {
    retire_transferred_aliases_except(owners, raw_aliases, storage_origins, source, target, &[]);
}

fn retire_transferred_aliases_except(
    owners: &mut OwnerTable,
    raw_aliases: &RawCellAddressAliases,
    storage_origins: &mut StorageOriginTable,
    source: &Place,
    target: &Place,
    protected_aliases: &[Place],
) {
    for alias in raw_aliases.aliases_for(source) {
        if same_owner_path(&alias, source) || same_owner_path(&alias, target) {
            continue;
        }
        if protected_aliases
            .iter()
            .any(|protected| places_overlap(&alias, protected))
        {
            continue;
        }
        match owners.state(&alias) {
            Some(
                OwnerState::Live { .. }
                | OwnerState::Reserved { .. }
                | OwnerState::MaybeFreed { .. },
            ) => {
                owners.set_state(&alias, OwnerState::Moved);
                storage_origins.clear(&alias);
            }
            Some(OwnerState::Moved | OwnerState::Freed | OwnerState::NoFreeObligation) | None => {}
        }
    }
}

fn same_owner_path(left: &Place, right: &Place) -> bool {
    left.root == right.root && left.projections == right.projections
}
