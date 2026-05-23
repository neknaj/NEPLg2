use alloc::boxed::Box;
use alloc::vec::Vec;

use super::collection_slot_lifecycle::{
    CollectionSlotLifecycleOp, CollectionSlotLifecycleRefutation, CollectionSlotState,
};
use super::collection_slot_state_identity::place_covers_slot;
use super::collection_slot_state_merge::merge_collection_slot_states;
use super::collection_slot_state_table::{
    CollectionSlotStateEntry, CollectionSlotStateTable, CollectionSlotTableRefutation,
};
use super::initialized_alias::RawCellAddressAliases;
use super::model::{Place, PlaceProjection, ResourceOffset};
use super::place_utils::{
    push_unique_place, replace_embedded_place_prefixes, replace_place_prefix, should_track,
};
use super::raw_cell_value_flow_alias::place_with_canonical_symbolic_offsets;

impl CollectionSlotStateTable {
    pub(super) fn transfer_storage_prefix(
        &mut self,
        source: &Place,
        target: &Place,
    ) -> Result<(), CollectionSlotTableRefutation> {
        if source == target {
            return Ok(());
        }
        if !should_track(source) {
            return Ok(());
        }
        if !should_track(target) {
            self.clear_storage_prefix(source);
            return Ok(());
        }
        self.require_transfer_target_vacant(source, target)?;

        let moved_entries = self.entries_under_prefix(source, target)?;
        let released_storage = transfer_storage_markers(&self.released_storage, source, target);
        let maybe_released_storage =
            transfer_storage_markers(&self.maybe_released_storage, source, target);
        self.clear_storage_prefix(source);
        for entry in moved_entries {
            self.set_slot_state(&entry.slot, entry.state);
        }
        self.released_storage = released_storage;
        self.maybe_released_storage = maybe_released_storage;
        Ok(())
    }

    pub(super) fn transfer_storage_prefix_with_aliases(
        &mut self,
        source: &Place,
        target: &Place,
        raw_aliases: &RawCellAddressAliases,
    ) -> Result<(), CollectionSlotTableRefutation> {
        if source == target {
            return Ok(());
        }
        if !should_track(source) {
            return Ok(());
        }
        let source_candidates = storage_transfer_alias_candidates(source, raw_aliases);
        if !should_track(target) {
            self.clear_storage_prefixes(&source_candidates);
            return Ok(());
        }
        self.require_transfer_target_vacant_with_sources(&source_candidates, target)?;

        let moved_entries =
            self.entries_under_alias_prefixes(&source_candidates, target, raw_aliases)?;
        let released_storage = transfer_storage_markers_with_aliases(
            &self.released_storage,
            &source_candidates,
            target,
            raw_aliases,
        );
        let maybe_released_storage = transfer_storage_markers_with_aliases(
            &self.maybe_released_storage,
            &source_candidates,
            target,
            raw_aliases,
        );
        self.clear_storage_prefixes(&source_candidates);
        for entry in moved_entries {
            self.set_slot_state(&entry.slot, entry.state);
        }
        self.released_storage = released_storage;
        self.maybe_released_storage = maybe_released_storage;
        Ok(())
    }

    pub(super) fn clear_storage_prefix(&mut self, storage: &Place) {
        self.slots
            .retain(|entry| !place_covers_slot(&entry.slot, storage));
        self.released_storage
            .retain(|released| !place_covers_slot(released, storage));
        self.maybe_released_storage
            .retain(|released| !place_covers_slot(released, storage));
    }

    pub(super) fn clear_storage_prefix_with_aliases(
        &mut self,
        storage: &Place,
        raw_aliases: &RawCellAddressAliases,
    ) {
        let storage_candidates = storage_transfer_alias_candidates(storage, raw_aliases);
        self.clear_storage_prefixes(&storage_candidates);
    }

    fn clear_storage_prefixes(&mut self, storages: &[Place]) {
        self.slots.retain(|entry| {
            !storages
                .iter()
                .any(|storage| place_covers_slot(&entry.slot, storage))
        });
        self.released_storage.retain(|released| {
            !storages
                .iter()
                .any(|storage| place_covers_slot(released, storage))
        });
        self.maybe_released_storage.retain(|released| {
            !storages
                .iter()
                .any(|storage| place_covers_slot(released, storage))
        });
    }

