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
    match owners.state(place) {
        Some(OwnerState::Live { .. })
        | Some(OwnerState::Reserved { .. })
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
    let aliases = raw_aliases.aliases_for(place);
    if let Some(alias) = best_available_owner_state_alias(owners, &aliases) {
        return alias;
    }
    let mut unavailable_alias = None;
    for alias in &aliases {
        if owners.has_tracked_state_under(&alias) {
            return alias.clone();
        }
        if unavailable_alias.is_none() {
            unavailable_alias = unavailable_owner_state_alias(owners, alias);
        }
    }
    if let Some(alias) = unavailable_alias {
        return alias;
    }
    let prefix_aliases = raw_aliases.prefix_aliases_for(place);
    if let Some(alias) = best_available_owner_state_alias(owners, &prefix_aliases) {
        return alias;
    }
    let mut unavailable_prefix_alias = None;
    for alias in &prefix_aliases {
        if owners.has_tracked_state_under(&alias) {
            return alias.clone();
        }
        if unavailable_prefix_alias.is_none() {
            unavailable_prefix_alias = unavailable_owner_state_alias(owners, alias);
        }
    }
    if let Some(alias) = unavailable_prefix_alias {
        return alias;
    }
    place.clone()
}

fn best_available_owner_state_alias(owners: &OwnerTable, aliases: &[Place]) -> Option<Place> {
    let mut best: Option<(u8, usize, Place)> = None;
    for (index, alias) in aliases.iter().enumerate() {
        let Some(rank) = owners
            .state(alias)
            .as_ref()
            .and_then(available_owner_state_alias_rank)
        else {
            continue;
        };
        if best
            .as_ref()
            .is_none_or(|(best_rank, best_index, _)| (rank, index) < (*best_rank, *best_index))
        {
            best = Some((rank, index, alias.clone()));
        }
    }
    best.map(|(_, _, alias)| alias)
}

fn available_owner_state_alias_rank(state: &OwnerState) -> Option<u8> {
    match state {
        OwnerState::Live { .. } | OwnerState::MaybeFreed { .. } => Some(0),
        OwnerState::Reserved { .. } => Some(1),
        OwnerState::NoFreeObligation | OwnerState::Moved | OwnerState::Freed => None,
    }
}

fn unavailable_owner_state_alias(owners: &OwnerTable, alias: &Place) -> Option<Place> {
    match owners.state(alias) {
        Some(OwnerState::Moved | OwnerState::Freed) => Some(alias.clone()),
        Some(
            OwnerState::NoFreeObligation
            | OwnerState::Live { .. }
            | OwnerState::Reserved { .. }
            | OwnerState::MaybeFreed { .. },
        )
        | None => None,
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
                if !place_has_raw_cell_projection(&entry.place)
                    && !projections_have_raw_cell_projection(&suffix)
                {
                    continue;
                }
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
    projections_have_raw_cell_projection(&place.projections)
}

fn projections_have_raw_cell_projection(projections: &[PlaceProjection]) -> bool {
    projections.iter().any(|projection| {
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
