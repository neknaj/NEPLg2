extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;

use crate::types::TypeCtx;

use super::effect_check::ResourceEffectBoundaryEngine;
use super::effect_counts::ResourceEffectCounts;
use super::effect_diagnostic::ResourceEffectBoundaryDiagnostic;
use super::effect_raw_provenance::{
    function_needs_raw_provenance_tracking, report_internal_alloc_escapes_from_summary,
};
use super::effect_summary::{RawIdentityReturnSummaryIndex, RawPointerReturnSummaryIndex};
use super::effect_summary_identity::compute_raw_identity_return_summaries;
use super::effect_summary_pointer::compute_raw_pointer_return_summaries;
use super::model::ResourceModule;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceEffectBoundaryReport {
    pub functions: Vec<ResourceEffectFunctionCheck>,
    pub diagnostics: Vec<ResourceEffectBoundaryDiagnostic>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceEffectFunctionCheck {
    pub name: String,
    pub counts: ResourceEffectCounts,
}

pub fn check_resource_effect_boundaries(module: &ResourceModule) -> ResourceEffectBoundaryReport {
    check_resource_effect_boundaries_with_types(module, None)
}

pub fn check_resource_effect_boundaries_typed(
    module: &ResourceModule,
    types: &TypeCtx,
) -> ResourceEffectBoundaryReport {
    check_resource_effect_boundaries_with_types(module, Some(types))
}

fn check_resource_effect_boundaries_with_types(
    module: &ResourceModule,
    types: Option<&TypeCtx>,
) -> ResourceEffectBoundaryReport {
    let mut functions = Vec::new();
    let mut diagnostics = Vec::new();
    let pointer_summaries = compute_raw_pointer_return_summaries(module, types);
    let pointer_summary_index = RawPointerReturnSummaryIndex::new(&pointer_summaries);
    let summaries = compute_raw_identity_return_summaries(module, &pointer_summaries, types);
    let summary_index = RawIdentityReturnSummaryIndex::new(&summaries);

    for function in &module.functions {
        let track_raw_provenance = function_needs_raw_provenance_tracking(function, types);
        let mut engine = ResourceEffectBoundaryEngine {
            function: function.name.as_str(),
            effect: function.effect,
            summaries: &summary_index,
            pointer_summaries: &pointer_summary_index,
            types,
            track_alloc_identities: track_raw_provenance,
            propagate_return_provenance: track_raw_provenance,
            diagnostics: Vec::new(),
            counts: ResourceEffectCounts::default(),
        };
        engine.check_function(function);
        diagnostics.extend(engine.diagnostics);
        report_internal_alloc_escapes_from_summary(
            &mut diagnostics,
            function,
            summary_index.get(function.name.as_str()),
            types,
        );
        functions.push(ResourceEffectFunctionCheck {
            name: function.name.clone(),
            counts: engine.counts,
        });
    }

    ResourceEffectBoundaryReport {
        functions,
        diagnostics,
    }
}
