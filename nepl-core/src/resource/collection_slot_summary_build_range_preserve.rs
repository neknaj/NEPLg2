extern crate alloc;

use crate::ast::Effect;

use super::function_alias::FunctionAliasTable;
use super::initialized::ResourceCheckEngine;
use super::initialized_alias::RawCellAddressAliases;
use super::initialized_scalar_flow_ops::propagate_i32_scalar_ops;
use super::model::{EffectOp, Place, RawMemoryOp, ResourceOp};
use super::place_utils::place_suffix_after_prefix;

pub(super) fn body_preserves_place(
    engine: &ResourceCheckEngine<'_>,
    initial_raw_aliases: &RawCellAddressAliases,
    ops: &[ResourceOp],
    protected: &Place,
) -> bool {
    let mut raw_aliases = initial_raw_aliases.clone();
    let mut function_aliases = FunctionAliasTable::default();
    for op in ops {
        if !op_preserves_place(engine, &raw_aliases, op, protected) {
            return false;
        }
        propagate_i32_scalar_ops(
            &mut raw_aliases,
            &mut function_aliases,
            core::slice::from_ref(op),
            engine.i32_scalar_summaries,
            engine.raw_alias_summaries,
            engine.types,
        );
    }
    true
}

fn op_preserves_place(
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
        | ResourceOp::CollectionSlotDropTraversal { .. } => false,
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

#[cfg(test)]
mod tests {
    use alloc::{string::ToString, vec, vec::Vec};

    use crate::span::Span;
    use crate::types::{TypeCtx, TypeId, TypeKind};

    use super::*;
    use crate::resource::collection_slot_summary_model::CollectionSlotLifecycleFunctionSummaryIndex;
    use crate::resource::initialized_alias_flow::RawCellAddressReturnSummaryIndex;
    use crate::resource::initialized_scalar_flow::I32ScalarReturnSummaryIndex;
    use crate::resource::initialized_summary::RawCellInitializationFunctionSummaryIndex;
    use crate::resource::model::{AggregateKind, ResourceCallTarget, ResourceId, ResourceOp};
    use crate::resource::report::ResourceCheckDeferred;

    #[test]
    fn body_preserve_rejects_noncopy_assign_source_consumption() {
        let (types, owned_ty) = preserve_test_types();
        let protected = Place::local("owner".to_string(), owned_ty);
        let sink = Place::temporary(ResourceId(100), owned_ty);
        let op = ResourceOp::Assign {
            target: sink,
            value: protected.clone(),
            span: Span::dummy(),
        };

        with_preserve_test_engine(&types, |engine| {
            assert!(
                !body_preserves_place(
                    engine,
                    &RawCellAddressAliases::default(),
                    &[op],
                    &protected,
                ),
                "Assign must not be treated as preserving a protected non-Copy anchor when the value consumes that anchor"
            );
        });
    }

    #[test]
    fn body_preserve_rejects_noncopy_construct_input_consumption() {
        let (types, owned_ty) = preserve_test_types();
        let protected = Place::local("owner".to_string(), owned_ty);
        let output = Place::temporary(ResourceId(101), owned_ty);
        let op = ResourceOp::Construct {
            output,
            kind: AggregateKind::Struct {
                name: "OwnerPair".to_string(),
                field_offsets: vec![0],
            },
            inputs: vec![protected.clone()],
            span: Span::dummy(),
        };

        with_preserve_test_engine(&types, |engine| {
            assert!(
                !body_preserves_place(
                    engine,
                    &RawCellAddressAliases::default(),
                    &[op],
                    &protected,
                ),
                "Construct must not be treated as preserving a protected non-Copy anchor when an input consumes that anchor"
            );
        });
    }

    #[test]
    fn body_preserve_rejects_opaque_pure_call_anchor_argument() {
        let (types, owned_ty) = preserve_test_types();
        let protected = Place::local("owner".to_string(), owned_ty);
        let output = Place::temporary(ResourceId(102), types.unit());
        let op = ResourceOp::Call {
            output,
            target: ResourceCallTarget::User {
                name: "opaque".to_string(),
                type_args: Vec::new(),
            },
            args: vec![protected.clone()],
            effect: EffectOp::UserCall {
                name: "opaque".to_string(),
                effect: Effect::Pure,
            },
            span: Span::dummy(),
        };

        with_preserve_test_engine(&types, |engine| {
            assert!(
                !body_preserves_place(
                    engine,
                    &RawCellAddressAliases::default(),
                    &[op],
                    &protected,
                ),
                "pure user calls are not generic preservation proof when they receive the protected anchor"
            );
        });
    }

    fn preserve_test_types() -> (TypeCtx, TypeId) {
        let mut types = TypeCtx::new();
        types.set_copy_trait_enabled(true);
        types.register_copy_impl_target(types.unit());
        types.register_copy_impl_target(types.i32());
        types.register_copy_impl_target(types.bool());
        let i32_ty = types.i32();
        let owned_ty = types.register_named(
            "OwnedAnchor".to_string(),
            TypeKind::Struct {
                name: "OwnedAnchor".to_string(),
                type_params: vec![],
                fields: vec![i32_ty],
                field_names: vec!["value".to_string()],
            },
        );
        (types, owned_ty)
    }

    fn with_preserve_test_engine<R>(
        types: &TypeCtx,
        test: impl FnOnce(&ResourceCheckEngine<'_>) -> R,
    ) -> R {
        let raw_alias_summaries = [];
        let i32_scalar_summaries = [];
        let raw_init_summaries = [];
        let collection_slot_summaries = [];
        let raw_alias_summary_index = RawCellAddressReturnSummaryIndex::new(&raw_alias_summaries);
        let i32_scalar_summary_index = I32ScalarReturnSummaryIndex::new(&i32_scalar_summaries);
        let raw_init_summary_index =
            RawCellInitializationFunctionSummaryIndex::new(&raw_init_summaries);
        let collection_slot_summary_index =
            CollectionSlotLifecycleFunctionSummaryIndex::new(&collection_slot_summaries);
        let engine = ResourceCheckEngine {
            function: "preserve_test",
            types,
            raw_alias_summaries: &raw_alias_summary_index,
            i32_scalar_summaries: &i32_scalar_summary_index,
            raw_init_summaries: &raw_init_summary_index,
            collection_slot_summaries: &collection_slot_summary_index,
            diagnostics: Vec::new(),
            auto_drop_points: Vec::new(),
            deferred: ResourceCheckDeferred::default(),
            path_alternatives: Default::default(),
        };
        test(&engine)
    }
}
