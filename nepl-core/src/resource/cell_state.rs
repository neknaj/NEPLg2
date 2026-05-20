use alloc::vec::Vec;

use crate::layout::aggregate_fields_with_offsets;
use crate::types::{TypeCtx, TypeId};

use super::cell_state_raw_range::{
    merge_initialized_raw_byte_ranges, rekey_initialized_raw_byte_ranges, InitializedRawByteRange,
};
use super::cell_state_raw_range_merge::merge_initialized_raw_byte_ranges_with_raw_aliases;
use super::initialized_alias::RawCellAddressAliases;
use super::model::{CellState, CellStateEntry, Place, PlaceProjection, ResourceOffset};
use super::place_utils::{
    place_suffix_after_prefix, place_with_suffix, projection_result_type, push_unique_place,
    raw_memory_cell_place, should_track,
};
use super::type_pattern::type_pattern_matches;

#[derive(Debug, Clone, Default)]
pub(super) struct CellTable {
    cells: Vec<CellStateEntry>,
    owned_raw_storage_roots: Vec<Place>,
    external_raw_storage_roots: Vec<Place>,
    pub(super) initialized_raw_byte_ranges: Vec<InitializedRawByteRange>,
}
impl CellTable {
    pub(super) fn into_entries(self) -> Vec<CellStateEntry> {
        self.cells
    }

    pub(super) fn entries(&self) -> &[CellStateEntry] {
        &self.cells
    }

    pub(super) fn availability_state(&self, place: &Place) -> CellState {
        self.availability_state_by(place, None, &|left, right| left == right)
    }

    pub(super) fn availability_state_with_types(
        &self,
        types: &TypeCtx,
        place: &Place,
    ) -> CellState {
        self.availability_state_by(place, Some(types), &|left, right| {
            type_pattern_matches(types, left, right)
        })
    }

    fn availability_state_by(
        &self,
        place: &Place,
        types: Option<&TypeCtx>,
        type_matches: &impl Fn(TypeId, TypeId) -> bool,
    ) -> CellState {
        if let Some(state) = self.state(place) {
            if !matches!(state, CellState::Initialized(_)) {
                return state;
            }
        }
        for entry in self.ancestor_entries(place) {
            if !matches!(entry.state, CellState::Initialized(_))
                && cell_descendant_state_flows(&entry.place, place)
            {
                return entry.state;
            }
        }
        for entry in self.descendant_entries(place) {
            if !matches!(entry.state, CellState::Initialized(_))
                && cell_descendant_state_flows(place, &entry.place)
            {
                return entry.state;
            }
        }
        for entry in &self.cells {
            if entry.place != *place
                && !matches!(entry.state, CellState::Initialized(_))
                && raw_cell_state_flows_to_query(&entry.place, place)
            {
                return entry.state.clone();
            }
        }
        if let Some(state @ CellState::Initialized(_)) = self.state(place) {
            return state;
        }
        for entry in &self.cells {
            if let CellState::Initialized(ty) = entry.state {
                if initialized_state_flows_to_by(&entry.place, place, ty, types, type_matches) {
                    return CellState::Initialized(place.ty);
                }
            }
        }
        if self.raw_cell_place_is_untracked_external(place) {
            return CellState::Initialized(place.ty);
        }
        CellState::Uninit
    }

    pub(super) fn mark_initialized(&mut self, place: &Place) {
        self.set_state(place, CellState::Initialized(place.ty));
        self.clear_descendants(place);
    }

    pub(super) fn mark_raw_cell_moved(&mut self, address: &Place, ty: TypeId) {
        self.cells
            .retain(|entry| !raw_cell_entry_invalidated_by_raw_move(entry, address));
        self.clear_initialized_raw_byte_ranges_under(address);
        let cell = raw_memory_cell_place(address, ty);
        self.set_state(&cell, CellState::Moved);
    }

    pub(super) fn set_state(&mut self, place: &Place, state: CellState) {
        if !should_track(place) {
            return;
        }
        if let Some(entry) = self.cells.iter_mut().find(|entry| entry.place == *place) {
            entry.state = state;
        } else {
            self.cells.push(CellStateEntry {
                place: place.clone(),
                state,
            });
        }
    }

    pub(super) fn mark_owned_raw_storage_root(&mut self, address: &Place) {
        push_unique_place(&mut self.owned_raw_storage_roots, address);
    }

    pub(super) fn mark_external_raw_storage_root(&mut self, address: &Place) {
        push_unique_place(&mut self.external_raw_storage_roots, address);
    }

