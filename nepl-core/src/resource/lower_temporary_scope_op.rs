extern crate alloc;

use alloc::collections::{BTreeMap, BTreeSet};
use alloc::vec::Vec;

use crate::types::{TypeCtx, TypeId, TypeKind};

use super::model::{Place, PlaceRoot, ResourceOp};
use super::owner_summary_leaf::owner_leaf_projections_mapped;

pub(super) fn push_copy_state_only_temporary_from_op(
    types: &TypeCtx,
    op: &ResourceOp,
    temporaries: &mut Vec<Place>,
) {
    match op {
        ResourceOp::Expr { output, .. }
        | ResourceOp::Read { output, .. }
        | ResourceOp::Borrow { output, .. }
        | ResourceOp::Move { output, .. }
        | ResourceOp::Call { output, .. }
        | ResourceOp::IndirectCall { output, .. }
        | ResourceOp::RawMemory { output, .. }
        | ResourceOp::FunctionValue { output, .. }
        | ResourceOp::Construct { output, .. } => {
            push_copy_state_only_temporary(types, output, temporaries);
        }
        ResourceOp::RawAddressAlias { target, .. }
        | ResourceOp::RawAddressView { target, .. }
        | ResourceOp::StorageOrigin { target, .. }
        | ResourceOp::CollectionSlotLifecycle { target, .. } => {
            push_copy_state_only_temporary(types, target, temporaries);
        }
        ResourceOp::CollectionStorageRelocate {
            old_storage,
            new_storage,
            ..
        } => {
            push_copy_state_only_temporary(types, old_storage, temporaries);
            push_copy_state_only_temporary(types, new_storage, temporaries);
        }
        ResourceOp::CollectionSlotDropTraversal {
            storage,
            initialized_count,
            ..
        } => {
            push_copy_state_only_temporary(types, storage, temporaries);
            push_copy_state_only_temporary(types, initialized_count, temporaries);
        }
        ResourceOp::CollectionSlotTransformRange {
            source_storage,
            source_initialized_count,
            output_storage,
            output_initialized_count,
            ..
        } => {
            push_copy_state_only_temporary(types, source_storage, temporaries);
            push_copy_state_only_temporary(types, source_initialized_count, temporaries);
            push_copy_state_only_temporary(types, output_storage, temporaries);
            push_copy_state_only_temporary(types, output_initialized_count, temporaries);
        }
        ResourceOp::Branch { output, .. } | ResourceOp::Match { output, .. } => {
            push_copy_state_only_temporary(types, output, temporaries);
        }
        ResourceOp::DeclareLocal { .. }
        | ResourceOp::Assign { .. }
        | ResourceOp::Drop { .. }
        | ResourceOp::EndScope { .. }
        | ResourceOp::CallEffect { .. }
        | ResourceOp::Loop { .. } => {}
    }
}

fn push_copy_state_only_temporary(types: &TypeCtx, place: &Place, temporaries: &mut Vec<Place>) {
    if !matches!(place.root, PlaceRoot::Temporary(_))
        || !copy_state_only_temporary_needs_resource_scope(types, place.ty)
        || temporaries.iter().any(|existing| existing == place)
    {
        return;
    }
    temporaries.push(place.clone());
}

fn copy_state_only_temporary_needs_resource_scope(types: &TypeCtx, ty: TypeId) -> bool {
    if !types.is_copy(ty) {
        return false;
    }
    let mapping = BTreeMap::new();
    let mut seen = BTreeSet::new();
    owner_leaf_projections_mapped(types, ty, &mapping, &mut seen)
        .into_iter()
        .any(|leaf| {
            let resolved = types.resolve_named_type_id(types.resolve_id(leaf.ty));
            matches!(types.get_ref(resolved), TypeKind::Str)
        })
}
