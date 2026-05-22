use alloc::vec::Vec;

use super::model::{OwnerStorageExtent, Place, ResourceConditionFact};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct PendingRawRealloc {
    pub(super) origin: Place,
    pub(super) source: Place,
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
        result: &Place,
        new_extent: OwnerStorageExtent,
        collection_managed_non_copy_cells: Vec<Place>,
    ) {
        self.remove_origin(result);
        self.remove_certified_result(result);
        self.entries.push(PendingRawRealloc {
            origin: result.clone(),
            source: source.clone(),
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
            .filter(|entry| entry.result == *source)
            .map(|entry| PendingRawRealloc {
                origin: entry.origin.clone(),
                source: entry.source.clone(),
                result: target.clone(),
                new_extent: entry.new_extent.clone(),
                collection_managed_non_copy_cells: entry.collection_managed_non_copy_cells.clone(),
            })
            .collect::<Vec<_>>();
        let certified_copies = self
            .certified_relocations
            .iter()
            .filter(|entry| entry.result == *source)
            .map(|entry| CertifiedRawStorageRelocation {
                source: entry.source.clone(),
                result: target.clone(),
            })
            .collect::<Vec<_>>();
        self.remove_result(target);
        for entry in copies {
            self.push_unique_entry(entry);
        }
        for entry in certified_copies {
            self.push_unique_certified_relocation(entry);
        }
    }

    pub(super) fn clear_result(&mut self, result: &Place) {
        self.remove_result(result);
        self.remove_certified_result(result);
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

    fn remove_result(&mut self, result: &Place) {
        self.entries.retain(|entry| entry.result != *result);
    }

    fn remove_certified_result(&mut self, result: &Place) {
        self.certified_relocations
            .retain(|entry| entry.result != *result);
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
