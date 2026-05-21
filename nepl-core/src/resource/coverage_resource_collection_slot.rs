extern crate alloc;

use alloc::vec::Vec;

use crate::span::Span;

use super::coverage::{ResourceCoverageCounts, ResourceCoverageDiagnostic};
use super::coverage_operation::ResourceCoveragePlaceOperation as CoveragePlaceOp;
use super::coverage_resource_place::{resource_alias_place_coverage, resource_place_coverage};
use super::model::Place;

pub(super) fn collection_slot_lifecycle_coverage(
    function: &str,
    target: &Place,
    span: Span,
    counts: &mut ResourceCoverageCounts,
    diagnostics: &mut Vec<ResourceCoverageDiagnostic>,
) {
    counts.collection_slot_lifecycle_ops += 1;
    resource_alias_place_coverage(
        function,
        CoveragePlaceOp::CollectionSlotLifecycleTarget,
        target,
        span,
        counts,
        diagnostics,
    );
}

pub(super) fn collection_storage_relocate_coverage(
    function: &str,
    old_storage: &Place,
    new_storage: &Place,
    span: Span,
    counts: &mut ResourceCoverageCounts,
    diagnostics: &mut Vec<ResourceCoverageDiagnostic>,
) {
    counts.collection_storage_relocates += 1;
    resource_alias_place_coverage(
        function,
        CoveragePlaceOp::CollectionStorageRelocateOld,
        old_storage,
        span,
        counts,
        diagnostics,
    );
    resource_alias_place_coverage(
        function,
        CoveragePlaceOp::CollectionStorageRelocateNew,
        new_storage,
        span,
        counts,
        diagnostics,
    );
}

pub(super) fn collection_slot_drop_traversal_coverage(
    function: &str,
    storage: &Place,
    initialized_count: &Place,
    span: Span,
    counts: &mut ResourceCoverageCounts,
    diagnostics: &mut Vec<ResourceCoverageDiagnostic>,
) {
    counts.collection_slot_lifecycle_ops += 1;
    resource_alias_place_coverage(
        function,
        CoveragePlaceOp::CollectionSlotDropTraversalStorage,
        storage,
        span,
        counts,
        diagnostics,
    );
    resource_place_coverage(
        function,
        CoveragePlaceOp::CollectionSlotDropTraversalInitializedCount,
        initialized_count,
        span,
        counts,
        diagnostics,
    );
}
