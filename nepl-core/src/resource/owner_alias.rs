use super::initialized_alias::RawCellAddressAliases;
use super::model::{OwnerState, Place};
use super::owner_state::OwnerTable;

pub(super) fn resolve_owner_alias_place(
    owners: &OwnerTable,
    raw_aliases: &RawCellAddressAliases,
    place: &Place,
) -> Place {
    match owners.state(place) {
        Some(OwnerState::Live { .. })
        | Some(OwnerState::Moved)
        | Some(OwnerState::Freed)
        | Some(OwnerState::MaybeFreed { .. }) => return place.clone(),
        Some(OwnerState::NoFreeObligation) | None => {}
    }
    // Descendant owner states make the aggregate a concrete ownership root.
    // Raw-address aliases are only a fallback for read temporaries without local owner state.
    if !owners.descendant_entries(place).is_empty() {
        return place.clone();
    }
    for alias in raw_aliases.aliases_for(place) {
        match owners.state(&alias) {
            Some(OwnerState::Live { .. })
            | Some(OwnerState::Moved)
            | Some(OwnerState::Freed)
            | Some(OwnerState::MaybeFreed { .. }) => return alias,
            Some(OwnerState::NoFreeObligation) | None => {}
        }
        if owners.has_tracked_state_under(&alias) {
            return alias;
        }
    }
    for alias in raw_aliases.prefix_aliases_for(place) {
        match owners.state(&alias) {
            Some(OwnerState::Live { .. })
            | Some(OwnerState::Moved)
            | Some(OwnerState::Freed)
            | Some(OwnerState::MaybeFreed { .. }) => return alias,
            Some(OwnerState::NoFreeObligation) | None => {}
        }
        if owners.has_tracked_state_under(&alias) {
            return alias;
        }
    }
    place.clone()
}
