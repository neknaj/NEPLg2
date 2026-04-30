use alloc::vec::Vec;

use super::initialized_alias::RawCellAddressAliases;
use super::model::{OwnerState, OwnerStateEntry, Place, PlaceProjection};
use super::owner_state::OwnerTable;
use super::place_utils::place_suffix_after_prefix;

pub(super) struct AliasedOwnerDescendant {
    pub(super) entry: OwnerStateEntry,
    pub(super) suffix: Vec<PlaceProjection>,
}

pub(super) fn resolve_owner_alias_place(
    owners: &OwnerTable,
    raw_aliases: &RawCellAddressAliases,
    place: &Place,
) -> Place {
    if place_has_concrete_owner_state(owners, place) {
        return place.clone();
    }
    let aliases = raw_aliases.aliases_for(place);
    for alias in &aliases {
        if place_has_concrete_owner_state(owners, alias) {
            return alias.clone();
        }
    }
    let prefix_aliases = raw_aliases.prefix_aliases_for(place);
    for alias in &prefix_aliases {
        if place_has_concrete_owner_state(owners, alias) {
            return alias.clone();
        }
    }
    // Descendant owner states make the aggregate a concrete ownership root.
    // Raw-address aliases are only a fallback for read temporaries without local owner state.
    if place_has_non_no_free_tracked_state(owners, place) {
        return place.clone();
    }
    for alias in aliases {
        if place_has_non_no_free_tracked_state(owners, &alias) {
            return alias;
        }
    }
    for alias in prefix_aliases {
        if place_has_non_no_free_tracked_state(owners, &alias) {
            return alias;
        }
    }
    place.clone()
}

fn place_has_non_no_free_tracked_state(owners: &OwnerTable, place: &Place) -> bool {
    owners
        .state(place)
        .is_some_and(|state| state != OwnerState::NoFreeObligation)
        || owners
            .descendant_entries(place)
            .iter()
            .any(|entry| entry.state != OwnerState::NoFreeObligation)
}

fn place_has_concrete_owner_state(owners: &OwnerTable, place: &Place) -> bool {
    match owners.state(place) {
        Some(OwnerState::Live { .. })
        | Some(OwnerState::Reserved { .. })
        | Some(OwnerState::Moved)
        | Some(OwnerState::Freed)
        | Some(OwnerState::MaybeFreed { .. }) => true,
        Some(OwnerState::NoFreeObligation) | None => owners
            .descendant_entries(place)
            .iter()
            .any(|entry| !matches!(entry.state, OwnerState::NoFreeObligation)),
    }
}

pub(super) fn aliased_owner_descendant_entries(
    owners: &OwnerTable,
    raw_aliases: &RawCellAddressAliases,
    source: &Place,
) -> Vec<AliasedOwnerDescendant> {
    let mut out = Vec::new();
    for entry in owners.entries() {
        if matches!(
            entry.state,
            OwnerState::Reserved { .. } | OwnerState::Moved | OwnerState::Freed
        ) {
            continue;
        }
        if !place_has_raw_cell_projection(&entry.place) {
            continue;
        }
        if entry.place == *source || place_suffix_after_prefix(&entry.place, source).is_some() {
            continue;
        }
        for prefix_len in 0..=entry.place.projections.len() {
            let alias_prefix = place_prefix(&entry.place, prefix_len);
            let suffix_after_alias = entry.place.projections[prefix_len..].to_vec();
            for source_alias in raw_aliases.aliases_for(&alias_prefix) {
                if same_owner_path(&source_alias, &alias_prefix) {
                    continue;
                }
                let Some(mut suffix) = place_suffix_after_prefix(&source_alias, source) else {
                    continue;
                };
                suffix.extend_from_slice(&suffix_after_alias);
                push_unique_aliased_owner_descendant(&mut out, entry.clone(), suffix);
            }
        }
    }
    out
}

fn place_prefix(place: &Place, prefix_len: usize) -> Place {
    let mut out = place.clone();
    out.projections.truncate(prefix_len);
    out
}

fn same_owner_path(left: &Place, right: &Place) -> bool {
    left.root == right.root && left.projections == right.projections
}

fn place_has_raw_cell_projection(place: &Place) -> bool {
    place.projections.iter().any(|projection| {
        matches!(
            projection,
            PlaceProjection::Deref | PlaceProjection::StorageOffset(_)
        )
    })
}

fn push_unique_aliased_owner_descendant(
    entries: &mut Vec<AliasedOwnerDescendant>,
    entry: OwnerStateEntry,
    suffix: Vec<PlaceProjection>,
) {
    if !entries.iter().any(|existing| {
        same_owner_path(&existing.entry.place, &entry.place) && existing.suffix == suffix
    }) {
        entries.push(AliasedOwnerDescendant { entry, suffix });
    }
}