    fn require_transfer_target_vacant(
        &self,
        source: &Place,
        target: &Place,
    ) -> Result<(), CollectionSlotTableRefutation> {
        for entry in self.slots.iter().filter(|entry| {
            place_covers_slot(&entry.slot, target) && !place_covers_slot(&entry.slot, source)
        }) {
            return Err(value_transfer_refutation(&entry.slot, entry.state));
        }
        if let Some(released) = self
            .released_storage
            .iter()
            .find(|released| place_covers_slot(released, target))
        {
            return Err(value_transfer_refutation(
                released,
                CollectionSlotState::Released,
            ));
        }
        if let Some(released) = self
            .maybe_released_storage
            .iter()
            .find(|released| place_covers_slot(released, target))
        {
            return Err(value_transfer_refutation(
                released,
                CollectionSlotState::MaybeReleased,
            ));
        }
        Ok(())
    }

    fn require_transfer_target_vacant_with_sources(
        &self,
        sources: &[Place],
        target: &Place,
    ) -> Result<(), CollectionSlotTableRefutation> {
        for entry in self.slots.iter().filter(|entry| {
            place_covers_slot(&entry.slot, target)
                && !sources
                    .iter()
                    .any(|source| place_covers_slot(&entry.slot, source))
        }) {
            return Err(value_transfer_refutation(&entry.slot, entry.state));
        }
        if let Some(released) = self
            .released_storage
            .iter()
            .find(|released| place_covers_slot(released, target))
        {
            return Err(value_transfer_refutation(
                released,
                CollectionSlotState::Released,
            ));
        }
        if let Some(released) = self
            .maybe_released_storage
            .iter()
            .find(|released| place_covers_slot(released, target))
        {
            return Err(value_transfer_refutation(
                released,
                CollectionSlotState::MaybeReleased,
            ));
        }
        Ok(())
    }

    fn entries_under_prefix(
        &self,
        source: &Place,
        target: &Place,
    ) -> Result<Vec<CollectionSlotStateEntry>, CollectionSlotTableRefutation> {
        let mut entries: Vec<CollectionSlotStateEntry> = Vec::new();
        for entry in self
            .slots
            .iter()
            .filter(|entry| place_covers_slot(&entry.slot, source))
        {
            let Some(slot) = replace_storage_transfer_place(&entry.slot, source, target) else {
                return Err(value_transfer_refutation(
                    &entry.slot,
                    CollectionSlotState::Uninitialized,
                ));
            };
            entries.push(CollectionSlotStateEntry {
                slot,
                state: entry.state,
            });
        }
        Ok(entries)
    }

    fn entries_under_alias_prefixes(
        &self,
        sources: &[Place],
        target: &Place,
        raw_aliases: &RawCellAddressAliases,
    ) -> Result<Vec<CollectionSlotStateEntry>, CollectionSlotTableRefutation> {
        let mut entries: Vec<CollectionSlotStateEntry> = Vec::new();
        for entry in &self.slots {
            let Some(source) = sources
                .iter()
                .find(|source| place_covers_slot(&entry.slot, source))
            else {
                continue;
            };
            let Some(slot) = replace_storage_transfer_place_with_aliases(
                &entry.slot,
                source,
                target,
                raw_aliases,
            ) else {
                return Err(value_transfer_refutation(
                    &entry.slot,
                    CollectionSlotState::Uninitialized,
                ));
            };
            if let Some(existing) = entries.iter_mut().find(|existing| existing.slot == slot) {
                existing.state = merge_collection_slot_states(existing.state, entry.state);
            } else {
                entries.push(CollectionSlotStateEntry {
                    slot,
                    state: entry.state,
                });
            }
        }
        Ok(entries)
    }
}

fn transfer_storage_markers(markers: &[Place], source: &Place, target: &Place) -> Vec<Place> {
    let mut out = Vec::new();
    for marker in markers {
        let moved = replace_storage_transfer_place(marker, source, target)
            .unwrap_or_else(|| marker.clone());
        push_unique_place(&mut out, &moved);
    }
    out
}

