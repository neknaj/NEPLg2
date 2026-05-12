extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;

use crate::span::Span;

use super::coverage::{ResourceCoverageCounts, ResourceCoverageDiagnostic};
use super::coverage_operation::ResourceCoveragePlaceOperation;
use super::model::{Place, PlaceProjection, PlaceRoot};

pub(super) fn resource_alias_place_coverage(
    function: &str,
    operation: ResourceCoveragePlaceOperation,
    place: &Place,
    span: Span,
    counts: &mut ResourceCoverageCounts,
    diagnostics: &mut Vec<ResourceCoverageDiagnostic>,
) {
    if matches!(place.root, PlaceRoot::Unknown) {
        counts.unknown_places += 1;
        diagnostics.push(ResourceCoverageDiagnostic::UnknownPlace {
            function: String::from(function),
            operation,
            place: place.clone(),
            span,
        });
    }
}

pub(super) fn resource_place_coverage(
    function: &str,
    operation: ResourceCoveragePlaceOperation,
    place: &Place,
    span: Span,
    counts: &mut ResourceCoverageCounts,
    diagnostics: &mut Vec<ResourceCoverageDiagnostic>,
) {
    counts.deref_projections += place
        .projections
        .iter()
        .filter(|projection| matches!(projection, PlaceProjection::Deref))
        .count();
    if matches!(place.root, PlaceRoot::Unknown) {
        counts.unknown_places += 1;
        diagnostics.push(ResourceCoverageDiagnostic::UnknownPlace {
            function: String::from(function),
            operation,
            place: place.clone(),
            span,
        });
    }
}
