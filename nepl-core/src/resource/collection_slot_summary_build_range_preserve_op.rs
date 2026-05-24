use crate::ast::Effect;

use super::initialized::ResourceCheckEngine;
use super::initialized_alias::RawCellAddressAliases;
use super::model::{EffectOp, Place, RawMemoryOp, ResourceOp};
use super::place_utils::place_suffix_after_prefix;

pub(super) fn op_preserves_place(
    engine: &ResourceCheckEngine<'_>,
    raw_aliases: &RawCellAddressAliases,
    op: &ResourceOp,
    protected: &Place,
) -> bool {
    match op {
        ResourceOp::Assign { target, value, .. } => {
            !place_touches(raw_aliases, target, protected)
                && !consumes_protected_place(engine, raw_aliases, value, protected)
        }
        ResourceOp::Drop { place: target, .. }
        | ResourceOp::CollectionSlotLifecycle { target, .. } => {
            !place_touches(raw_aliases, target, protected)
        }
        ResourceOp::Move { source, output, .. } => {
            !place_touches(raw_aliases, source, protected)
                && !place_touches(raw_aliases, output, protected)
        }
        ResourceOp::RawMemory {
            operation, output, ..
        } => {
            matches!(operation, RawMemoryOp::Load) && !place_touches(raw_aliases, output, protected)
        }
        ResourceOp::Call {
            effect,
            output,
            args,
            ..
        } => {
            call_preserves_loop_place(engine, raw_aliases, effect, args, protected)
                && !place_touches(raw_aliases, output, protected)
        }
        ResourceOp::Read { output, .. }
        | ResourceOp::Borrow { output, .. }
        | ResourceOp::FunctionValue { output, .. }
        | ResourceOp::Expr { output, .. }
        | ResourceOp::RawAddressAlias { target: output, .. }
        | ResourceOp::RawAddressView { target: output, .. }
        | ResourceOp::StorageOrigin { target: output, .. } => {
            !place_touches(raw_aliases, output, protected)
        }
        ResourceOp::Construct { output, inputs, .. } => {
            !place_touches(raw_aliases, output, protected)
                && inputs
                    .iter()
                    .all(|input| !consumes_protected_place(engine, raw_aliases, input, protected))
        }
        ResourceOp::Branch { .. }
        | ResourceOp::Loop { .. }
        | ResourceOp::Match { .. }
        | ResourceOp::CollectionStorageRelocate { .. }
        | ResourceOp::CollectionSlotDropTraversal { .. }
        | ResourceOp::CollectionSlotTransformRange { .. } => false,
        ResourceOp::EndScope { locals, result, .. } => {
            locals
                .iter()
                .all(|local| !place_touches(raw_aliases, local, protected))
                && result.as_ref().is_none_or(|result| {
                    !consumes_protected_place(engine, raw_aliases, result, protected)
                })
        }
        ResourceOp::CallEffect { .. } => true,
        ResourceOp::DeclareLocal {
            place, initializer, ..
        } => {
            !place_touches(raw_aliases, place, protected)
                && initializer.as_ref().is_none_or(|initializer| {
                    !consumes_protected_place(engine, raw_aliases, initializer, protected)
                })
        }
        ResourceOp::IndirectCall { .. } => false,
    }
}

pub(super) fn op_preserves_place_after_drop_witness(
    engine: &ResourceCheckEngine<'_>,
    raw_aliases: &RawCellAddressAliases,
    op: &ResourceOp,
    protected: &Place,
) -> bool {
    if let ResourceOp::EndScope { locals, result, .. } = op {
        return locals.iter().all(|local| !places_touch(local, protected))
            && result
                .as_ref()
                .is_none_or(|result| !place_touches(raw_aliases, result, protected));
    }
    op_preserves_place(engine, raw_aliases, op, protected)
        && !matches!(
            op,
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
            } if args
                .iter()
                .any(|arg| place_touches(raw_aliases, arg, protected))
        )
}

fn call_preserves_loop_place(
    engine: &ResourceCheckEngine<'_>,
    raw_aliases: &RawCellAddressAliases,
    effect: &EffectOp,
    args: &[Place],
    protected: &Place,
) -> bool {
    match effect {
        EffectOp::Pure
        | EffectOp::UserCall {
            effect: Effect::Pure,
            ..
        } => args
            .iter()
            .all(|arg| !place_touches(raw_aliases, arg, protected)),
        EffectOp::UnsafeMemory {
            operation: RawMemoryOp::Load,
        } => args
            .iter()
            .all(|arg| !consumes_protected_place(engine, raw_aliases, arg, protected)),
        EffectOp::InternalAlloc { .. }
        | EffectOp::UserCall { .. }
        | EffectOp::UnsafeMemory { .. }
        | EffectOp::ExternalIo { .. }
        | EffectOp::Nondet { .. }
        | EffectOp::IndirectCall { .. }
        | EffectOp::Unknown { .. } => false,
    }
}

fn consumes_protected_place(
    engine: &ResourceCheckEngine<'_>,
    raw_aliases: &RawCellAddressAliases,
    place: &Place,
    protected: &Place,
) -> bool {
    !engine.types.is_copy(place.ty) && place_touches(raw_aliases, place, protected)
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
