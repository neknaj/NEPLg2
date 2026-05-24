use alloc::vec::Vec;

use super::effect_return_summary_filter::raw_identity_return_projection_requires_summary;
use super::effect_summary::RawIdentityReturnSummaryIndex;
use super::effect_summary_seed::parameter_summary_seed_places;
use super::model::{Place, ResourceCallTarget, ResourceFunction, ResourceOp};
use super::place_utils::{place_suffix_after_prefix, place_with_checked_suffix, push_unique_place};
use crate::types::TypeCtx;

pub(super) fn parameter_identity_summary_seed_places(
    function: &ResourceFunction,
    parameter: &Place,
    summaries: &RawIdentityReturnSummaryIndex<'_>,
    types: Option<&TypeCtx>,
) -> Vec<Place> {
    let mut places = parameter_summary_seed_places(function, parameter);
    for block in &function.blocks {
        collect_call_summary_source_seeds(&block.ops, parameter, summaries, types, &mut places);
    }
    places.sort();
    places
}

pub(super) fn summary_seed_can_carry_raw_identity(
    types: Option<&TypeCtx>,
    parameter: &Place,
    seed: &Place,
) -> bool {
    let Some(types) = types else {
        return true;
    };
    raw_identity_seed_type_can_escape(types, seed)
        || (place_suffix_after_prefix(seed, parameter).is_some()
            && raw_identity_seed_type_can_escape(types, parameter))
}

fn raw_identity_seed_type_can_escape(types: &TypeCtx, place: &Place) -> bool {
    raw_identity_return_projection_requires_summary(Some(types), place, &[], place.ty)
}

fn collect_call_summary_source_seeds(
    ops: &[ResourceOp],
    parameter: &Place,
    summaries: &RawIdentityReturnSummaryIndex<'_>,
    types: Option<&TypeCtx>,
    places: &mut Vec<Place>,
) {
    for op in ops {
        match op {
            ResourceOp::Call { target, args, .. } => {
                collect_direct_call_summary_source_seeds(
                    target, args, parameter, summaries, types, places,
                );
            }
            ResourceOp::Branch {
                then_ops, else_ops, ..
            } => {
                collect_call_summary_source_seeds(then_ops, parameter, summaries, types, places);
                collect_call_summary_source_seeds(else_ops, parameter, summaries, types, places);
            }
            ResourceOp::Loop {
                condition_ops,
                body_ops,
                ..
            } => {
                collect_call_summary_source_seeds(
                    condition_ops,
                    parameter,
                    summaries,
                    types,
                    places,
                );
                collect_call_summary_source_seeds(body_ops, parameter, summaries, types, places);
            }
            ResourceOp::Match { arms, .. } => {
                for arm in arms {
                    collect_call_summary_source_seeds(
                        &arm.ops, parameter, summaries, types, places,
                    );
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
            | ResourceOp::CollectionSlotTransformRange { .. }
            | ResourceOp::Construct { .. } => {}
        }
    }
}

fn collect_direct_call_summary_source_seeds(
    target: &ResourceCallTarget,
    args: &[Place],
    parameter: &Place,
    summaries: &RawIdentityReturnSummaryIndex<'_>,
    types: Option<&TypeCtx>,
    places: &mut Vec<Place>,
) {
    let ResourceCallTarget::User { name, .. } = target else {
        return;
    };
    let Some(summary) = summaries.get(name) else {
        return;
    };
    for parameter_return in &summary.parameter_returns {
        let Some(arg) = args.get(parameter_return.parameter_index) else {
            continue;
        };
        let Some(seed) = place_with_checked_suffix(
            types,
            arg,
            &parameter_return.source_projections,
            parameter_return.source_ty,
        ) else {
            continue;
        };
        push_parameter_identity_seed(places, parameter, &seed);
    }
}

fn push_parameter_identity_seed(places: &mut Vec<Place>, parameter: &Place, seed: &Place) {
    if place_suffix_after_prefix(seed, parameter).is_some() {
        push_unique_place(places, seed);
    }
}
