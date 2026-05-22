use crate::ast::Effect;

use super::initialized_alias::RawCellAddressAliases;
use super::model::{EffectOp, Place, RawMemoryOp, ResourceOp};
use super::place_utils::place_suffix_after_prefix;

pub(super) fn unsafe_load_call_matches_raw_load(call: &ResourceOp, load: &ResourceOp) -> bool {
    match (call, load) {
        (
            ResourceOp::Call {
                effect:
                    EffectOp::UnsafeMemory {
                        operation: RawMemoryOp::Load,
                    },
                output: call_output,
                args: call_args,
                ..
            },
            ResourceOp::RawMemory {
                operation: RawMemoryOp::Load,
                output: load_output,
                args: load_args,
                ..
            },
        ) => call_output == load_output && call_args == load_args,
        _ => false,
    }
}

pub(super) fn paired_witness_load_call_preserves_place(
    call: &ResourceOp,
    protected: &Place,
) -> bool {
    match call {
        ResourceOp::Call { output, .. } => !places_touch(output, protected),
        _ => false,
    }
}

pub(super) fn op_preserves_place_during_drop_witness(
    raw_aliases: &RawCellAddressAliases,
    op: &ResourceOp,
    protected: &Place,
) -> bool {
    match op {
        ResourceOp::Assign { target, .. } | ResourceOp::DeclareLocal { place: target, .. } => {
            !places_touch(target, protected)
        }
        ResourceOp::Move { source, output, .. } => {
            !place_touches_anchor(raw_aliases, source, protected)
                && !places_touch(output, protected)
        }
        ResourceOp::Borrow { output: target, .. }
        | ResourceOp::Read { output: target, .. }
        | ResourceOp::FunctionValue { output: target, .. }
        | ResourceOp::Expr { output: target, .. }
        | ResourceOp::RawAddressAlias { target, .. }
        | ResourceOp::RawAddressView { target, .. }
        | ResourceOp::StorageOrigin { target, .. }
        | ResourceOp::Construct { output: target, .. } => !places_touch(target, protected),
        ResourceOp::Drop { place, .. } => !places_touch(place, protected),
        ResourceOp::Call { effect, output, .. } => {
            call_is_drop_witness_safe(effect) && !places_touch(output, protected)
        }
        ResourceOp::RawMemory {
            operation,
            output,
            args,
            ..
        } => {
            matches!(
                operation,
                RawMemoryOp::Load
                    | RawMemoryOp::LoadU8
                    | RawMemoryOp::MemorySize
                    | RawMemoryOp::MemoryGrow
            ) && !places_touch(output, protected)
                && args
                    .iter()
                    .all(|arg| !place_touches_anchor(raw_aliases, arg, protected))
        }
        ResourceOp::EndScope { locals, result, .. } => {
            locals.iter().all(|local| !places_touch(local, protected))
                && result
                    .as_ref()
                    .is_none_or(|result| !places_touch(result, protected))
        }
        ResourceOp::CallEffect { .. } => true,
        ResourceOp::Branch { .. }
        | ResourceOp::Loop { .. }
        | ResourceOp::Match { .. }
        | ResourceOp::IndirectCall { .. }
        | ResourceOp::CollectionSlotLifecycle { .. }
        | ResourceOp::CollectionStorageRelocate { .. }
        | ResourceOp::CollectionSlotDropTraversal { .. } => false,
    }
}

fn call_is_drop_witness_safe(effect: &EffectOp) -> bool {
    matches!(
        effect,
        EffectOp::Pure
            | EffectOp::UserCall {
                effect: Effect::Pure,
                ..
            }
    )
}

pub(super) fn op_loads_from_place(
    raw_aliases: &RawCellAddressAliases,
    op: &ResourceOp,
    protected: &Place,
) -> bool {
    match op {
        ResourceOp::RawMemory {
            operation: RawMemoryOp::Load,
            args,
            ..
        }
        | ResourceOp::Call {
            effect:
                EffectOp::UnsafeMemory {
                    operation: RawMemoryOp::Load,
                },
            args,
            ..
        } => args
            .iter()
            .any(|arg| place_touches(raw_aliases, arg, protected)),
        ResourceOp::Call { .. } | ResourceOp::CallEffect { .. } | ResourceOp::RawMemory { .. } => {
            false
        }
        _ => false,
    }
}

fn place_touches_anchor(raw_aliases: &RawCellAddressAliases, left: &Place, right: &Place) -> bool {
    places_touch(left, right)
        || places_touch(
            &raw_aliases.canonicalize_scalar(left),
            &raw_aliases.canonicalize_scalar(right),
        )
}

fn place_touches(raw_aliases: &RawCellAddressAliases, left: &Place, right: &Place) -> bool {
    places_touch(left, right)
        || places_touch(
            &raw_aliases.canonicalize_owner_cell_address(left),
            &raw_aliases.canonicalize_owner_cell_address(right),
        )
        || places_touch(
            &raw_aliases.canonicalize_scalar(left),
            &raw_aliases.canonicalize_scalar(right),
        )
}

fn places_touch(left: &Place, right: &Place) -> bool {
    place_suffix_after_prefix(left, right).is_some()
        || place_suffix_after_prefix(right, left).is_some()
}