    pub(super) fn release_owned_raw_storage_under(&mut self, address: &Place) {
        self.owned_raw_storage_roots
            .retain(|root| !raw_addresses_overlap(root, address));
    }

    pub(super) fn owns_raw_storage_under(&self, address: &Place) -> bool {
        self.owned_raw_storage_roots
            .iter()
            .any(|root| raw_addresses_overlap(root, address))
    }

    pub(super) fn raw_cell_is_untracked_external(&self, address: &Place) -> bool {
        !self.has_raw_cell_entry_under(address)
            && !self.owns_raw_storage_under(address)
            && self
                .external_raw_storage_roots
                .iter()
                .any(|root| external_raw_storage_address_overlaps(root, address))
    }

    pub(super) fn external_raw_storage_overlaps(&self, address: &Place) -> bool {
        self.external_raw_storage_roots
            .iter()
            .any(|root| external_raw_storage_address_overlaps(root, address))
    }

    pub(super) fn raw_address_has_tracked_storage(&self, address: &Place) -> bool {
        self.has_raw_cell_entry_under(address)
            || self.owns_raw_storage_under(address)
            || self.external_raw_storage_overlaps(address)
    }

    pub(super) fn clear_raw_cells_under(&mut self, address: &Place) {
        self.cells
            .retain(|entry| raw_cell_suffix_after_address(&entry.place, address).is_none());
        self.clear_initialized_raw_byte_ranges_under(address);
    }

    pub(super) fn clear_raw_cells_overwritten_by_store(
        &mut self,
        address: &Place,
        value_ty: TypeId,
        types: &TypeCtx,
    ) {
        self.cells.retain(|entry| {
            raw_cell_suffix_after_address(&entry.place, address).is_none()
                || initialized_copy_cell_survives_store(entry, value_ty, types)
        });
    }

    pub(super) fn extend_entries(&mut self, entries: Vec<CellStateEntry>) {
        for entry in entries {
            self.set_state(&entry.place, entry.state);
        }
    }

    pub(super) fn rekey_raw_cells(&mut self, source: &Place, target: &Place) {
        if source == target {
            return;
        }
        let mut relocated = Vec::new();
        self.cells.retain(|entry| {
            let Some(suffix) = raw_cell_suffix_after_address(&entry.place, source) else {
                return true;
            };
            relocated.push(CellStateEntry {
                place: place_with_suffix(target, &suffix, entry.place.ty),
                state: entry.state.clone(),
            });
            false
        });
        self.extend_entries(relocated);
        rekey_raw_storage_roots(&mut self.owned_raw_storage_roots, source, target);
        rekey_raw_storage_roots(&mut self.external_raw_storage_roots, source, target);
        rekey_initialized_raw_byte_ranges(&mut self.initialized_raw_byte_ranges, source, target);
    }

    pub(super) fn live_non_copy_raw_cells_under(
        &self,
        address: &Place,
        types: &TypeCtx,
    ) -> Vec<CellStateEntry> {
        self.cells
            .iter()
            .filter(|entry| {
                raw_cell_suffix_after_address(&entry.place, address).is_some()
                    && raw_cell_state_has_live_non_copy_obligation(entry, types)
            })
            .cloned()
            .collect()
    }

    pub(super) fn copy_initialized_copy_raw_cells(
        &self,
        source: &Place,
        destination: &Place,
        types: &TypeCtx,
    ) -> Vec<CellStateEntry> {
        self.cells
            .iter()
            .filter_map(|entry| {
                let suffix = raw_cell_suffix_after_address(&entry.place, source)?;
                let CellState::Initialized(ty) = entry.state else {
                    return None;
                };
                if !types.is_copy(ty) {
                    return None;
                }
                Some(CellStateEntry {
                    place: place_with_suffix(destination, &suffix, entry.place.ty),
                    state: entry.state.clone(),
                })
            })
            .collect()
    }

    pub(super) fn merge_paths(paths: &[CellTable]) -> Self {
        let mut out = CellTable::default();
        let mut places = Vec::new();
        for path in paths {
            for entry in &path.cells {
                push_unique_place(&mut places, &entry.place);
            }
            for root in &path.owned_raw_storage_roots {
                push_unique_place(&mut out.owned_raw_storage_roots, root);
            }
            for root in &path.external_raw_storage_roots {
                push_unique_place(&mut out.external_raw_storage_roots, root);
            }
        }
        out.initialized_raw_byte_ranges = merge_initialized_raw_byte_ranges(paths);
        for place in places {
            let mut states = paths.iter().map(|path| path.availability_state(&place));
            if let Some(mut merged) = states.next() {
                for state in states {
                    merged = merge_cell_states(merged, state);
                }
                out.set_state(&place, merged);
            }
        }
        out
    }

