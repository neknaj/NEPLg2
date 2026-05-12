extern crate alloc;

use alloc::vec::Vec;

use crate::span::Span;
use crate::types::{TypeCtx, TypeId, TypeKind};

use super::model::{Place, PlaceRoot, ResourceOp};

pub(super) fn push_line_copy_state_only_temporary_scope(
    types: &TypeCtx,
    ops: &mut Vec<ResourceOp>,
    op_start: usize,
    result: Option<Place>,
    span: Span,
) {
    let temporary_locals = copy_state_only_temporaries_from_ops(types, &ops[op_start..]);
    if temporary_locals.is_empty() {
        return;
    }
    ops.push(ResourceOp::EndScope {
        locals: temporary_locals,
        result,
        span,
    });
}

fn copy_state_only_temporaries_from_ops(types: &TypeCtx, ops: &[ResourceOp]) -> Vec<Place> {
    let mut temporaries = Vec::new();
    for op in ops {
        push_copy_state_only_temporary_from_op(types, op, &mut temporaries);
    }
    temporaries
}

fn push_copy_state_only_temporary_from_op(
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
        | ResourceOp::StorageOrigin { target, .. } => {
            push_copy_state_only_temporary(types, target, temporaries);
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
    let resolved = types.resolve_named_type_id(types.resolve_id(ty));
    matches!(types.get_ref(resolved), TypeKind::Str)
}