fn transfer_storage_markers_with_aliases(
    markers: &[Place],
    sources: &[Place],
    target: &Place,
    raw_aliases: &RawCellAddressAliases,
) -> Vec<Place> {
    let mut out = Vec::new();
    for marker in markers {
        let moved = sources
            .iter()
            .find_map(|source| {
                replace_storage_transfer_place_with_aliases(marker, source, target, raw_aliases)
            })
            .unwrap_or_else(|| marker.clone());
        push_unique_place(&mut out, &moved);
    }
    out
}

fn replace_storage_transfer_place(place: &Place, source: &Place, target: &Place) -> Option<Place> {
    let moved = replace_place_prefix(place, source, target)?;
    let Some((owner_source, owner_target)) = storage_transfer_owner_prefixes(source, target) else {
        return Some(moved);
    };
    Some(replace_embedded_place_prefixes(
        &moved,
        &owner_source,
        &owner_target,
    ))
}

fn replace_storage_transfer_place_with_aliases(
    place: &Place,
    source: &Place,
    target: &Place,
    raw_aliases: &RawCellAddressAliases,
) -> Option<Place> {
    let moved = replace_place_prefix(place, source, target)?;
    let moved = if let Some((owner_source, owner_target)) =
        storage_transfer_owner_prefixes(source, target)
    {
        replace_embedded_storage_transfer_prefixes(
            &moved,
            &owner_source,
            &owner_target,
            raw_aliases,
        )
    } else {
        moved
    };
    Some(place_with_canonical_symbolic_offsets(&moved, raw_aliases))
}

fn replace_embedded_storage_transfer_prefixes(
    place: &Place,
    source: &Place,
    target: &Place,
    raw_aliases: &RawCellAddressAliases,
) -> Place {
    let mut out = place.clone();
    out.projections = out
        .projections
        .iter()
        .map(|projection| {
            replace_storage_transfer_projection_places(projection, source, target, raw_aliases)
        })
        .collect();
    out
}

fn replace_storage_transfer_projection_places(
    projection: &PlaceProjection,
    source: &Place,
    target: &Place,
    raw_aliases: &RawCellAddressAliases,
) -> PlaceProjection {
    match projection {
        PlaceProjection::Field {
            index,
            offset_bytes,
        } => PlaceProjection::Field {
            index: *index,
            offset_bytes: *offset_bytes,
        },
        PlaceProjection::TupleField {
            index,
            offset_bytes,
        } => PlaceProjection::TupleField {
            index: *index,
            offset_bytes: *offset_bytes,
        },
        PlaceProjection::EnumPayload { variant } => PlaceProjection::EnumPayload {
            variant: variant.clone(),
        },
        PlaceProjection::Deref => PlaceProjection::Deref,
        PlaceProjection::StorageOffset(offset) => PlaceProjection::StorageOffset(
            replace_storage_transfer_offset_places(offset, source, target, raw_aliases),
        ),
    }
}

fn replace_storage_transfer_offset_places(
    offset: &ResourceOffset,
    source: &Place,
    target: &Place,
    raw_aliases: &RawCellAddressAliases,
) -> ResourceOffset {
    match offset {
        ResourceOffset::Known(value) => ResourceOffset::Known(*value),
        ResourceOffset::Symbolic { place } => {
            replace_storage_transfer_offset_place(place, 1, 0, source, target, raw_aliases)
        }
        ResourceOffset::ScaledSymbolic { place, scale } => {
            replace_storage_transfer_offset_place(place, *scale, 0, source, target, raw_aliases)
        }
        ResourceOffset::Offset { place, offset } => {
            replace_storage_transfer_offset_place(place, 1, *offset, source, target, raw_aliases)
        }
        ResourceOffset::ScaledOffset {
            place,
            offset,
            scale,
        } => replace_storage_transfer_offset_place(
            place,
            *scale,
            *offset,
            source,
            target,
            raw_aliases,
        ),
        ResourceOffset::Unknown => ResourceOffset::Unknown,
    }
}

fn replace_storage_transfer_offset_place(
    place: &Place,
    scale: usize,
    offset: i64,
    source: &Place,
    target: &Place,
    raw_aliases: &RawCellAddressAliases,
) -> ResourceOffset {
    let Some(replaced) = replace_place_prefix(place, source, target) else {
        return resource_offset_from_symbolic_shift(place.clone(), scale, offset);
    };
    if scalar_places_equivalent(place, &replaced, raw_aliases) {
        return resource_offset_from_symbolic_shift(replaced, scale, offset);
    }
    if let Some(offset) = storage_transfer_offset_shift(place, &replaced, offset, raw_aliases) {
        return resource_offset_from_symbolic_shift(replaced, scale, offset);
    }
    resource_offset_from_symbolic_shift(place.clone(), scale, offset)
}