    pub(super) fn merge_paths_with_raw_aliases(
        paths: &[CellTable],
        raw_alias_paths: &[RawCellAddressAliases],
        merged_raw_aliases: &RawCellAddressAliases,
    ) -> Self {
        let mut out = Self::merge_paths(paths);
        out.initialized_raw_byte_ranges = merge_initialized_raw_byte_ranges_with_raw_aliases(
            paths,
            raw_alias_paths,
            merged_raw_aliases,
        );
        out
    }

    fn has_raw_cell_entry_under(&self, address: &Place) -> bool {
        self.cells
            .iter()
            .any(|entry| raw_cell_suffix_after_address(&entry.place, address).is_some())
    }

    fn raw_cell_place_is_untracked_external(&self, place: &Place) -> bool {
        self.cells.iter().all(|entry| entry.place != *place)
            && self
                .owned_raw_storage_roots
                .iter()
                .all(|root| raw_cell_suffix_after_address(place, root).is_none())
            && self
                .external_raw_storage_roots
                .iter()
                .any(|root| external_raw_cell_suffix_after_storage_root(place, root).is_some())
    }

    fn state(&self, place: &Place) -> Option<CellState> {
        self.cells
            .iter()
            .find(|entry| entry.place == *place)
            .map(|entry| entry.state.clone())
    }

    fn ancestor_entries(&self, place: &Place) -> Vec<CellStateEntry> {
        self.cells
            .iter()
            .filter(|entry| {
                entry.place != *place && place_suffix_after_prefix(place, &entry.place).is_some()
            })
            .cloned()
            .collect()
    }

    fn descendant_entries(&self, prefix: &Place) -> Vec<CellStateEntry> {
        self.cells
            .iter()
            .filter(|entry| {
                entry.place != *prefix && place_suffix_after_prefix(&entry.place, prefix).is_some()
            })
            .cloned()
            .collect()
    }

    fn clear_descendants(&mut self, prefix: &Place) {
        self.cells.retain(|entry| {
            entry.place == *prefix || place_suffix_after_prefix(&entry.place, prefix).is_none()
        });
    }
}

fn initialized_state_flows_to_by(
    prefix: &Place,
    place: &Place,
    initialized_ty: TypeId,
    types: Option<&TypeCtx>,
    type_matches: &impl Fn(TypeId, TypeId) -> bool,
) -> bool {
    let Some(suffix) = place_suffix_after_prefix(place, prefix)
        .or_else(|| place_suffix_after_address_prefix(place, prefix))
    else {
        return false;
    };
    if suffix
        .iter()
        .any(|projection| matches!(projection, PlaceProjection::Deref))
    {
        return types.is_some_and(|types| {
            initialized_storage_view_flows_to(types, initialized_ty, place.ty, &suffix)
        });
    }
    if suffix.is_empty()
        && prefix
            .projections
            .iter()
            .any(|projection| matches!(projection, PlaceProjection::Deref))
    {
        return type_matches(initialized_ty, place.ty);
    }
    true
}

fn initialized_storage_view_flows_to(
    types: &TypeCtx,
    initialized_ty: TypeId,
    query_ty: TypeId,
    suffix: &[PlaceProjection],
) -> bool {
    if !types.is_copy(query_ty) {
        return false;
    }
    let [PlaceProjection::StorageOffset(ResourceOffset::Known(offset_bytes)), PlaceProjection::Deref, rest @ ..] =
        suffix
    else {
        return false;
    };
    let Some(field) = aggregate_fields_with_offsets(types, initialized_ty)
        .into_iter()
        .find(|field| field.offset == *offset_bytes)
    else {
        return false;
    };
    let Some(projected_ty) = rest.iter().try_fold(field.ty, |ty, projection| {
        if matches!(
            projection,
            PlaceProjection::Deref | PlaceProjection::StorageOffset(_)
        ) {
            None
        } else {
            projection_result_type(types, ty, projection)
        }
    }) else {
        return false;
    };
    type_pattern_matches(types, projected_ty, query_ty)
}

fn cell_descendant_state_flows(prefix: &Place, place: &Place) -> bool {
    place_suffix_after_prefix(place, prefix)
        .map(|suffix| {
            suffix.iter().all(|projection| {
                !matches!(
                    projection,
                    PlaceProjection::Deref | PlaceProjection::StorageOffset(_)
                )
            })
        })
        .unwrap_or(false)
}

