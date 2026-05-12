extern crate alloc;

use alloc::vec::Vec;

use super::model::{Place, PlaceProjection, StorageOrigin, StorageOriginEntry};
use super::place_utils::{
    place_suffix_after_prefix, place_with_suffix, push_unique_place, replace_place_prefix,
    should_track,
};

#[derive(Debug, Clone, Default)]
pub(super) struct StorageOriginTable {
    origins: Vec<StorageOriginEntry>,
    origin_sources: Vec<StorageOriginSourceEntry>,
}

#[derive(Debug, Clone)]
struct StorageOriginSourceEntry {
    place: Place,
    source: Place,
}

struct TranslatedStorageOrigin {
    place: Place,
    origin: StorageOrigin,
    source: Place,
}

impl StorageOriginTable {
    pub(super) fn origin(&self, place: &Place) -> Option<StorageOrigin> {
        self.origins
            .iter()
            .filter_map(|entry| {
                place_suffix_after_prefix(place, &entry.place)
                    .filter(|suffix| storage_origin_suffix_preserves_owner_identity(suffix))
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
                && place_suffix_after_prefix(&entry.place, place)
                    .is_some_and(|suffix| storage_origin_suffix_preserves_owner_identity(&suffix))
        })
    }

    pub(super) fn entries_under(&self, place: &Place) -> Vec<StorageOriginEntry> {
        let mut entries = Vec::new();
        if let Some(origin) = self.origin(place) {
            entries.push(StorageOriginEntry {
                place: place.clone(),
                origin,
            });
        }
        for entry in &self.origins {
            if entry.place == *place {
                continue;
            }
            if place_suffix_after_prefix(&entry.place, place)
                .is_some_and(|suffix| storage_origin_suffix_preserves_owner_identity(&suffix))
                && !entries.iter().any(|existing| existing.place == entry.place)
            {
                entries.push(entry.clone());
            }
        }
        entries
    }

    pub(super) fn origin_source(&self, place: &Place) -> Option<Place> {
        self.origin_sources
            .iter()
            .filter_map(|entry| {
                place_suffix_after_prefix(place, &entry.place)
                    .filter(|suffix| storage_origin_suffix_preserves_owner_identity(suffix))
                    .map(|suffix| {
                        (
                            place_with_suffix(&entry.source, &suffix, place.ty),
                            entry.place.projections.len(),
                        )
                    })
            })
            .max_by_key(|(_, projection_len)| *projection_len)
            .map(|(source, _)| source)
    }

    pub(super) fn has_origin_source_under(&self, place: &Place, source: &Place) -> bool {
        self.origin_sources.iter().any(|entry| {
            place_suffix_after_prefix(&entry.place, place)
                .is_some_and(|suffix| storage_origin_suffix_preserves_owner_identity(&suffix))
                && places_overlap_source(&entry.source, source)
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
        let origins = self.translated_origins_with_sources(source, target);
        self.clear(target);
        for entry in origins {
            self.set_origin(&entry.place, entry.origin);
            self.set_origin_source(&entry.place, entry.source);
        }
    }

    pub(super) fn move_origin(&mut self, source: &Place, target: &Place) {
        if source == target {
            return;
        }
        let origins = self.translated_origins_with_sources(source, target);
        self.clear(source);
        self.clear(target);
        for entry in origins {
            self.set_origin(&entry.place, entry.origin);
        }
    }

    pub(super) fn clear(&mut self, place: &Place) {
        self.origins
            .retain(|entry| place_suffix_after_prefix(&entry.place, place).is_none());
        self.origin_sources
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
                if let Some(source) = merge_origin_sources(paths, &place) {
                    out.set_origin_source(&place, source);
                }
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

    fn set_origin_source(&mut self, place: &Place, source: Place) {
        if !should_track(place) {
            return;
        }
        self.origin_sources
            .retain(|entry| place_suffix_after_prefix(&entry.place, place).is_none());
        self.origin_sources.push(StorageOriginSourceEntry {
            place: place.clone(),
            source,
        });
    }

    fn translated_origins_with_sources(
        &self,
        source: &Place,
        target: &Place,
    ) -> Vec<TranslatedStorageOrigin> {
        let mut origins = self
            .origins
            .iter()
            .filter_map(|entry| {
                let suffix = place_suffix_after_prefix(&entry.place, source)?;
                storage_origin_suffix_preserves_owner_identity(&suffix)
                    .then(|| replace_place_prefix(&entry.place, source, target))
                    .flatten()
                    .map(|place| {
                        let source = self
                            .origin_source(&entry.place)
                            .unwrap_or_else(|| entry.place.clone());
                        TranslatedStorageOrigin {
                            place,
                            origin: entry.origin,
                            source,
                        }
                    })
            })
            .collect::<Vec<_>>();
        if origins.is_empty() {
            if let Some(origin) = self.origin(source) {
                origins.push(TranslatedStorageOrigin {
                    place: target.clone(),
                    origin,
                    source: self.origin_source(source).unwrap_or_else(|| source.clone()),
                });
            }
        }
        origins
    }
}

fn storage_origin_suffix_preserves_owner_identity(suffix: &[PlaceProjection]) -> bool {
    !suffix
        .iter()
        .any(|projection| matches!(projection, PlaceProjection::Deref))
}

fn places_overlap_source(left: &Place, right: &Place) -> bool {
    left == right
        || place_suffix_after_prefix(left, right).is_some()
        || place_suffix_after_prefix(right, left).is_some()
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

fn merge_origin_sources(paths: &[StorageOriginTable], place: &Place) -> Option<Place> {
    let mut merged = None;
    for path in paths {
        if path.origin(place).is_none() {
            continue;
        }
        let source = path.origin_source(place)?;
        match &merged {
            Some(existing) if existing != &source => return None,
            Some(_) => {}
            None => merged = Some(source),
        }
    }
    merged
}
