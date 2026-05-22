use crate::ast::Effect;

use super::model::{EffectOp, Place, ResourceExprKind};
use super::place_utils::place_suffix_after_prefix;

pub(super) enum LoopStepExprEffect<'a> {
    Marker,
    LiteralOne(&'a Place),
    Clear(&'a Place),
    Reject,
}

pub(super) fn loop_step_expr_effect<'a>(
    kind: &ResourceExprKind,
    output: &'a Place,
    index: &Place,
) -> LoopStepExprEffect<'a> {
    if matches!(kind, ResourceExprKind::LiteralI32(1)) {
        return if place_touches(output, index) {
            LoopStepExprEffect::Reject
        } else {
            LoopStepExprEffect::LiteralOne(output)
        };
    }
    if matches!(
        kind,
        ResourceExprKind::LocalRead | ResourceExprKind::Call | ResourceExprKind::Intrinsic
    ) {
        return LoopStepExprEffect::Marker;
    }
    if place_touches(output, index) {
        LoopStepExprEffect::Reject
    } else {
        LoopStepExprEffect::Clear(output)
    }
}

pub(super) fn effect_is_proof_pure(effect: &EffectOp) -> bool {
    matches!(
        effect,
        EffectOp::Pure
            | EffectOp::UserCall {
                effect: Effect::Pure,
                ..
            }
    )
}

fn place_touches(left: &Place, right: &Place) -> bool {
    place_suffix_after_prefix(left, right).is_some()
        || place_suffix_after_prefix(right, left).is_some()
}
