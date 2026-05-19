extern crate alloc;

use alloc::vec::Vec;

use crate::types::TypeCtx;

use super::model::{OwnerStateEntry, ResourceModule};
use super::owner_check::ResourceOwnerCheckEngine;
use super::owner_check_utils::merge_owner_deferred;
use super::report::{
    ResourceOwnerCheckDeferred, ResourceOwnerCheckReport, ResourceOwnerFunctionCheck,
};
use super::summary::compute_owner_return_summaries;
use super::summary::OwnerReturnSummaryIndex;
use super::timing::ResourceStageTimer;

pub fn check_resource_owner_obligations(
    module: &ResourceModule,
    types: &TypeCtx,
) -> ResourceOwnerCheckReport {
    let stage_start = ResourceStageTimer::start();
    let mut functions = Vec::new();
    let mut diagnostics = Vec::new();
    let mut deferred = ResourceOwnerCheckDeferred::default();
    let summaries = compute_owner_return_summaries(module, types);
    let summary_index = OwnerReturnSummaryIndex::new(&summaries);
    stage_start.log("resource_owner_summaries");
    let stage_start = ResourceStageTimer::start();

    for function in &module.functions {
        let mut engine = ResourceOwnerCheckEngine {
            function: function.name.as_str(),
            types,
            summaries: &summary_index,
            diagnostics: Vec::new(),
            deferred: ResourceOwnerCheckDeferred::default(),
            owner_extent_requirements: Vec::new(),
            memory_span_requirements: Vec::new(),
            params: &function.params,
        };
        let final_owners: Vec<OwnerStateEntry> = engine.check_function(function);
        merge_owner_deferred(&mut deferred, engine.deferred);
        diagnostics.extend(engine.diagnostics);
        functions.push(ResourceOwnerFunctionCheck {
            name: function.name.clone(),
            final_owners,
            deferred: engine.deferred,
        });
    }
    stage_start.log("resource_owner_function_checks");

    ResourceOwnerCheckReport {
        functions,
        diagnostics,
        deferred,
    }
}
