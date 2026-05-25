use alloc::vec::Vec;

use super::collection_slot_drop_traversal_range::collection_slot_offset_is_inside_initialized_count;
use super::collection_slot_lifecycle::{
    apply_collection_slot_lifecycle_event, CollectionSlotLifecycleEvent, CollectionSlotLifecycleOp,
    CollectionSlotLifecycleRefutation, CollectionSlotState,
};
use super::collection_slot_state_alias::{
    place_covers_slot_with_aliases, storage_aliases_for_place,
};
use super::collection_slot_state_identity::{place_covers_slot, same_collection_slot_identity};
use super::collection_slot_state_merge::merge_collection_slot_states;
use super::initialized_alias::RawCellAddressAliases;
use super::model::{Place, ResourceI32RelationOp};
use super::place_utils::{replace_place_prefix, should_track};
use super::raw_cell_value_flow_alias::{
    raw_cell_place_alias_candidates, raw_cell_places_equivalent,
};
use crate::types::{TypeCtx, TypeId};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CollectionSlotStateEntry {
    pub slot: Place,
    pub state: CollectionSlotState,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct CollectionSlotInitializedRangeStateEntry {
    pub(super) storage: Place,
    pub(super) initialized_count: Place,
    pub(super) value_ty: TypeId,
    pub(super) element_stride: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CollectionSlotTableRefutation {
    pub slot: Place,
    pub reason: CollectionSlotLifecycleRefutation,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CollectionSlotStateTable {
    pub(super) slots: Vec<CollectionSlotStateEntry>,
    pub(super) initialized_ranges: Vec<CollectionSlotInitializedRangeStateEntry>,
    pub(super) maybe_initialized_ranges: Vec<CollectionSlotInitializedRangeStateEntry>,
    pub(super) released_storage: Vec<Place>,
    pub(super) maybe_released_storage: Vec<Place>,
}

impl CollectionSlotStateTable {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn entries(&self) -> &[CollectionSlotStateEntry] {
        &self.slots
    }

    pub(super) fn initialized_ranges(&self) -> &[CollectionSlotInitializedRangeStateEntry] {
        &self.initialized_ranges
    }

    pub(super) fn maybe_initialized_ranges(&self) -> &[CollectionSlotInitializedRangeStateEntry] {
        &self.maybe_initialized_ranges
    }

    pub fn released_storage(&self) -> &[Place] {
        &self.released_storage
    }

    pub fn maybe_released_storage(&self) -> &[Place] {
        &self.maybe_released_storage
    }

    pub fn state(&self, slot: &Place) -> CollectionSlotState {
        if self.storage_release_covers_slot(slot) {
            return CollectionSlotState::Released;
        }
        if self.storage_maybe_release_covers_slot(slot) {
            return CollectionSlotState::MaybeReleased;
        }
        self.slots
            .iter()
            .find(|entry| same_collection_slot_identity(&entry.slot, slot))
            .map(|entry| entry.state)
            .unwrap_or(CollectionSlotState::Uninitialized)
    }

    pub(super) fn state_with_aliases(
        &self,
        slot: &Place,
        raw_aliases: &RawCellAddressAliases,
    ) -> CollectionSlotState {
        let candidates = raw_cell_place_alias_candidates(slot, raw_aliases);
        if candidates
            .iter()
            .any(|candidate| self.storage_release_covers_slot(candidate))
        {
            return CollectionSlotState::Released;
        }
        if candidates
            .iter()
            .any(|candidate| self.storage_maybe_release_covers_slot(candidate))
        {
            return CollectionSlotState::MaybeReleased;
        }
        self.slots
            .iter()
            .find(|entry| slot_matches_alias_candidates(&entry.slot, &candidates, raw_aliases))
            .map(|entry| entry.state)
            .unwrap_or(CollectionSlotState::Uninitialized)
    }

    pub(super) fn state_with_aliases_and_ranges(
        &self,
        types: &TypeCtx,
        slot: &Place,
        raw_aliases: &RawCellAddressAliases,
    ) -> CollectionSlotState {
        let state = self.state_with_aliases(slot, raw_aliases);
        if !matches!(state, CollectionSlotState::Uninitialized) {
            return state;
        }
        self.initialized_range_state_with_aliases(types, slot, raw_aliases)
            .unwrap_or(CollectionSlotState::Uninitialized)
    }

    pub(super) fn mark_initialized_range_with_aliases(
        &mut self,
        storage: &Place,
        initialized_count: &Place,
        value_ty: TypeId,
        element_stride: usize,
        raw_aliases: &RawCellAddressAliases,
    ) {
        let storage = raw_aliases.canonicalize_owner_cell_address(storage);
        let initialized_count = raw_aliases.canonicalize_scalar(initialized_count);
        let entry = CollectionSlotInitializedRangeStateEntry {
            storage,
            initialized_count,
            value_ty,
            element_stride,
        };
        self.maybe_initialized_ranges
            .retain(|existing| existing != &entry);
        if !self.initialized_ranges.contains(&entry) {
            self.initialized_ranges.push(entry);
        }
    }

    pub(super) fn clear_initialized_range_with_aliases(
        &mut self,
        types: &TypeCtx,
        storage: &Place,
        initialized_count: &Place,
        value_ty: TypeId,
        element_stride: usize,
        raw_aliases: &RawCellAddressAliases,
    ) {
        let storage = raw_aliases.canonicalize_owner_cell_address(storage);
        let initialized_count = raw_aliases.canonicalize_scalar(initialized_count);
        self.initialized_ranges.retain(|entry| {
            !same_initialized_range_with_aliases(
                entry,
                &storage,
                &initialized_count,
                value_ty,
                element_stride,
                raw_aliases,
            )
        });
        self.maybe_initialized_ranges.retain(|entry| {
            !same_initialized_range_with_aliases(
                entry,
                &storage,
                &initialized_count,
                value_ty,
                element_stride,
                raw_aliases,
            )
        });
        self.clear_slots_described_by_range_with_aliases(
            types,
            &storage,
            &initialized_count,
            value_ty,
            raw_aliases,
        );
    }

    pub(super) fn entries_covered_by_storage_with_aliases(
        &self,
        storage: &Place,
        raw_aliases: &RawCellAddressAliases,
    ) -> Vec<CollectionSlotStateEntry> {
        let storage_candidates = raw_cell_place_alias_candidates(storage, raw_aliases);
        let mut entries = Vec::<CollectionSlotStateEntry>::new();
        for entry in &self.slots {
            let Some(storage_candidate) = storage_candidates
                .iter()
                .find(|candidate| place_covers_slot(&entry.slot, candidate))
            else {
                continue;
            };
            let slot = replace_place_prefix(&entry.slot, storage_candidate, storage)
                .unwrap_or_else(|| entry.slot.clone());
            if let Some(existing) = entries
                .iter_mut()
                .find(|existing| same_collection_slot_identity(&existing.slot, &slot))
            {
                existing.state = merge_collection_slot_states(existing.state, entry.state);
            } else {
                entries.push(CollectionSlotStateEntry {
                    slot,
                    state: entry.state,
                });
            }
        }
        entries
    }

    pub fn apply_slot_event(
        &mut self,
        types: &TypeCtx,
        slot: &Place,
        event: CollectionSlotLifecycleEvent,
    ) -> Result<CollectionSlotState, CollectionSlotTableRefutation> {
        if !should_track(slot) {
            return Err(CollectionSlotTableRefutation {
                slot: slot.clone(),
                reason: CollectionSlotLifecycleRefutation::Unavailable {
                    operation: collection_slot_event_operation(event),
                    state: CollectionSlotState::Uninitialized,
                },
            });
        }
        let state = self.state(slot);
        let next =
            apply_collection_slot_lifecycle_event(types, state, event).map_err(|reason| {
                CollectionSlotTableRefutation {
                    slot: slot.clone(),
                    reason,
                }
            })?;
        self.set_slot_state(slot, next);
        Ok(next)
    }

    pub(super) fn apply_slot_event_with_aliases(
        &mut self,
        types: &TypeCtx,
        slot: &Place,
        raw_aliases: &RawCellAddressAliases,
        event: CollectionSlotLifecycleEvent,
    ) -> Result<CollectionSlotState, CollectionSlotTableRefutation> {
        if !should_track(slot) {
            return Err(CollectionSlotTableRefutation {
                slot: slot.clone(),
                reason: CollectionSlotLifecycleRefutation::Unavailable {
                    operation: collection_slot_event_operation(event),
                    state: CollectionSlotState::Uninitialized,
                },
            });
        }
        let state = self.state_with_aliases_and_ranges(types, slot, raw_aliases);
        let next =
            apply_collection_slot_lifecycle_event(types, state, event).map_err(|reason| {
                CollectionSlotTableRefutation {
                    slot: slot.clone(),
                    reason,
                }
            })?;
        self.set_slot_state_with_aliases(slot, raw_aliases, next);
        Ok(next)
    }

    pub(super) fn set_slot_state(&mut self, slot: &Place, state: CollectionSlotState) {
        if matches!(state, CollectionSlotState::Uninitialized) {
            self.slots
                .retain(|entry| !same_collection_slot_identity(&entry.slot, slot));
            return;
        }
        if let Some(entry) = self
            .slots
            .iter_mut()
            .find(|entry| same_collection_slot_identity(&entry.slot, slot))
        {
            entry.slot = slot.clone();
            entry.state = state;
        } else {
            self.slots.push(CollectionSlotStateEntry {
                slot: slot.clone(),
                state,
            });
        }
    }

    pub(super) fn set_slot_state_with_aliases(
        &mut self,
        slot: &Place,
        raw_aliases: &RawCellAddressAliases,
        state: CollectionSlotState,
    ) {
        let candidates = raw_cell_place_alias_candidates(slot, raw_aliases);
        if matches!(state, CollectionSlotState::Uninitialized) {
            self.slots.retain(|entry| {
                !slot_matches_alias_candidates(&entry.slot, &candidates, raw_aliases)
            });
            return;
        }
        self.slots
            .retain(|entry| !slot_matches_alias_candidates(&entry.slot, &candidates, raw_aliases));
        self.slots.push(CollectionSlotStateEntry {
            slot: slot.clone(),
            state,
        });
    }

    fn clear_slots_described_by_range_with_aliases(
        &mut self,
        types: &TypeCtx,
        storage: &Place,
        initialized_count: &Place,
        value_ty: TypeId,
        raw_aliases: &RawCellAddressAliases,
    ) {
        // range 事実は collection storage の initialized_count 以下の payload slot を
        // まとめて表す正規形である。DropTraversal や TransformRange がその range を
        // 消費した後に、同じ storage 配下の古い具体 slot 状態を残すと、すでに処理済みの
        // 要素が storage dealloc 時に二重に live と判定される。
        self.slots.retain(|entry| {
            !place_covers_slot_with_aliases(&entry.slot, storage, raw_aliases)
                || !slot_state_describes_range_value(entry.state, value_ty)
                || !collection_slot_offset_is_inside_initialized_count(
                    types,
                    raw_aliases,
                    &entry.slot,
                    storage,
                    initialized_count,
                    value_ty,
                )
        });
    }

    pub(super) fn weaken_slots_described_by_maybe_ranges_with_aliases(
        &mut self,
        raw_aliases: &RawCellAddressAliases,
    ) {
        for entry in &mut self.slots {
            let mut range_ty = None;
            let mut saw_range = false;
            for range in &self.maybe_initialized_ranges {
                if !place_covers_slot_with_aliases(&entry.slot, &range.storage, raw_aliases) {
                    continue;
                }
                saw_range = true;
                range_ty = merge_optional_range_value_ty(range_ty, range.value_ty);
            }
            if saw_range {
                entry.state = weaken_slot_state_with_maybe_range(entry.state, range_ty);
            }
        }
    }
}

impl CollectionSlotStateTable {
    fn initialized_range_state_with_aliases(
        &self,
        types: &TypeCtx,
        slot: &Place,
        raw_aliases: &RawCellAddressAliases,
    ) -> Option<CollectionSlotState> {
        self.initialized_ranges
            .iter()
            .find_map(|entry| {
                if entry.element_stride != crate::layout::storage_size_bytes(types, entry.value_ty)
                {
                    return None;
                }
                let initialized_count = raw_aliases.canonicalize_scalar(&entry.initialized_count);
                storage_aliases_for_place(&entry.storage, raw_aliases)
                    .into_iter()
                    .any(|storage| {
                        collection_slot_offset_is_inside_initialized_count(
                            types,
                            raw_aliases,
                            slot,
                            &storage,
                            &initialized_count,
                            entry.value_ty,
                        )
                    })
                    .then_some(CollectionSlotState::Initialized(entry.value_ty))
            })
            .or_else(|| {
                self.maybe_initialized_ranges.iter().find_map(|entry| {
                    if entry.element_stride
                        != crate::layout::storage_size_bytes(types, entry.value_ty)
                    {
                        return None;
                    }
                    let initialized_count =
                        raw_aliases.canonicalize_scalar(&entry.initialized_count);
                    storage_aliases_for_place(&entry.storage, raw_aliases)
                        .into_iter()
                        .any(|storage| {
                            collection_slot_offset_is_inside_initialized_count(
                                types,
                                raw_aliases,
                                slot,
                                &storage,
                                &initialized_count,
                                entry.value_ty,
                            )
                        })
                        .then_some(CollectionSlotState::MaybeInitialized(Some(entry.value_ty)))
                })
            })
    }
}

fn collection_slot_event_operation(
    event: CollectionSlotLifecycleEvent,
) -> CollectionSlotLifecycleOp {
    match event {
        CollectionSlotLifecycleEvent::InitializeEmpty { .. } => {
            CollectionSlotLifecycleOp::InitializeEmpty
        }
        CollectionSlotLifecycleEvent::BorrowRead { .. } => CollectionSlotLifecycleOp::BorrowRead,
        CollectionSlotLifecycleEvent::MoveOut { .. } => CollectionSlotLifecycleOp::MoveOut,
        CollectionSlotLifecycleEvent::ReplaceInitialized { .. } => {
            CollectionSlotLifecycleOp::ReplaceInitialized
        }
        CollectionSlotLifecycleEvent::DropInitialized { .. } => {
            CollectionSlotLifecycleOp::DropInitialized
        }
        CollectionSlotLifecycleEvent::StorageDealloc => CollectionSlotLifecycleOp::StorageDealloc,
    }
}

fn slot_matches_alias_candidates(
    slot: &Place,
    candidates: &[Place],
    raw_aliases: &RawCellAddressAliases,
) -> bool {
    let slot_candidates = raw_cell_place_alias_candidates(slot, raw_aliases);
    slot_candidates.iter().any(|slot_candidate| {
        candidates.iter().any(|candidate| {
            same_collection_slot_identity(slot_candidate, candidate)
                || raw_cell_places_equivalent(slot_candidate, candidate)
        })
    })
}

fn same_initialized_range_with_aliases(
    entry: &CollectionSlotInitializedRangeStateEntry,
    storage: &Place,
    initialized_count: &Place,
    value_ty: TypeId,
    element_stride: usize,
    raw_aliases: &RawCellAddressAliases,
) -> bool {
    entry.value_ty == value_ty
        && entry.element_stride == element_stride
        && same_collection_slot_identity(
            &raw_aliases.canonicalize_owner_cell_address(&entry.storage),
            storage,
        )
        && same_scalar_place_with_aliases(&entry.initialized_count, initialized_count, raw_aliases)
}

fn same_scalar_place_with_aliases(
    left: &Place,
    right: &Place,
    raw_aliases: &RawCellAddressAliases,
) -> bool {
    let left = raw_aliases.canonicalize_scalar(left);
    let right = raw_aliases.canonicalize_scalar(right);
    let result = same_collection_slot_identity(&left, &right)
        || raw_aliases
            .scalar_aliases_for(&left)
            .iter()
            .any(|alias| same_collection_slot_identity(alias, &right))
        || raw_aliases
            .scalar_aliases_for(&right)
            .iter()
            .any(|alias| same_collection_slot_identity(alias, &left))
        || matches!(
            (raw_aliases.i32_value(&left), raw_aliases.i32_value(&right)),
            (Some(left_value), Some(right_value)) if left_value == right_value
        )
        || raw_aliases.i32_relation_truth(&left, ResourceI32RelationOp::Eq, &right) == Some(true);
    result
}

fn slot_state_describes_range_value(state: CollectionSlotState, value_ty: TypeId) -> bool {
    match state {
        CollectionSlotState::Initialized(slot_ty)
        | CollectionSlotState::Moved(slot_ty)
        | CollectionSlotState::Dropped(slot_ty) => slot_ty == value_ty,
        CollectionSlotState::MaybeInitialized(Some(slot_ty)) => slot_ty == value_ty,
        CollectionSlotState::MaybeInitialized(None) => true,
        CollectionSlotState::Uninitialized
        | CollectionSlotState::Released
        | CollectionSlotState::MaybeReleased => false,
    }
}

fn weaken_slot_state_with_maybe_range(
    state: CollectionSlotState,
    range_ty: Option<TypeId>,
) -> CollectionSlotState {
    match state {
        CollectionSlotState::Initialized(slot_ty) => {
            CollectionSlotState::MaybeInitialized(merge_optional_range_value_ty(range_ty, slot_ty))
        }
        CollectionSlotState::MaybeInitialized(slot_ty) => {
            CollectionSlotState::MaybeInitialized(merge_optional_types(range_ty, slot_ty))
        }
        CollectionSlotState::Uninitialized
        | CollectionSlotState::Moved(_)
        | CollectionSlotState::Dropped(_)
        | CollectionSlotState::Released
        | CollectionSlotState::MaybeReleased => state,
    }
}

fn merge_optional_range_value_ty(existing: Option<TypeId>, value_ty: TypeId) -> Option<TypeId> {
    match existing {
        Some(existing) if existing == value_ty => Some(existing),
        Some(_) => None,
        None => Some(value_ty),
    }
}

fn merge_optional_types(left: Option<TypeId>, right: Option<TypeId>) -> Option<TypeId> {
    match (left, right) {
        (Some(left), Some(right)) if left == right => Some(left),
        _ => None,
    }
}
