extern crate alloc;

use alloc::vec::Vec;

use super::model::{Place, StorageOrigin, StorageOriginEntry};
use super::place_utils::{
    place_suffix_after_prefix, push_unique_place, replace_place_prefix, should_track,
};

#[derive(Debug, Clone, Default)]
pub(super) struct StorageOriginTable {
    origins: Vec<StorageOriginEntry>,
}

impl StorageOriginTable {
    pub(super) fn origin(&self, place: &Place) -> Option<StorageOrigin> {
        self.origins
            .iter()
            .filter_map(|entry| {
                place_suffix_after_prefix(place, &entry.place)
                    .map(|_| (entry.origin, entry.place.projections.len()))
            })
            .max_by_key(|(_, projection_len)| *projection_len)
            .map(|(origin, _)| origin)
    }

    pub(super) fn expects_owned(&self, place: &Place) -> bool {
        matches!(self.origin(place), Some(StorageOrigin::Owned))
    }

    pub(super) fn expects_owned_under(&self, place: &Place) -> bool {
        self.origins.iter().any(|entry| {
            matches!(entry.origin, StorageOrigin::Owned)
                && place_suffix_after_prefix(&entry.place, place).is_some()
        })
    }

    pub(super) fn mark_owned(&mut self, place: &Place) {
        self.set_origin(place, StorageOrigin::Owned);
    }

    pub(super) fn mark_origin(&mut self, place: &Place, origin: StorageOrigin) {
        self.set_origin(place, origin);
    }

    pub(super) fn copy_origin(&mut self, source: &Place, target: &Place) {
        if source == target {
            return;
        }
        let origins = self.translated_origins(source, target);
        self.clear(target);
        for entry in origins {
            self.set_origin(&entry.place, entry.origin);
        }
    }

    pub(super) fn move_origin(&mut self, source: &Place, target: &Place) {
        if source == target {
            return;
        }
        let origins = self.translated_origins(source, target);
        self.clear(source);
        self.clear(target);
        for entry in origins {
            self.set_origin(&entry.place, entry.origin);
        }
    }

    pub(super) fn clear(&mut self, place: &Place) {
        self.origins
            .retain(|entry| place_suffix_after_prefix(&entry.place, place).is_none());
    }

    pub(super) fn merge_paths(paths: &[StorageOriginTable]) -> Self {
        let mut places = Vec::new();
        for path in paths {
            for entry in &path.origins {
                push_unique_place(&mut places, &entry.place);
            }
        }

        let mut out = StorageOriginTable::default();
        for place in places {
            let origin = paths.iter().fold(None, |left, path| {
                merge_storage_origins(left, path.origin(&place))
            });
            if let Some(origin) = origin {
                out.set_origin(&place, origin);
            }
        }
        out
    }

    fn set_origin(&mut self, place: &Place, origin: StorageOrigin) {
        if !should_track(place) {
            return;
        }
        self.clear(place);
        self.origins.push(StorageOriginEntry {
            place: place.clone(),
            origin,
        });
    }

    fn translated_origins(&self, source: &Place, target: &Place) -> Vec<StorageOriginEntry> {
        let mut origins = self
            .origins
            .iter()
            .filter_map(|entry| {
                replace_place_prefix(&entry.place, source, target).map(|place| StorageOriginEntry {
                    place,
                    origin: entry.origin,
                })
            })
            .collect::<Vec<_>>();
        if origins.is_empty() {
            if let Some(origin) = self.origin(source) {
                origins.push(StorageOriginEntry {
                    place: target.clone(),
                    origin,
                });
            }
        }
        origins
    }
}

fn merge_storage_origins(
    left: Option<StorageOrigin>,
    right: Option<StorageOrigin>,
) -> Option<StorageOrigin> {
    match (left, right) {
        (Some(StorageOrigin::Owned), _) | (_, Some(StorageOrigin::Owned)) => {
            Some(StorageOrigin::Owned)
        }
        (Some(StorageOrigin::Internal), _) | (_, Some(StorageOrigin::Internal)) => {
            Some(StorageOrigin::Internal)
        }
        (Some(StorageOrigin::Unmanaged), _) | (_, Some(StorageOrigin::Unmanaged)) => {
            Some(StorageOrigin::Unmanaged)
        }
        (None, None) => None,
    }
}
