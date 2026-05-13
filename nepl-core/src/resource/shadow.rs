use crate::hir::HirModule;
use crate::types::TypeCtx;

use super::borrow_check::check_resource_borrow_lifetimes;
use super::coverage::compare_hir_resource_lowering_typed;
use super::effect::check_resource_effect_boundaries;
use super::initialized::check_resource_initialized_moves;
use super::lower::lower_hir_module;
use super::owner_entry::check_resource_owner_obligations;
use super::report::ResourceSafetyShadowReport;

pub fn check_hir_resource_safety_shadow(
    module: &HirModule,
    types: &TypeCtx,
) -> ResourceSafetyShadowReport {
    let resource = lower_hir_module(module, types);
    ResourceSafetyShadowReport {
        lowering_coverage: compare_hir_resource_lowering_typed(module, &resource, types),
        initialized_moves: check_resource_initialized_moves(&resource, types),
        owner_obligations: check_resource_owner_obligations(&resource, types),
        borrow_lifetimes: check_resource_borrow_lifetimes(&resource, types),
        effect_boundaries: check_resource_effect_boundaries(&resource),
    }
}