fn storage_transfer_offset_shift(
    source_place: &Place,
    target_place: &Place,
    existing_offset: i64,
    raw_aliases: &RawCellAddressAliases,
) -> Option<i64> {
    let mut shift = None;
    for (candidate, offset) in raw_aliases.i32_offset_targets(source_place) {
        if !scalar_places_equivalent(&candidate, target_place, raw_aliases) {
            continue;
        }
        let shifted = existing_offset.checked_sub(offset)?;
        match shift {
            Some(existing) if existing != shifted => return None,
            Some(_) => {}
            None => shift = Some(shifted),
        }
    }
    shift
}

fn scalar_places_equivalent(
    left: &Place,
    right: &Place,
    raw_aliases: &RawCellAddressAliases,
) -> bool {
    left == right
        || raw_aliases
            .scalar_aliases_for_value(left)
            .iter()
            .any(|alias| alias == right)
        || raw_aliases
            .scalar_aliases_for_value(right)
            .iter()
            .any(|alias| alias == left)
}

fn resource_offset_from_symbolic_shift(place: Place, scale: usize, offset: i64) -> ResourceOffset {
    match (scale, offset) {
        (1, 0) => ResourceOffset::Symbolic {
            place: Box::new(place),
        },
        (_, 0) => ResourceOffset::ScaledSymbolic {
            place: Box::new(place),
            scale,
        },
        (1, _) => ResourceOffset::Offset {
            place: Box::new(place),
            offset,
        },
        _ => ResourceOffset::ScaledOffset {
            place: Box::new(place),
            offset,
            scale,
        },
    }
}

fn storage_transfer_owner_prefixes(source: &Place, target: &Place) -> Option<(Place, Place)> {
    let common_suffix_len = common_projection_suffix_len(&source.projections, &target.projections);
    if common_suffix_len == 0 || common_suffix_len > source.projections.len() {
        return None;
    }
    let mut owner_source = source.clone();
    owner_source
        .projections
        .truncate(source.projections.len() - common_suffix_len);
    let mut owner_target = target.clone();
    owner_target
        .projections
        .truncate(target.projections.len() - common_suffix_len);
    if owner_source == *source || owner_source == owner_target {
        return None;
    }
    Some((owner_source, owner_target))
}

fn common_projection_suffix_len(
    left: &[super::model::PlaceProjection],
    right: &[super::model::PlaceProjection],
) -> usize {
    left.iter()
        .rev()
        .zip(right.iter().rev())
        .take_while(|(left, right)| left == right)
        .count()
}

fn storage_transfer_alias_candidates(
    source: &Place,
    raw_aliases: &RawCellAddressAliases,
) -> Vec<Place> {
    let mut candidates = Vec::new();
    push_unique_place(&mut candidates, source);
    for alias in raw_aliases.raw_address_aliases_for_value(source) {
        push_unique_place(&mut candidates, &alias);
    }
    for alias in raw_aliases.prefix_aliases_for(source) {
        push_unique_place(&mut candidates, &alias);
    }
    let canonical_owner = raw_aliases.canonicalize_owner_cell_address(source);
    push_unique_place(&mut candidates, &canonical_owner);
    let canonical_offset = place_with_canonical_symbolic_offsets(source, raw_aliases);
    push_unique_place(&mut candidates, &canonical_offset);
    let canonical_owner_offset =
        place_with_canonical_symbolic_offsets(&canonical_owner, raw_aliases);
    push_unique_place(&mut candidates, &canonical_owner_offset);
    candidates
}

fn value_transfer_refutation(
    slot: &Place,
    state: CollectionSlotState,
) -> CollectionSlotTableRefutation {
    CollectionSlotTableRefutation {
        slot: slot.clone(),
        reason: CollectionSlotLifecycleRefutation::Unavailable {
            operation: CollectionSlotLifecycleOp::ValueTransfer,
            state,
        },
    }
}
