use crate::ast::Effect;
use crate::types::TypeKind;

use super::initialized::ResourceCheckEngine;
use super::initialized_alias::RawCellAddressAliases;
use super::model::{EffectOp, Place, RawMemoryOp, ResourceOp};
use super::place_utils::{place_suffix_after_prefix, reference_target_place};

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
        } => effect_call_preserves_place(engine, raw_aliases, effect, output, args, protected),
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

pub(super) fn effect_call_preserves_place(
    engine: &ResourceCheckEngine<'_>,
    raw_aliases: &RawCellAddressAliases,
    effect: &EffectOp,
    output: &Place,
    args: &[Place],
    protected: &Place,
) -> bool {
    if place_touches(raw_aliases, output, protected) {
        return false;
    }
    // effectful call でも protected owner を値渡しで消費せず、protected owner へ
    // 到達する参照も渡さないなら、この検査で保護している place 自体は保持されます。
    // `Drop::drop &loaded` のような full-range traversal witness は副作用を持ちますが、
    // loop bound や storage token を渡していない限り、範囲証明の保持条件を壊しません。
    match effect {
        EffectOp::Pure
        | EffectOp::UserCall {
            effect: Effect::Pure,
            ..
        } => args.iter().all(|arg| {
            !place_touches(raw_aliases, arg, protected)
                && !reference_argument_targets_place(engine, raw_aliases, arg, protected)
        }),
        EffectOp::UnsafeMemory {
            operation: RawMemoryOp::Load,
        } => args
            .iter()
            .all(|arg| !consumes_protected_place(engine, raw_aliases, arg, protected)),
        EffectOp::UserCall { .. }
        | EffectOp::ExternalIo { .. }
        | EffectOp::Nondet { .. }
        | EffectOp::IndirectCall { .. }
        | EffectOp::Unknown { .. } => args.iter().all(|arg| {
            !consumes_protected_place(engine, raw_aliases, arg, protected)
                && !reference_argument_targets_place(engine, raw_aliases, arg, protected)
        }),
        EffectOp::InternalAlloc { .. } | EffectOp::UnsafeMemory { .. } => false,
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

fn reference_argument_targets_place(
    engine: &ResourceCheckEngine<'_>,
    raw_aliases: &RawCellAddressAliases,
    arg: &Place,
    protected: &Place,
) -> bool {
    let resolved = engine
        .types
        .resolve_named_type_id(engine.types.resolve_id(arg.ty));
    let TypeKind::Reference(target_ty, _) = engine.types.get_ref(resolved) else {
        return false;
    };
    let target = reference_target_place(arg, *target_ty);
    place_touches(raw_aliases, &target, protected)
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
