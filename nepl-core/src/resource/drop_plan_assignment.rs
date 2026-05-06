use crate::span::Span;
use crate::types::TypeCtx;

use super::drop_model::{ResourceAutoDrop, ResourceAutoDropKind, ResourceDropPoint};
use super::drop_point_path::ResourceDropPointPath;
use super::drop_requirement::resource_drop_requirement_for_type;
use super::model::Place;

pub(super) fn auto_drop_candidate_for_assignment_overwrite(
    types: &TypeCtx,
    target: &Place,
    span: Span,
) -> Option<ResourceAutoDrop> {
    (!types.is_copy(target.ty)).then(|| ResourceAutoDrop {
        place: target.clone(),
        kind: ResourceAutoDropKind::AssignmentOverwrite,
        requirement: resource_drop_requirement_for_type(types, target.ty),
        span,
    })
}

pub(super) fn assignment_overwrite_drop_point(
    types: &TypeCtx,
    target: &Place,
    path: ResourceDropPointPath,
    span: Span,
) -> Option<ResourceDropPoint> {
    auto_drop_candidate_for_assignment_overwrite(types, target, span).map(|auto_drop| {
        ResourceDropPoint {
            path,
            span,
            auto_drops: alloc::vec![auto_drop],
        }
    })
}
