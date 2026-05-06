use alloc::collections::{BTreeMap, BTreeSet};
use alloc::string::String;
use alloc::vec::Vec;

use crate::span::Span;

use super::drop_elaboration_bindings::{function_source_bindings, source_name_for_place};
use super::drop_elaboration_validate::validate_drop_point_kind;
use super::drop_model::ResourceAutoDropKind;
use super::drop_point_path::ResourceDropPointPath;
use super::drop_point_resolve::ResourceDropPointResolutionError;
use super::drop_requirement::ResourceDropRequirement;
use super::model::{Place, ResourceFunction, ResourceModule};
use super::report::{ResourceCheckReport, ResourceFunctionCheck};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceDropElaborationPlan {
    pub functions: Vec<ResourceDropElaborationFunction>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceDropElaborationFunction {
    pub name: String,
    pub origin_name: String,
    pub auto_drops: Vec<ResourceDropElaborationDrop>,
    pub drop_points: Vec<ResourceDropElaborationPoint>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceDropElaborationPoint {
    pub path: ResourceDropPointPath,
    pub span: Span,
    pub auto_drops: Vec<ResourceDropElaborationDrop>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceDropElaborationDrop {
    pub place: Place,
    pub source_name: String,
    pub kind: ResourceAutoDropKind,
    pub requirement: ResourceDropRequirement,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResourceDropElaborationPlanError {
    DuplicateFunctionCheck {
        function: String,
    },
    MissingFunctionCheck {
        function: String,
    },
    MissingResourceFunction {
        function: String,
    },
    InvalidDropPointPath {
        function: String,
        path: ResourceDropPointPath,
        span: Span,
        error: ResourceDropPointResolutionError,
    },
    DropPlaceOutsideEndScope {
        function: String,
        path: ResourceDropPointPath,
        place: Place,
        span: Span,
    },
    DropPlaceDoesNotMatchAssignmentTarget {
        function: String,
        path: ResourceDropPointPath,
        place: Place,
        target: Place,
        span: Span,
    },
    MissingDropBinding {
        function: String,
        path: ResourceDropPointPath,
        place: Place,
        span: Span,
    },
}

pub fn compute_resource_drop_elaboration_plan(
    module: &ResourceModule,
    check: &ResourceCheckReport,
) -> Result<ResourceDropElaborationPlan, Vec<ResourceDropElaborationPlanError>> {
    let mut errors = Vec::new();
    let checks_by_name = collect_checks_by_name(check, &mut errors);
    let resource_functions = module
        .functions
        .iter()
        .map(|function| function.name.as_str())
        .collect::<BTreeSet<_>>();

    for check in &check.functions {
        if !resource_functions.contains(check.name.as_str()) {
            errors.push(ResourceDropElaborationPlanError::MissingResourceFunction {
                function: check.name.clone(),
            });
        }
    }

    let mut functions = Vec::new();
    for function in &module.functions {
        let Some(check) = checks_by_name.get(function.name.as_str()) else {
            errors.push(ResourceDropElaborationPlanError::MissingFunctionCheck {
                function: function.name.clone(),
            });
            continue;
        };
        functions.push(validate_function_drop_points(function, check, &mut errors));
    }

    if errors.is_empty() {
        Ok(ResourceDropElaborationPlan { functions })
    } else {
        Err(errors)
    }
}

fn collect_checks_by_name<'a>(
    report: &'a ResourceCheckReport,
    errors: &mut Vec<ResourceDropElaborationPlanError>,
) -> BTreeMap<&'a str, &'a ResourceFunctionCheck> {
    let mut checks_by_name = BTreeMap::new();
    for check in &report.functions {
        if checks_by_name.insert(check.name.as_str(), check).is_some() {
            errors.push(ResourceDropElaborationPlanError::DuplicateFunctionCheck {
                function: check.name.clone(),
            });
        }
    }
    checks_by_name
}

fn validate_function_drop_points(
    function: &ResourceFunction,
    check: &ResourceFunctionCheck,
    errors: &mut Vec<ResourceDropElaborationPlanError>,
) -> ResourceDropElaborationFunction {
    let source_bindings = function_source_bindings(function);
    let mut drop_points = Vec::new();
    for point in &check.auto_drop_points {
        let mut auto_drops = Vec::new();
        for drop in &point.auto_drops {
            if validate_drop_point_kind(function, point, drop, errors) {
                let Some(source_name) = source_name_for_place(&source_bindings, &drop.place) else {
                    errors.push(ResourceDropElaborationPlanError::MissingDropBinding {
                        function: function.name.clone(),
                        path: point.path.clone(),
                        place: drop.place.clone(),
                        span: drop.span,
                    });
                    continue;
                };
                auto_drops.push(ResourceDropElaborationDrop {
                    place: drop.place.clone(),
                    source_name,
                    kind: drop.kind,
                    requirement: drop.requirement.clone(),
                    span: drop.span,
                });
            }
        }
        drop_points.push(ResourceDropElaborationPoint {
            path: point.path.clone(),
            span: point.span,
            auto_drops,
        });
    }

    ResourceDropElaborationFunction {
        name: function.name.clone(),
        origin_name: function.origin_name.clone(),
        auto_drops: drop_points
            .iter()
            .flat_map(|point| point.auto_drops.iter().cloned())
            .collect(),
        drop_points,
    }
}
