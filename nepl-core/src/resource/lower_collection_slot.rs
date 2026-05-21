use alloc::vec::Vec;

use crate::hir::HirExpr;
use crate::resource_primitives::CollectionSlotLifecyclePrimitive;
use crate::span::Span;
use crate::types::TypeId;

use super::collection_slot_lifecycle::{CollectionSlotLifecycleEvent, CollectionSlotReplacement};
use super::lower::LoweringEnvironment;
use super::lower_raw_address::{
    raw_address_offset_from_actual_arg, raw_address_source_from_actual_arg,
};
use super::model::{Place, ResourceOp};
use super::place_utils::raw_memory_cell_place;

pub(super) fn push_collection_slot_lifecycle_intrinsic(
    name: &str,
    type_args: &[TypeId],
    hir_args: &[HirExpr],
    arg_places: &[Place],
    ops: &mut Vec<ResourceOp>,
    env: &LoweringEnvironment,
    span: Span,
) -> bool {
    let Some(primitive) = CollectionSlotLifecyclePrimitive::from_intrinsic_name(name) else {
        return false;
    };
    if primitive.requires_storage_pair() {
        push_collection_storage_relocate(hir_args, arg_places, ops, env, span);
        return true;
    }
    if primitive.requires_storage_drop_traversal() {
        push_collection_slot_drop_traversal(
            primitive, type_args, hir_args, arg_places, ops, env, span,
        );
        return true;
    }
    let Some(target) =
        collection_slot_lifecycle_target(primitive, type_args, hir_args, arg_places, env)
    else {
        return true;
    };
    let Some(event) = collection_slot_lifecycle_event(primitive, type_args) else {
        return true;
    };
    ops.push(ResourceOp::CollectionSlotLifecycle {
        target,
        event,
        span,
    });
    true
}

fn push_collection_slot_drop_traversal(
    primitive: CollectionSlotLifecyclePrimitive,
    type_args: &[TypeId],
    hir_args: &[HirExpr],
    arg_places: &[Place],
    ops: &mut Vec<ResourceOp>,
    env: &LoweringEnvironment,
    span: Span,
) {
    let Some(storage) = storage_lifecycle_place(0, hir_args, arg_places, env) else {
        return;
    };
    let Some(expected_ty) = slot_value_type(primitive, type_args) else {
        return;
    };
    ops.push(ResourceOp::CollectionSlotDropTraversal {
        storage,
        expected_ty,
        span,
    });
}

fn push_collection_storage_relocate(
    hir_args: &[HirExpr],
    arg_places: &[Place],
    ops: &mut Vec<ResourceOp>,
    env: &LoweringEnvironment,
    span: Span,
) {
    let Some(old_storage) = storage_lifecycle_place(0, hir_args, arg_places, env) else {
        return;
    };
    let Some(new_storage) = storage_lifecycle_place(1, hir_args, arg_places, env) else {
        return;
    };
    ops.push(ResourceOp::CollectionStorageRelocate {
        old_storage,
        new_storage,
        span,
    });
}

fn storage_lifecycle_place(
    index: usize,
    hir_args: &[HirExpr],
    arg_places: &[Place],
    env: &LoweringEnvironment,
) -> Option<Place> {
    let source = raw_address_source_from_actual_arg(index, hir_args, arg_places, env)?;
    Some(source.into_place_and_view(env.types.i32()).place)
}

fn collection_slot_lifecycle_target(
    primitive: CollectionSlotLifecyclePrimitive,
    type_args: &[TypeId],
    hir_args: &[HirExpr],
    arg_places: &[Place],
    env: &LoweringEnvironment,
) -> Option<Place> {
    let source = raw_address_source_from_actual_arg(0, hir_args, arg_places, env)?;
    let raw = if primitive.has_slot_offset() {
        let value_ty = slot_value_type(primitive, type_args)?;
        let offset = raw_address_offset_from_actual_arg(1, hir_args, arg_places, env);
        let raw = source
            .with_added_offset(offset)
            .into_place_and_view(env.types.i32())
            .place;
        raw_memory_cell_place(&raw, value_ty)
    } else {
        source.into_place_and_view(env.types.i32()).place
    };
    Some(raw)
}

fn collection_slot_lifecycle_event(
    primitive: CollectionSlotLifecyclePrimitive,
    type_args: &[TypeId],
) -> Option<CollectionSlotLifecycleEvent> {
    match primitive {
        CollectionSlotLifecyclePrimitive::InitializeEmpty => {
            Some(CollectionSlotLifecycleEvent::InitializeEmpty {
                value_ty: type_args.first().copied()?,
            })
        }
        CollectionSlotLifecyclePrimitive::BorrowRead => {
            Some(CollectionSlotLifecycleEvent::BorrowRead {
                expected_ty: type_args.first().copied()?,
            })
        }
        CollectionSlotLifecyclePrimitive::MoveOut => Some(CollectionSlotLifecycleEvent::MoveOut {
            expected_ty: type_args.first().copied()?,
        }),
        CollectionSlotLifecyclePrimitive::ReplaceReturnOld => {
            Some(CollectionSlotLifecycleEvent::ReplaceInitialized {
                old_ty: type_args.first().copied()?,
                new_ty: type_args.get(1).copied()?,
                old_owner: CollectionSlotReplacement::ReturnOldOwner,
            })
        }
        CollectionSlotLifecyclePrimitive::ReplaceDropOld => {
            Some(CollectionSlotLifecycleEvent::ReplaceInitialized {
                old_ty: type_args.first().copied()?,
                new_ty: type_args.get(1).copied()?,
                old_owner: CollectionSlotReplacement::DropOldOwner,
            })
        }
        CollectionSlotLifecyclePrimitive::DropInitialized => {
            Some(CollectionSlotLifecycleEvent::DropInitialized {
                expected_ty: type_args.first().copied()?,
            })
        }
        CollectionSlotLifecyclePrimitive::StorageDealloc => {
            Some(CollectionSlotLifecycleEvent::StorageDealloc)
        }
        CollectionSlotLifecyclePrimitive::DropTraversal
        | CollectionSlotLifecyclePrimitive::StorageRelocate => None,
    }
}

fn slot_value_type(
    primitive: CollectionSlotLifecyclePrimitive,
    type_args: &[TypeId],
) -> Option<TypeId> {
    let index = primitive.slot_target_type_arg_index()?;
    type_args.get(index).copied()
}
