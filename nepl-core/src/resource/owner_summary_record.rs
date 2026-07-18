use alloc::collections::{BTreeMap, BTreeSet};
use alloc::vec::Vec;

use crate::types::TypeId;

use super::model::{Place, PlaceProjection, StorageId};
use super::owner_extent::merge_owner_extent_summaries;
use super::place_utils::push_unique_usize;
use super::summary::{
    OwnerExtentSummary, OwnerParameterReturnExtent, OwnerProjectionMarker,
    OwnerProjectionReturnSummary, OwnerProjectionSource,
};

pub(super) fn record_projection_owner_return(
    projection_returns: &mut OwnerProjectionReturnRecorder,
    suffix: Vec<PlaceProjection>,
    ty: TypeId,
    storage: StorageId,
    fresh_extent: OwnerExtentSummary,
    parameter_storage_sources: &[OwnerParameterStorageSource],
    returned_sources: &mut OwnerProjectionSourceRecorder,
) {
    let entry_index = projection_return_entry_index(projection_returns, suffix, ty);
    if let Some(source) = owner_source_for_storage(storage, parameter_storage_sources) {
        projection_returns.record_owner_source(
            entry_index,
            returned_sources,
            source,
            fresh_extent,
        );
    } else {
        if projection_returns[entry_index].returns_fresh_owner {
            projection_returns[entry_index].returns_fresh_owner_extent =
                merge_owner_extent_summaries(
                    projection_returns[entry_index]
                        .returns_fresh_owner_extent
                        .clone(),
                    fresh_extent,
                );
        } else {
            projection_returns[entry_index].returns_fresh_owner = true;
            projection_returns[entry_index].returns_fresh_owner_extent = fresh_extent;
        }
    }
}

pub(super) fn record_projection_maybe_owner_return(
    projection_returns: &mut OwnerProjectionReturnRecorder,
    suffix: Vec<PlaceProjection>,
    ty: TypeId,
) {
    let entry_index = projection_return_entry_index(projection_returns, suffix, ty);
    projection_returns[entry_index].returns_maybe_owner = true;
}

fn projection_return_entry_index(
    projection_returns: &mut OwnerProjectionReturnRecorder,
    suffix: Vec<PlaceProjection>,
    ty: TypeId,
) -> usize {
    if let Some(index) = projection_returns
        .indices_by_suffix
        .get(&suffix)
        .and_then(|indices| indices.get(&ty))
        .copied()
    {
        return index;
    }
    let index = projection_returns.entries.len();
    projection_returns
        .indices_by_suffix
        .entry(suffix.clone())
        .or_default()
        .insert(ty, index);
    projection_returns.entries.push(OwnerProjectionReturnSummary {
        suffix,
        ty,
        parameter_indices: Vec::new(),
        parameter_sources: Vec::new(),
        parameter_return_extents: Vec::new(),
        returns_fresh_owner: false,
        returns_fresh_owner_extent: OwnerExtentSummary::Unknown,
        returns_maybe_owner: false,
    });
    projection_returns
        .parameter_source_memberships
        .push(BTreeSet::new());
    projection_returns
        .return_extent_indices
        .push(BTreeMap::new());
    index
}

#[derive(Default)]
pub(super) struct OwnerProjectionReturnRecorder {
    entries: Vec<OwnerProjectionReturnSummary>,
    indices_by_suffix: BTreeMap<Vec<PlaceProjection>, BTreeMap<TypeId, usize>>,
    parameter_source_memberships: Vec<BTreeSet<OwnerProjectionSource>>,
    return_extent_indices: Vec<BTreeMap<OwnerProjectionSource, usize>>,
}

impl OwnerProjectionReturnRecorder {
    pub(super) fn into_entries(self) -> Vec<OwnerProjectionReturnSummary> {
        self.entries
    }

