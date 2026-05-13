use alloc::vec::Vec;

use super::model::{OwnerStorageExtent, Place, ResourceConditionFact};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct PendingRawRealloc {
    pub(super) origin: Place,
    pub(super) source: Place,
    pub(super) result: Place,
    pub(super) new_extent: OwnerStorageExtent,
}

#[derive(Debug, Clone, Default)]
pub(super) struct PendingRawReallocs {
    entries: Vec<PendingRawRealloc>,
}

impl PendingRawReallocs {
    pub(super) fn mark(&mut self, source: &Place, result: &Place, new_extent: OwnerStorageExtent) {
        self.remove_origin(result);
        self.entries.push(PendingRawRealloc {
            origin: result.clone(),
            source: source.clone(),
            result: result.clone(),
            new_extent,
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
            })
            .collect::<Vec<_>>();
        self.remove_result(target);
        for entry in copies {
            self.push_unique_entry(entry);
        }
    }

    pub(super) fn clear_result(&mut self, result: &Place) {
        self.remove_result(result);
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

    pub(super) fn merge_paths(paths: &[PendingRawReallocs]) -> Self {
        let mut out = PendingRawReallocs::default();
        for path in paths {
            for entry in &path.entries {
                out.push_unique_entry(entry.clone());
            }
        }
        out
    }

    fn remove_origin(&mut self, origin: &Place) {
        self.entries.retain(|entry| entry.origin != *origin);
    }

    fn remove_result(&mut self, result: &Place) {
        self.entries.retain(|entry| entry.result != *result);
    }

    fn push_unique_entry(&mut self, entry: PendingRawRealloc) {
        if self.entries.iter().any(|existing| existing == &entry) {
            return;
        }
        self.entries.push(entry);
    }
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
