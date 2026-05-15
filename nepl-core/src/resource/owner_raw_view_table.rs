use alloc::vec::Vec;

use super::model::Place;
use super::owner_raw_view_model::RawAddressViewOwnership;
use super::place_utils::{place_suffix_after_prefix, replace_place_prefix};

#[derive(Clone, Default)]
pub(super) struct RawAddressViewTable {
    entries: Vec<RawAddressViewEntry>,
}

#[derive(Clone)]
struct RawAddressViewEntry {
    place: Place,
    ownership: RawAddressViewOwnership,
}

impl RawAddressViewTable {
    pub(super) fn non_owning_entries(
        &self,
    ) -> impl Iterator<Item = (&Place, RawAddressViewOwnership)> {
        self.entries.iter().filter_map(|entry| {
            if entry.ownership.is_non_owning() {
                Some((&entry.place, entry.ownership))
            } else {
                None
            }
        })
    }

    pub(super) fn mark(&mut self, place: &Place) {
        self.mark_with(place, RawAddressViewOwnership::AddressView);
    }

    pub(super) fn mark_non_owning(&mut self, place: &Place) {
        self.mark_with(place, RawAddressViewOwnership::NonOwning);
    }

    pub(super) fn mark_non_owning_projection(&mut self, place: &Place) {
        self.mark_with(place, RawAddressViewOwnership::NonOwningProjection);
    }

    pub(super) fn copy(&mut self, source: &Place, target: &Place) {
        let copied = self
            .entries
            .iter()
            .filter_map(|entry| {
                replace_place_prefix(&entry.place, source, target).map(|place| {
                    RawAddressViewEntry {
                        place,
                        ownership: entry.ownership,
                    }
                })
            })
            .collect::<Vec<_>>();
        self.clear(target);
        for entry in copied {
            push_unique_raw_view_entry(&mut self.entries, entry);
        }
    }

    pub(super) fn copy_non_owning(&mut self, source: &Place, target: &Place) {
        let copied = self
            .entries
            .iter()
            .filter(|entry| entry.ownership.is_non_owning())
            .filter_map(|entry| {
                replace_place_prefix(&entry.place, source, target).map(|place| {
                    RawAddressViewEntry {
                        place,
                        ownership: entry.ownership,
                    }
                })
            })
            .collect::<Vec<_>>();
        self.clear(target);
        for entry in copied {
            push_unique_raw_view_entry(&mut self.entries, entry);
        }
    }

    pub(super) fn clear(&mut self, place: &Place) {
        self.entries
            .retain(|entry| place_suffix_after_prefix(&entry.place, place).is_none());
    }

    pub(super) fn contains(&self, place: &Place) -> bool {
        self.entries
            .iter()
            .any(|entry| same_raw_view_place(&entry.place, place))
    }

    pub(super) fn contains_non_owning(&self, place: &Place) -> bool {
        self.entries.iter().any(|entry| {
            entry.ownership.is_non_owning() && same_raw_view_place(&entry.place, place)
        })
    }

    pub(super) fn contains_non_owning_projection(&self, place: &Place) -> bool {
        self.entries.iter().any(|entry| {
            matches!(
                entry.ownership,
                RawAddressViewOwnership::NonOwningProjection
            ) && same_raw_view_place(&entry.place, place)
        })
    }

    pub(super) fn contains_non_owning_under(&self, prefix: &Place) -> bool {
        self.entries.iter().any(|entry| {
            entry.ownership.is_non_owning()
                && place_suffix_after_prefix(&entry.place, prefix).is_some()
        })
    }

    pub(super) fn contains_under(&self, prefix: &Place) -> bool {
        self.entries
            .iter()
            .any(|entry| place_suffix_after_prefix(&entry.place, prefix).is_some())
    }

    pub(super) fn merge_paths(paths: &[RawAddressViewTable]) -> Self {
        let mut out = RawAddressViewTable::default();
        for path in paths {
            for entry in &path.entries {
                push_unique_raw_view_entry(&mut out.entries, entry.clone());
            }
        }
        out
    }

    fn mark_with(&mut self, place: &Place, ownership: RawAddressViewOwnership) {
        self.clear(place);
        self.entries.push(RawAddressViewEntry {
            place: place.clone(),
            ownership,
        });
    }
}

fn same_raw_view_place(left: &Place, right: &Place) -> bool {
    left.root == right.root && left.projections == right.projections
}

fn push_unique_raw_view_entry(entries: &mut Vec<RawAddressViewEntry>, entry: RawAddressViewEntry) {
    if let Some(existing) = entries
        .iter_mut()
        .find(|existing| same_raw_view_place(&existing.place, &entry.place))
    {
        if entry.ownership.priority() >= existing.ownership.priority() {
            existing.ownership = entry.ownership;
            existing.place = entry.place;
        }
    } else {
        entries.push(entry);
    }
}
