use alloc::boxed::Box;
use alloc::vec::Vec;

use super::model::{BorrowKind, BorrowState, BorrowStateEntry, Place};
use super::place_utils::{
    place_suffix_after_prefix, place_with_suffix, places_overlap, push_unique_place, should_track,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct BorrowBinding {
    pub(super) token: Place,
    pub(super) source: Place,
    pub(super) kind: BorrowKind,
}

#[derive(Debug, Clone, Default)]
pub(super) struct BorrowTable {
    sources: Vec<BorrowStateEntry>,
    bindings: Vec<BorrowBinding>,
}

impl BorrowTable {
    pub(super) fn into_entries(self) -> Vec<BorrowStateEntry> {
        self.sources
    }

    pub(super) fn state(&self, place: &Place) -> BorrowState {
        self.sources
            .iter()
            .find(|entry| entry.place == *place)
            .map(|entry| entry.state.clone())
            .unwrap_or(BorrowState::Unborrowed)
    }

    pub(super) fn active_state_overlapping(&self, place: &Place) -> Option<BorrowState> {
        self.sources
            .iter()
            .filter(|entry| places_overlap(&entry.place, place))
            .map(|entry| entry.state.clone())
            .find(|state| {
                matches!(
                    state,
                    BorrowState::Shared { .. } | BorrowState::Unique { .. }
                )
            })
    }

    pub(super) fn unique_state_overlapping(&self, place: &Place) -> Option<BorrowState> {
        self.sources
            .iter()
            .filter(|entry| places_overlap(&entry.place, place))
            .map(|entry| entry.state.clone())
            .find(|state| matches!(state, BorrowState::Unique { .. }))
    }

    pub(super) fn add_shared(&mut self, source: &Place, token: &Place) {
        let next_count = match self.state(source) {
            BorrowState::Shared { count } => count + 1,
            _ => 1,
        };
        self.set_source(source, BorrowState::Shared { count: next_count });
        self.bindings.push(BorrowBinding {
            token: token.clone(),
            source: source.clone(),
            kind: BorrowKind::Shared,
        });
    }

    pub(super) fn add_unique(&mut self, source: &Place, token: &Place) {
        self.set_source(
            source,
            BorrowState::Unique {
                source: Box::new(source.clone()),
            },
        );
        self.bindings.push(BorrowBinding {
            token: token.clone(),
            source: source.clone(),
            kind: BorrowKind::Unique,
        });
    }

    pub(super) fn copy_or_move_token_tree(
        &mut self,
        source: &Place,
        output: &Place,
        copy_shared: bool,
    ) -> bool {
        let bindings = self
            .bindings
            .iter()
            .enumerate()
            .filter_map(|(index, binding)| {
                let suffix = place_suffix_after_prefix(&binding.token, source)?;
                let target = place_with_suffix(output, &suffix, binding.token.ty);
                Some((index, binding.clone(), target))
            })
            .collect::<Vec<_>>();
        if bindings.is_empty() {
            return false;
        }
        for (index, binding, target) in bindings {
            if copy_shared && binding.kind == BorrowKind::Shared {
                self.add_shared(&binding.source, &target);
            } else if let Some(slot) = self.bindings.get_mut(index) {
                slot.token = target;
            }
        }
        true
    }

    pub(super) fn release_token_tree(&mut self, token: &Place) -> bool {
        let mut released = false;
        let mut index = 0;
        while index < self.bindings.len() {
            if place_suffix_after_prefix(&self.bindings[index].token, token).is_none() {
                index += 1;
                continue;
            }
            let binding = self.bindings.remove(index);
            self.release_source(&binding.source, binding.kind);
            released = true;
        }
        released
    }

    pub(super) fn release_tokens_not_used_by<F>(&mut self, mut is_used: F)
    where
        F: FnMut(&BorrowBinding) -> bool,
    {
        let mut index = 0;
        while index < self.bindings.len() {
            if is_used(&self.bindings[index]) {
                index += 1;
                continue;
            }
            let binding = self.bindings.remove(index);
            self.release_source(&binding.source, binding.kind);
        }
    }

    pub(super) fn active_state_for_deref_access(&self, place: &Place) -> Option<BorrowState> {
        let deref_index = place
            .projections
            .iter()
            .position(|projection| matches!(projection, super::model::PlaceProjection::Deref))?;
        let mut token = place.clone();
        token.projections.truncate(deref_index);
        let binding = self.binding_by_root_projection(&token)?;
        let state = self.state(&binding.source);
        matches!(
            state,
            BorrowState::Shared { .. } | BorrowState::Unique { .. }
        )
        .then_some(state)
    }

    pub(super) fn bindings_overlapping_token(&self, place: &Place) -> Vec<BorrowBinding> {
        self.bindings
            .iter()
            .filter(|binding| places_overlap(&binding.token, place))
            .cloned()
            .collect()
    }

    pub(super) fn bindings_with_source_overlapping(&self, place: &Place) -> Vec<BorrowBinding> {
        self.bindings
            .iter()
            .filter(|binding| places_overlap(&binding.source, place))
            .cloned()
            .collect()
    }

    pub(super) fn merge_paths(paths: &[BorrowTable]) -> Self {
        let mut out = BorrowTable::default();
        let mut places = Vec::new();
        for path in paths {
            for entry in &path.sources {
                push_unique_place(&mut places, &entry.place);
            }
        }
        for place in places {
            let mut merged = BorrowState::Unborrowed;
            for path in paths {
                merged = merge_borrow_states(merged, path.state(&place));
            }
            out.set_source(&place, merged);
        }
        for path in paths {
            for binding in &path.bindings {
                if out.binding_index(&binding.token).is_none() {
                    out.bindings.push(binding.clone());
                }
            }
        }
        let sources = out.sources.clone();
        out.bindings.retain(|binding| {
            let state = sources
                .iter()
                .find(|entry| entry.place == binding.source)
                .map(|entry| entry.state.clone())
                .unwrap_or(BorrowState::Unborrowed);
            matches!(
                state,
                BorrowState::Shared { .. } | BorrowState::Unique { .. }
            )
        });
        out
    }

    fn release_source(&mut self, source: &Place, kind: BorrowKind) {
        match (kind, self.state(source)) {
            (BorrowKind::Shared, BorrowState::Shared { count }) if count > 1 => {
                self.set_source(source, BorrowState::Shared { count: count - 1 });
            }
            (BorrowKind::Shared, BorrowState::Shared { .. })
            | (BorrowKind::Unique, BorrowState::Unique { .. }) => {
                self.set_source(source, BorrowState::Released);
            }
            _ => {}
        }
    }

    fn set_source(&mut self, place: &Place, state: BorrowState) {
        if !should_track(place) {
            return;
        }
        if let Some(entry) = self.sources.iter_mut().find(|entry| entry.place == *place) {
            entry.state = state;
        } else {
            self.sources.push(BorrowStateEntry {
                place: place.clone(),
                state,
            });
        }
    }

    fn binding_index(&self, token: &Place) -> Option<usize> {
        self.bindings
            .iter()
            .position(|binding| binding.token == *token)
    }

    fn binding_by_root_projection(&self, token: &Place) -> Option<&BorrowBinding> {
        self.bindings.iter().find(|binding| {
            binding.token.root == token.root && binding.token.projections == token.projections
        })
    }
}

fn merge_borrow_states(left: BorrowState, right: BorrowState) -> BorrowState {
    if left == right {
        return left;
    }
    match (left, right) {
        (BorrowState::Unique { source }, _) | (_, BorrowState::Unique { source }) => {
            BorrowState::Unique { source }
        }
        (BorrowState::Shared { count: left_count }, BorrowState::Shared { count: right_count }) => {
            BorrowState::Shared {
                count: left_count.max(right_count),
            }
        }
        (BorrowState::Shared { count }, BorrowState::Unborrowed)
        | (BorrowState::Unborrowed, BorrowState::Shared { count })
        | (BorrowState::Shared { count }, BorrowState::Released)
        | (BorrowState::Released, BorrowState::Shared { count }) => BorrowState::Shared { count },
        (BorrowState::Released, BorrowState::Unborrowed)
        | (BorrowState::Unborrowed, BorrowState::Released)
        | (BorrowState::Released, BorrowState::Released) => BorrowState::Released,
        (BorrowState::Unborrowed, BorrowState::Unborrowed) => BorrowState::Unborrowed,
    }
}