fn raw_cell_state_flows_to_query(entry: &Place, query: &Place) -> bool {
    let Some(query_address) = raw_cell_address_prefix(query) else {
        return false;
    };
    raw_cell_suffix_after_address(entry, &query_address).is_some()
}

pub(super) fn raw_cell_address_prefix(cell: &Place) -> Option<Place> {
    let deref_index = cell
        .projections
        .iter()
        .position(|projection| matches!(projection, PlaceProjection::Deref))?;
    let mut address = cell.clone();
    address.projections.truncate(deref_index);
    Some(address)
}

fn raw_cell_state_has_live_non_copy_obligation(entry: &CellStateEntry, types: &TypeCtx) -> bool {
    match entry.state {
        CellState::Initialized(ty) => !types.is_copy(ty),
        CellState::MaybeMoved => !types.is_copy(entry.place.ty),
        CellState::Uninit | CellState::Moved | CellState::Dropped => false,
    }
}

fn initialized_copy_cell_survives_store(
    entry: &CellStateEntry,
    value_ty: TypeId,
    types: &TypeCtx,
) -> bool {
    let CellState::Initialized(entry_ty) = entry.state else {
        return false;
    };
    types.is_copy(entry_ty)
        && types.is_copy(value_ty)
        && type_pattern_matches(types, entry_ty, value_ty)
}

pub(super) fn raw_cell_suffix_after_address(
    cell: &Place,
    address: &Place,
) -> Option<Vec<PlaceProjection>> {
    let suffix = place_suffix_after_address_prefix(cell, address)?;
    if suffix
        .iter()
        .any(|projection| matches!(projection, PlaceProjection::Deref))
    {
        Some(suffix)
    } else {
        None
    }
}

pub(super) fn raw_address_suffix_after_address(
    address: &Place,
    prefix: &Place,
) -> Option<Vec<PlaceProjection>> {
    let suffix = place_suffix_after_address_prefix(address, prefix)?;
    if suffix
        .iter()
        .any(|projection| matches!(projection, PlaceProjection::Deref))
    {
        None
    } else {
        Some(suffix)
    }
}

fn raw_cell_belongs_to_address_cell(cell: &Place, address: &Place) -> bool {
    raw_cell_suffix_after_address(cell, address)
        .and_then(|suffix| suffix.first().cloned())
        .is_some_and(|projection| matches!(projection, PlaceProjection::Deref))
}

fn raw_cell_entry_invalidated_by_raw_move(entry: &CellStateEntry, address: &Place) -> bool {
    raw_cell_belongs_to_address_cell(&entry.place, address)
        || (matches!(entry.state, CellState::Initialized(_))
            && raw_cell_address_prefix(&entry.place)
                .is_some_and(|entry_address| raw_addresses_overlap(&entry_address, address)))
}

pub(super) fn raw_addresses_overlap(left: &Place, right: &Place) -> bool {
    place_suffix_after_address_prefix(left, right).is_some()
        || place_suffix_after_address_prefix(right, left).is_some()
}

fn external_raw_storage_address_overlaps(root: &Place, address: &Place) -> bool {
    raw_addresses_overlap(root, address)
        || place_suffix_after_external_storage_root(address, root).is_some()
        || place_suffix_after_external_storage_root(root, address).is_some()
}

fn external_raw_cell_suffix_after_storage_root(
    cell: &Place,
    root: &Place,
) -> Option<Vec<PlaceProjection>> {
    if let Some(suffix) = raw_cell_suffix_after_address(cell, root) {
        return Some(suffix);
    }
    let address = raw_cell_address_prefix(cell)?;
    let mut suffix = place_suffix_after_external_storage_root(&address, root)?;
    suffix.extend(
        cell.projections[address.projections.len()..]
            .iter()
            .cloned(),
    );
    Some(suffix)
}

fn rekey_raw_storage_roots(roots: &mut Vec<Place>, source: &Place, target: &Place) {
    let mut relocated = Vec::new();
    roots.retain(|root| {
        if !raw_addresses_overlap(root, source) {
            return true;
        }
        if let Some(suffix) = place_suffix_after_address_prefix(root, source) {
            let place = place_with_suffix(target, &suffix, root.ty);
            push_unique_place(&mut relocated, &place);
            return false;
        }
        if raw_address_covers(root, source) {
            push_unique_place(&mut relocated, target);
            return true;
        }
        true
    });
    for place in relocated {
        push_unique_place(roots, &place);
    }
}

fn raw_address_covers(root: &Place, address: &Place) -> bool {
    place_suffix_after_address_prefix(address, root).is_some()
}

