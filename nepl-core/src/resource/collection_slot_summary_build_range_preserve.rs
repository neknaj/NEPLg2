use crate::ast::Effect;

use super::model::{EffectOp, Place, RawMemoryOp, ResourceOp};
use super::place_utils::place_suffix_after_prefix;

pub(super) fn body_preserves_place(ops: &[ResourceOp], protected: &Place) -> bool {
    ops.iter().all(|op| match op {
        ResourceOp::Assign { target, .. }
        | ResourceOp::Move { source: target, .. }
        | ResourceOp::Drop { place: target, .. }
        | ResourceOp::CollectionSlotLifecycle { target, .. } => !place_touches(target, protected),
        ResourceOp::RawMemory {
            operation, output, ..
        } => matches!(operation, RawMemoryOp::Load) && !place_touches(output, protected),
        ResourceOp::Call { effect, output, .. } => {
            effect_preserves_loop_places(effect) && !place_touches(output, protected)
        }
        ResourceOp::Read { output, .. }
        | ResourceOp::Borrow { output, .. }
        | ResourceOp::FunctionValue { output, .. }
        | ResourceOp::Construct { output, .. }
        | ResourceOp::Expr { output, .. }
        | ResourceOp::RawAddressAlias { target: output, .. }
        | ResourceOp::RawAddressView { target: output, .. }
        | ResourceOp::StorageOrigin { target: output, .. } => !place_touches(output, protected),
        ResourceOp::Branch { .. }
        | ResourceOp::Loop { .. }
        | ResourceOp::Match { .. }
        | ResourceOp::CollectionStorageRelocate { .. }
        | ResourceOp::CollectionSlotDropTraversal { .. } => false,
        ResourceOp::CallEffect { .. } | ResourceOp::EndScope { .. } => true,
        ResourceOp::DeclareLocal { place, .. } => !place_touches(place, protected),
        ResourceOp::IndirectCall { .. } => false,
    })
}

fn effect_preserves_loop_places(effect: &EffectOp) -> bool {
    matches!(
        effect,
        EffectOp::Pure
            | EffectOp::UserCall {
                effect: Effect::Pure,
                ..
            }
            | EffectOp::UnsafeMemory {
                operation: RawMemoryOp::Load
            }
    )
}

fn place_touches(left: &Place, right: &Place) -> bool {
    place_suffix_after_prefix(left, right).is_some()
        || place_suffix_after_prefix(right, left).is_some()
}
