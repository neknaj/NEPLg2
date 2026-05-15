use crate::types::{TypeCtx, TypeId, TypeKind};

use super::model::{Place, PlaceProjection};
use super::place_utils::projection_result_type;

pub(super) fn raw_identity_return_projection_is_escape(
    types: Option<&TypeCtx>,
    returned: &Place,
    suffix: &[PlaceProjection],
    projection_ty: TypeId,
) -> bool {
    let Some(types) = types else {
        return true;
    };
    if raw_identity_projection_has_owner_protection(types, returned.ty, suffix) {
        return false;
    }
    raw_identity_leaf_type_is_public_escape(types, projection_ty)
}

pub(super) fn raw_identity_projection_has_owner_protection(
    types: &TypeCtx,
    root_ty: TypeId,
    suffix: &[PlaceProjection],
) -> bool {
    let mut current_ty = types.resolve_named_type_id(types.resolve_id(root_ty));
    if type_protects_raw_identity(types, current_ty) {
        return true;
    }
    for projection in suffix {
        if type_protects_raw_identity(types, current_ty) {
            return true;
        }
        current_ty = projection_result_type(types, current_ty, projection).unwrap_or(current_ty);
        if type_protects_raw_identity(types, current_ty) {
            return true;
        }
    }
    false
}

fn raw_identity_leaf_type_is_public_escape(types: &TypeCtx, ty: TypeId) -> bool {
    let resolved = types.resolve_named_type_id(types.resolve_id(ty));
    match types.get_ref(resolved) {
        TypeKind::I32 => true,
        TypeKind::Struct { name, .. } => name == "MemPtr",
        TypeKind::Apply { base, .. } => {
            let base = types.resolve_named_type_id(*base);
            matches!(types.get_ref(base), TypeKind::Struct { name, .. } if name == "MemPtr")
        }
        _ => false,
    }
}

fn type_protects_raw_identity(types: &TypeCtx, ty: TypeId) -> bool {
    if types.resolve_named_type_id(types.resolve_id(ty)) == types.str() {
        return true;
    }
    is_region_token_type(types, ty)
}

fn is_region_token_type(types: &TypeCtx, ty: TypeId) -> bool {
    let resolved = types.resolve_named_type_id(types.resolve_id(ty));
    match types.get_ref(resolved) {
        TypeKind::Struct { name, .. } => name == "RegionToken",
        TypeKind::Apply { base, .. } => {
            let base = types.resolve_named_type_id(*base);
            matches!(types.get_ref(base), TypeKind::Struct { name, .. } if name == "RegionToken")
        }
        _ => false,
    }
}