pub(super) fn place_suffix_after_address_prefix(
    place: &Place,
    prefix: &Place,
) -> Option<Vec<PlaceProjection>> {
    if place.root != prefix.root {
        return None;
    }
    let mut place_index = 0;
    for prefix_projection in &prefix.projections {
        if storage_offset_is_zero(prefix_projection) {
            continue;
        }
        while matches!(
            place.projections.get(place_index),
            Some(projection) if storage_offset_is_zero(projection)
        ) {
            place_index += 1;
        }
        if matches!(
            prefix_projection,
            PlaceProjection::StorageOffset(super::model::ResourceOffset::Unknown)
        ) {
            if matches!(
                place.projections.get(place_index),
                Some(PlaceProjection::StorageOffset(_))
            ) {
                place_index += 1;
            }
            continue;
        }
        if matches!(
            prefix_projection,
            PlaceProjection::StorageOffset(
                super::model::ResourceOffset::Symbolic { .. }
                    | super::model::ResourceOffset::ScaledSymbolic { .. }
            )
        ) && !matches!(
            place.projections.get(place_index),
            Some(PlaceProjection::StorageOffset(_))
        ) {
            continue;
        }
        let place_projection = place.projections.get(place_index)?;
        if !address_projection_matches(place_projection, prefix_projection) {
            return None;
        }
        place_index += 1;
    }
    while matches!(
        place.projections.get(place_index),
        Some(projection) if storage_offset_is_zero(projection)
    ) {
        place_index += 1;
    }
    Some(place.projections[place_index..].to_vec())
}

fn place_suffix_after_external_storage_root(
    place: &Place,
    root: &Place,
) -> Option<Vec<PlaceProjection>> {
    if place.root != root.root {
        return None;
    }
    let mut place_index = 0;
    for root_projection in &root.projections {
        if storage_offset_is_zero(root_projection) {
            continue;
        }
        while matches!(
            place.projections.get(place_index),
            Some(projection) if storage_offset_is_zero(projection)
        ) {
            place_index += 1;
        }
        if matches!(root_projection, PlaceProjection::Deref)
            && matches!(
                place.projections.get(place_index),
                Some(
                    PlaceProjection::Field { .. }
                        | PlaceProjection::TupleField { .. }
                        | PlaceProjection::EnumPayload { .. }
                        | PlaceProjection::StorageOffset(_)
                )
            )
        {
            continue;
        }
        let place_projection = place.projections.get(place_index)?;
        if !address_projection_matches(place_projection, root_projection) {
            return None;
        }
        place_index += 1;
    }
    while matches!(
        place.projections.get(place_index),
        Some(projection) if storage_offset_is_zero(projection)
    ) {
        place_index += 1;
    }
    Some(place.projections[place_index..].to_vec())
}

fn storage_offset_is_zero(projection: &PlaceProjection) -> bool {
    matches!(
        projection,
        PlaceProjection::StorageOffset(super::model::ResourceOffset::Known(0))
    )
}

fn address_projection_matches(place: &PlaceProjection, prefix: &PlaceProjection) -> bool {
    match (place, prefix) {
        (PlaceProjection::StorageOffset(left), PlaceProjection::StorageOffset(right)) => {
            resource_offsets_may_overlap(left, right)
        }
        _ => place == prefix,
    }
}

fn resource_offsets_may_overlap(
    left: &super::model::ResourceOffset,
    right: &super::model::ResourceOffset,
) -> bool {
    match (left, right) {
        (super::model::ResourceOffset::Known(left), super::model::ResourceOffset::Known(right)) => {
            left == right
        }
        (super::model::ResourceOffset::Unknown, _)
        | (_, super::model::ResourceOffset::Unknown)
        | (super::model::ResourceOffset::Symbolic { .. }, _)
        | (_, super::model::ResourceOffset::Symbolic { .. })
        | (super::model::ResourceOffset::ScaledSymbolic { .. }, _)
        | (_, super::model::ResourceOffset::ScaledSymbolic { .. }) => true,
    }
}

fn merge_cell_states(left: CellState, right: CellState) -> CellState {
    if left == right {
        return left;
    }
    match (left, right) {
        (CellState::Initialized(left_ty), CellState::Initialized(right_ty))
            if left_ty == right_ty =>
        {
            CellState::Initialized(left_ty)
        }
        (CellState::Uninit, CellState::Uninit) => CellState::Uninit,
        (CellState::Moved, CellState::Moved) => CellState::Moved,
        (CellState::Dropped, CellState::Dropped) => CellState::Dropped,
        _ => CellState::MaybeMoved,
    }
}
