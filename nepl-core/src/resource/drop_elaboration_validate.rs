use alloc::vec::Vec;

use super::drop_elaboration::ResourceDropElaborationPlanError;
use super::drop_model::{ResourceAutoDrop, ResourceAutoDropKind, ResourceDropPoint};
use super::drop_point_resolve::resolve_resource_drop_point_end_scope;
use super::drop_point_resolve_assignment::resolve_resource_drop_point_assignment;
use super::model::ResourceFunction;

pub(super) fn validate_drop_point_kind(
    function: &ResourceFunction,
    point: &ResourceDropPoint,
    drop: &ResourceAutoDrop,
    errors: &mut Vec<ResourceDropElaborationPlanError>,
) -> bool {
    match drop.kind {
        ResourceAutoDropKind::ScopeLocal => validate_scope_local(function, point, drop, errors),
        ResourceAutoDropKind::AssignmentOverwrite => {
            validate_assignment_overwrite(function, point, drop, errors)
        }
    }
}

fn validate_scope_local(
    function: &ResourceFunction,
    point: &ResourceDropPoint,
    drop: &ResourceAutoDrop,
    errors: &mut Vec<ResourceDropElaborationPlanError>,
) -> bool {
    let end_scope = match resolve_resource_drop_point_end_scope(function, &point.path) {
        Ok(end_scope) => end_scope,
        Err(error) => {
            errors.push(ResourceDropElaborationPlanError::InvalidDropPointPath {
                function: function.name.clone(),
                path: point.path.clone(),
                span: point.span,
                error,
            });
            return false;
        }
    };
    if !end_scope.locals.iter().any(|local| local == &drop.place) {
        errors.push(ResourceDropElaborationPlanError::DropPlaceOutsideEndScope {
            function: function.name.clone(),
            path: point.path.clone(),
            place: drop.place.clone(),
            span: drop.span,
        });
        return false;
    }
    true
}

fn validate_assignment_overwrite(
    function: &ResourceFunction,
    point: &ResourceDropPoint,
    drop: &ResourceAutoDrop,
    errors: &mut Vec<ResourceDropElaborationPlanError>,
) -> bool {
    let assignment = match resolve_resource_drop_point_assignment(function, &point.path) {
        Ok(assignment) => assignment,
        Err(error) => {
            errors.push(ResourceDropElaborationPlanError::InvalidDropPointPath {
                function: function.name.clone(),
                path: point.path.clone(),
                span: point.span,
                error,
            });
            return false;
        }
    };
    if assignment.target != &drop.place {
        errors.push(
            ResourceDropElaborationPlanError::DropPlaceDoesNotMatchAssignmentTarget {
                function: function.name.clone(),
                path: point.path.clone(),
                place: drop.place.clone(),
                target: assignment.target.clone(),
                span: drop.span,
            },
        );
        return false;
    }
    true
}
