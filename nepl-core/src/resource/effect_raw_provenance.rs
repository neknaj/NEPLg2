use alloc::vec::Vec;

use crate::{ast::Effect, types::TypeCtx};

use super::effect_diagnostic::ResourceEffectBoundaryDiagnostic;
use super::effect_return_escape::raw_identity_return_projection_is_escape;
use super::effect_summary::RawIdentityReturnSummary;
use super::model::{EffectOp, Place, ResourceFunction, ResourceId, ResourceOp};
use super::place_utils::{checked_mem_ptr_wrapper_arg_indices, place_with_checked_suffix};

pub(super) fn report_internal_alloc_escapes_from_summary(
    diagnostics: &mut Vec<ResourceEffectBoundaryDiagnostic>,
    function: &ResourceFunction,
    summary: Option<&RawIdentityReturnSummary>,
    types: Option<&TypeCtx>,
) {
    if !matches!(function.effect, Effect::Pure) {
        return;
    }
    let Some(summary) = summary else {
        return;
    };
    let returned = Place::temporary(ResourceId(usize::MAX), function.result);
    for projection in &summary.internal_alloc_returns {
        if !raw_identity_return_projection_is_escape(
            types,
            &returned,
            &projection.projections,
            projection.ty,
        ) {
            continue;
        }
        let Some(place) =
            place_with_checked_suffix(types, &returned, &projection.projections, projection.ty)
        else {
            continue;
        };
        for origin in &projection.origins {
            diagnostics.push(
                ResourceEffectBoundaryDiagnostic::RawAddressEscapeFromInternalAlloc {
                    function: function.name.clone(),
                    operation: origin.operation,
                    place: place.clone(),
                    origin_span: origin.span,
                    span: projection.return_span,
                },
            );
        }
    }
}

pub(super) fn function_needs_raw_provenance_tracking(
    function: &ResourceFunction,
    types: Option<&TypeCtx>,
) -> bool {
    let Some(types) = types else {
        return true;
    };
    function_contains_checked_mem_ptr_access(function, types)
}

fn function_contains_checked_mem_ptr_access(function: &ResourceFunction, types: &TypeCtx) -> bool {
    function
        .blocks
        .iter()
        .any(|block| ops_contain_checked_mem_ptr_access(&block.ops, types))
}

fn ops_contain_checked_mem_ptr_access(ops: &[ResourceOp], types: &TypeCtx) -> bool {
    for op in ops {
        match op {
            ResourceOp::Call { effect, args, .. } => {
                if let EffectOp::UnsafeMemory { operation } = effect {
                    if !checked_mem_ptr_wrapper_arg_indices(types, *operation, args).is_empty() {
                        return true;
                    }
                }
            }
            ResourceOp::Branch {
                then_ops, else_ops, ..
            } => {
                if ops_contain_checked_mem_ptr_access(then_ops, types)
                    || ops_contain_checked_mem_ptr_access(else_ops, types)
                {
                    return true;
                }
            }
            ResourceOp::Loop {
                condition_ops,
                body_ops,
                ..
            } => {
                if ops_contain_checked_mem_ptr_access(condition_ops, types)
                    || ops_contain_checked_mem_ptr_access(body_ops, types)
                {
                    return true;
                }
            }
            ResourceOp::Match { arms, .. } => {
                if arms
                    .iter()
                    .any(|arm| ops_contain_checked_mem_ptr_access(&arm.ops, types))
                {
                    return true;
                }
            }
            ResourceOp::Expr { .. }
            | ResourceOp::DeclareLocal { .. }
            | ResourceOp::Read { .. }
            | ResourceOp::Assign { .. }
            | ResourceOp::Borrow { .. }
            | ResourceOp::Move { .. }
            | ResourceOp::Drop { .. }
            | ResourceOp::EndScope { .. }
            | ResourceOp::CallEffect { .. }
            | ResourceOp::FunctionValue { .. }
            | ResourceOp::IndirectCall { .. }
            | ResourceOp::RawMemory { .. }
            | ResourceOp::RawAddressAlias { .. }
            | ResourceOp::RawAddressView { .. }
            | ResourceOp::StorageOrigin { .. }
            | ResourceOp::CollectionSlotLifecycle { .. }
            | ResourceOp::CollectionStorageRelocate { .. }
            | ResourceOp::CollectionSlotDropTraversal { .. }
            | ResourceOp::Construct { .. } => {}
        }
    }
    false
}