    fn record_owner_source(
        &mut self,
        entry_index: usize,
        returned_sources: &mut OwnerProjectionSourceRecorder,
        source: &OwnerProjectionSource,
        returned_extent: OwnerExtentSummary,
    ) {
        let summary = &mut self.entries[entry_index];
        if source.suffix.is_empty() {
            push_unique_usize(&mut summary.parameter_indices, source.parameter_index);
        } else if self.parameter_source_memberships[entry_index].insert(source.clone()) {
            summary.parameter_sources.push(source.clone());
        }
        if let Some(extent_index) = self.return_extent_indices[entry_index].get(source).copied() {
            let extent = &mut summary.parameter_return_extents[extent_index].extent;
            *extent = merge_owner_extent_summaries(extent.clone(), returned_extent);
        } else {
            let extent_index = summary.parameter_return_extents.len();
            self.return_extent_indices[entry_index].insert(source.clone(), extent_index);
            summary
                .parameter_return_extents
                .push(OwnerParameterReturnExtent {
                    source: source.clone(),
                    extent: returned_extent,
                });
        }
        returned_sources.push_unique(source);
    }
}

impl core::ops::Index<usize> for OwnerProjectionReturnRecorder {
    type Output = OwnerProjectionReturnSummary;

    fn index(&self, index: usize) -> &Self::Output {
        &self.entries[index]
    }
}

impl core::ops::IndexMut<usize> for OwnerProjectionReturnRecorder {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.entries[index]
    }
}

pub(super) fn record_projection_marker(
    projection_markers: &mut Vec<OwnerProjectionMarker>,
    suffix: Vec<PlaceProjection>,
    ty: TypeId,
) {
    if !projection_markers
        .iter()
        .any(|marker| marker.suffix == suffix && marker.ty == ty)
    {
        projection_markers.push(OwnerProjectionMarker { suffix, ty });
    }
}

pub(super) fn record_root_owner_return(
    parameter_indices: &mut Vec<usize>,
    parameter_sources: &mut Vec<OwnerProjectionSource>,
    parameter_return_extents: &mut Vec<OwnerParameterReturnExtent>,
    returned_sources: &mut OwnerProjectionSourceRecorder,
    source: &OwnerProjectionSource,
    returned_extent: OwnerExtentSummary,
) {
    if source.suffix.is_empty() {
        push_unique_usize(parameter_indices, source.parameter_index);
    } else {
        push_unique_owner_projection_source(parameter_sources, source);
    }
    push_or_merge_parameter_return_extent(
        parameter_return_extents,
        OwnerParameterReturnExtent {
            source: source.clone(),
            extent: returned_extent,
        },
    );
    returned_sources.push_unique(source);
}

pub(super) fn owner_source_for_storage<'a>(
    storage: StorageId,
    parameter_storage_sources: &'a [OwnerParameterStorageSource],
) -> Option<&'a OwnerProjectionSource> {
    parameter_storage_sources
        .iter()
        .find_map(|source| (source.storage == storage).then_some(&source.source))
}

pub(super) fn push_unique_owner_projection_source(
    sources: &mut Vec<OwnerProjectionSource>,
    source: &OwnerProjectionSource,
) {
    if !sources.iter().any(|existing| existing == source) {
        sources.push(source.clone());
    }
}

pub(super) struct OwnerParameterStorageSource {
    pub(super) storage: StorageId,
    pub(super) source: OwnerProjectionSource,
    pub(super) place: Place,
}

pub(super) struct OwnerParameterConditionSource {
    pub(super) source: OwnerProjectionSource,
    pub(super) place: Place,
}

pub(super) fn parameter_return_extent_for_source<'a>(
    extents: &'a [OwnerParameterReturnExtent],
    source: &OwnerProjectionSource,
) -> Option<&'a OwnerExtentSummary> {
    extents
        .iter()
        .find_map(|entry| (&entry.source == source).then_some(&entry.extent))
}

pub(super) fn push_or_merge_parameter_return_extent(
    out: &mut Vec<OwnerParameterReturnExtent>,
    entry: OwnerParameterReturnExtent,
) {
    if let Some(existing) = out
        .iter_mut()
        .find(|existing| existing.source == entry.source)
    {
        existing.extent = merge_owner_extent_summaries(existing.extent.clone(), entry.extent);
        return;
    }
    out.push(entry);
}

#[derive(Default)]
pub(super) struct OwnerProjectionSourceRecorder {
    entries: Vec<OwnerProjectionSource>,
    membership: BTreeSet<OwnerProjectionSource>,
}

impl OwnerProjectionSourceRecorder {
    pub(super) fn push_unique(&mut self, source: &OwnerProjectionSource) {
        if self.membership.insert(source.clone()) {
            self.entries.push(source.clone());
        }
    }

