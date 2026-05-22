use alloc::vec::Vec;

use super::model::{OwnerStorageExtent, Place, ResourceConditionFact};
use super::place_utils::{place_suffix_after_prefix, replace_place_prefix};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct PendingRawRealloc {
    pub(super) origin: Place,
    pub(super) source: Place,
    pub(super) storage_source: Place,
    pub(super) result: Place,
    pub(super) new_extent: OwnerStorageExtent,
    pub(super) collection_managed_non_copy_cells: Vec<Place>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CertifiedRawStorageRelocation {
    source: Place,
    result: Place,
}

#[derive(Debug, Clone, Default)]
pub(super) struct PendingRawReallocs {
    entries: Vec<PendingRawRealloc>,
    certified_relocations: Vec<CertifiedRawStorageRelocation>,
    certified_releases: Vec<Place>,
}

impl PendingRawReallocs {
    pub(super) fn mark(
        &mut self,
        source: &Place,
        storage_source: &Place,
        result: &Place,
        new_extent: OwnerStorageExtent,
        collection_managed_non_copy_cells: Vec<Place>,
    ) {
        self.remove_origin(result);
        self.remove_result_at_or_below(result);
        self.entries.push(PendingRawRealloc {
            origin: result.clone(),
            source: source.clone(),
            storage_source: storage_source.clone(),
            result: result.clone(),
            new_extent,
            collection_managed_non_copy_cells,
        });
    }

    pub(super) fn copy_result(&mut self, source: &Place, target: &Place) {
        if source == target {
            return;
        }
        let copies = self
            .entries
            .iter()
            .filter_map(|entry| {
                let result = replace_place_prefix(&entry.result, source, target)?;
                Some(PendingRawRealloc {
                    origin: entry.origin.clone(),
                    source: entry.source.clone(),
                    storage_source: entry.storage_source.clone(),
                    result,
                    new_extent: entry.new_extent.clone(),
                    collection_managed_non_copy_cells: entry
                        .collection_managed_non_copy_cells
                        .clone(),
                })
            })
            .collect::<Vec<_>>();
        let certified_copies = self
            .certified_relocations
            .iter()
            .filter_map(|entry| {
                let result = replace_place_prefix(&entry.result, source, target)?;
                Some(CertifiedRawStorageRelocation {
                    source: entry.source.clone(),
                    result,
                })
            })
            .collect::<Vec<_>>();
        self.remove_result_at_or_below(target);
        for entry in copies {
            self.push_unique_entry(entry);
        }
        for entry in certified_copies {
            self.push_unique_certified_relocation(entry);
        }
    }

    pub(super) fn clear_result(&mut self, result: &Place) {
        self.remove_result_at_or_below(result);
    }

    pub(super) fn take_for_result(&mut self, result: &Place) -> Option<PendingRawRealloc> {
        let entry = self
            .entries
            .iter()
            .find(|entry| entry.result == *result)
            .cloned()?;
        self.remove_origin(&entry.origin);
        Some(entry)
    }

    pub(super) fn certify_success(&mut self, source: &Place, result: &Place) {
        self.push_unique_certified_relocation(CertifiedRawStorageRelocation {
            source: source.clone(),
            result: result.clone(),
        });
    }

    pub(super) fn certified_storage_relocation_available(
        &self,
        source: &Place,
        result: &Place,
    ) -> bool {
        self.certified_relocations
            .iter()
            .any(|entry| entry.source == *source && entry.result == *result)
    }

    pub(super) fn consume_certified_storage_relocation(
        &mut self,
        source: &Place,
        result: &Place,
    ) -> bool {
        let Some(index) = self
            .certified_relocations
            .iter()
            .position(|entry| entry.source == *source && entry.result == *result)
        else {
            return false;
        };
        self.certified_relocations.remove(index);
        true
    }

    pub(super) fn certify_release(&mut self, storage: &Place) {
        self.push_unique_certified_release(storage);
    }

    pub(super) fn certified_storage_release_available(&self, storage: &Place) -> bool {
        self.certified_releases
            .iter()
            .any(|released| released == storage)
    }

    pub(super) fn consume_certified_storage_release(&mut self, storage: &Place) -> bool {
        let Some(index) = self
            .certified_releases
            .iter()
            .position(|released| released == storage)
        else {
            return false;
        };
        self.certified_releases.remove(index);
        true
    }

    pub(super) fn merge_paths(paths: &[PendingRawReallocs]) -> Self {
        let mut out = PendingRawReallocs::default();
        for path in paths {
            for entry in &path.entries {
                out.push_unique_entry(entry.clone());
            }
        }
        out.certified_relocations = merge_certified_relocations(paths);
        out.certified_releases = merge_certified_releases(paths);
        out
    }

    fn remove_origin(&mut self, origin: &Place) {
        self.entries.retain(|entry| entry.origin != *origin);
    }

    fn remove_result_at_or_below(&mut self, result: &Place) {
        self.entries
            .retain(|entry| !place_is_at_or_below(&entry.result, result));
        self.certified_relocations
            .retain(|entry| !place_is_at_or_below(&entry.result, result));
    }

    fn push_unique_entry(&mut self, entry: PendingRawRealloc) {
        if self.entries.iter().any(|existing| existing == &entry) {
            return;
        }
        self.entries.push(entry);
    }

    fn push_unique_certified_relocation(&mut self, entry: CertifiedRawStorageRelocation) {
        if self
            .certified_relocations
            .iter()
            .any(|existing| existing == &entry)
        {
            return;
        }
        self.certified_relocations.push(entry);
    }

    fn push_unique_certified_release(&mut self, storage: &Place) {
        if self
            .certified_releases
            .iter()
            .any(|existing| existing == storage)
        {
            return;
        }
        self.certified_releases.push(storage.clone());
    }
}

fn place_is_at_or_below(place: &Place, prefix: &Place) -> bool {
    place_suffix_after_prefix(place, prefix).is_some()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resource::model::PlaceProjection;
    use crate::types::TypeId;
    use alloc::string::ToString;

    fn field0(base: &Place) -> Place {
        base.clone().with_projection(
            PlaceProjection::Field {
                index: 0,
                offset_bytes: 0,
            },
            TypeId(1),
        )
    }

    #[test]
    fn copy_result_preserves_certified_relocation_through_aggregate_field_move() {
        let old_storage = field0(&Place::local("old_region".to_string(), TypeId(2)));
        let realloc_result = Place::local("next_raw".to_string(), TypeId(1));
        let region_tmp = Place::local("region_tmp".to_string(), TypeId(2));
        let grown = Place::local("grown".to_string(), TypeId(2));
        let mut pending = PendingRawReallocs::default();

        pending.certify_success(&old_storage, &realloc_result);
        pending.copy_result(&realloc_result, &field0(&region_tmp));
        pending.copy_result(&region_tmp, &grown);

        assert!(pending.certified_storage_relocation_available(&old_storage, &field0(&grown)));
    }

    #[test]
    fn take_for_result_preserves_distinct_raw_source_and_storage_source() {
        let raw_source = Place::local("old_raw_read".to_string(), TypeId(1));
        let storage_source = field0(&Place::local("old_region".to_string(), TypeId(2)));
        let raw_result = Place::local("realloc_result".to_string(), TypeId(1));
        let next_raw = Place::local("next_raw".to_string(), TypeId(1));
        let mut pending = PendingRawReallocs::default();

        pending.mark(
            &raw_source,
            &storage_source,
            &raw_result,
            OwnerStorageExtent::Unknown,
            Vec::new(),
        );
        pending.copy_result(&raw_result, &next_raw);
        let copied = pending.take_for_result(&next_raw).unwrap();

        assert_eq!(copied.source, raw_source);
        assert_eq!(copied.storage_source, storage_source);
    }

    #[test]
    fn clear_result_removes_projected_relocation_under_aggregate() {
        let old_storage = field0(&Place::local("old_region".to_string(), TypeId(2)));
        let grown = Place::local("grown".to_string(), TypeId(2));
        let mut pending = PendingRawReallocs::default();

        pending.certify_success(&old_storage, &field0(&grown));
        pending.clear_result(&grown);

        assert!(!pending.certified_storage_relocation_available(&old_storage, &field0(&grown)));
    }
}

fn merge_certified_relocations(paths: &[PendingRawReallocs]) -> Vec<CertifiedRawStorageRelocation> {
    let Some((first, rest)) = paths.split_first() else {
        return Vec::new();
    };
    first
        .certified_relocations
        .iter()
        .filter(|entry| {
            rest.iter()
                .all(|path| path.certified_relocations.contains(entry))
        })
        .cloned()
        .collect()
}

fn merge_certified_releases(paths: &[PendingRawReallocs]) -> Vec<Place> {
    let Some((first, rest)) = paths.split_first() else {
        return Vec::new();
    };
    first
        .certified_releases
        .iter()
        .filter(|storage| {
            rest.iter()
                .all(|path| path.certified_releases.contains(storage))
        })
        .cloned()
        .collect()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum RawReallocConditionOutcome {
    Success,
    Failure,
}

pub(super) fn raw_realloc_condition_outcome(
    fact: &ResourceConditionFact,
    truthy_path: bool,
) -> Option<(&Place, RawReallocConditionOutcome)> {
    match (fact, truthy_path) {
        (ResourceConditionFact::EqZero { place }, true)
        | (ResourceConditionFact::NeZero { place }, false)
        | (ResourceConditionFact::Positive { place }, false)
        | (ResourceConditionFact::NonPositive { place }, true)
        | (ResourceConditionFact::Negative { place }, true)
        | (ResourceConditionFact::NonNegative { place }, false) => {
            Some((place, RawReallocConditionOutcome::Failure))
        }
        (ResourceConditionFact::EqZero { place }, false)
        | (ResourceConditionFact::NeZero { place }, true)
        | (ResourceConditionFact::Positive { place }, true)
        | (ResourceConditionFact::NonPositive { place }, false) => {
            Some((place, RawReallocConditionOutcome::Success))
        }
        (ResourceConditionFact::Negative { .. }, false)
        | (ResourceConditionFact::NonNegative { .. }, true)
        | (ResourceConditionFact::I32Relation { .. }, _)
        | (ResourceConditionFact::Any(_), _)
        | (ResourceConditionFact::All(_), _) => None,
    }
}
