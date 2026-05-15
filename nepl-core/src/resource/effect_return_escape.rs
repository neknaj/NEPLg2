use crate::types::{TypeCtx, TypeId, TypeKind};

use super::effect_return_protection::raw_identity_projection_has_owner_protection;
use super::model::{Place, PlaceProjection};

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