    pub(super) fn into_entries(self) -> Vec<OwnerProjectionSource> {
        self.entries
    }
}

#[cfg(test)]
mod tests {
    use alloc::vec;
    use alloc::vec::Vec;

    use crate::types::TypeId;

    use super::super::model::{Place, PlaceProjection, StorageId};
    use super::super::summary::{OwnerExtentSummary, OwnerProjectionSource};
    use super::{
        record_projection_maybe_owner_return, record_projection_owner_return,
        OwnerParameterStorageSource, OwnerProjectionReturnRecorder, OwnerProjectionSourceRecorder,
    };

    #[test]
    fn projection_return_records_preserve_first_seen_order_and_merge_duplicates() {
        let field_one = vec![PlaceProjection::Field {
            index: 1,
            offset_bytes: 4,
        }];
        let field_zero = vec![PlaceProjection::Field {
            index: 0,
            offset_bytes: 0,
        }];
        let mut returns = OwnerProjectionReturnRecorder::default();

        record_projection_maybe_owner_return(&mut returns, field_one.clone(), TypeId(2));
        record_projection_maybe_owner_return(&mut returns, field_zero.clone(), TypeId(3));
        record_projection_maybe_owner_return(&mut returns, field_zero.clone(), TypeId(1));
        record_projection_maybe_owner_return(&mut returns, field_one.clone(), TypeId(2));

        let returns = returns.into_entries();
        assert_eq!(returns.len(), 3);
        assert_eq!(returns[0].suffix, field_one);
        assert_eq!(returns[0].ty, TypeId(2));
        assert_eq!(returns[1].suffix, field_zero);
        assert_eq!(returns[1].ty, TypeId(3));
        assert_eq!(returns[2].ty, TypeId(1));
        assert!(returns.iter().all(|entry| entry.returns_maybe_owner));
    }

    #[test]
    fn projection_return_recorder_indexes_sources_and_extents_per_entry() {
        let suffix = vec![PlaceProjection::Field {
            index: 0,
            offset_bytes: 0,
        }];
        let source = OwnerProjectionSource {
            parameter_index: 0,
            suffix: suffix.clone(),
            ty: TypeId(4),
        };
        let storage_sources = vec![OwnerParameterStorageSource {
            storage: StorageId(7),
            source: source.clone(),
            place: Place::local("parameter".into(), TypeId(4)),
        }];
        let mut returned_sources = OwnerProjectionSourceRecorder::default();
        let mut returns = OwnerProjectionReturnRecorder::default();

        record_projection_owner_return(
            &mut returns,
            suffix.clone(),
            TypeId(4),
            StorageId(7),
            OwnerExtentSummary::RegionTokenSize,
            &storage_sources,
            &mut returned_sources,
        );
        record_projection_owner_return(
            &mut returns,
            suffix,
            TypeId(4),
            StorageId(7),
            OwnerExtentSummary::Unknown,
            &storage_sources,
            &mut returned_sources,
        );
        record_projection_maybe_owner_return(&mut returns, Vec::new(), TypeId(5));

        let returns = returns.into_entries();
        assert_eq!(returns.len(), 2);
        assert_eq!(returns[0].parameter_sources, vec![source.clone()]);
        assert_eq!(returns[0].parameter_return_extents.len(), 1);
        assert_eq!(returns[0].parameter_return_extents[0].source, source);
        assert_eq!(
            returns[0].parameter_return_extents[0].extent,
            OwnerExtentSummary::Unknown
        );
        assert_eq!(returned_sources.into_entries().len(), 1);
        assert!(returns[1].returns_maybe_owner);
    }

    #[test]
    fn projection_source_recorder_preserves_first_seen_order() {
        let first = OwnerProjectionSource {
            parameter_index: 1,
            suffix: Vec::new(),
            ty: TypeId(1),
        };
        let second = OwnerProjectionSource {
            parameter_index: 2,
            suffix: Vec::new(),
            ty: TypeId(2),
        };
        let mut sources = OwnerProjectionSourceRecorder::default();

        sources.push_unique(&first);
        sources.push_unique(&second);
        sources.push_unique(&first);

        assert_eq!(sources.into_entries(), vec![first, second]);
    }
}
