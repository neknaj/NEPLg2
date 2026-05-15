use crate::types::{TypeCtx, TypeId};

use super::effect_return_owner_type::{
    raw_identity_type_is_opaque_owner, raw_identity_type_is_structural_owner_carrier,
};
use super::model::PlaceProjection;
use super::place_utils::projection_result_type;

pub(super) fn raw_identity_projection_has_owner_protection(
    types: &TypeCtx,
    root_ty: TypeId,
    suffix: &[PlaceProjection],
) -> bool {
    let mut current_ty = types.resolve_named_type_id(types.resolve_id(root_ty));
    if raw_identity_type_is_opaque_owner(types, current_ty) {
        return true;
    }
    if suffix.is_empty() && raw_identity_type_is_structural_owner_carrier(types, current_ty) {
        return true;
    }
    for (index, projection) in suffix.iter().enumerate() {
        if raw_identity_type_is_opaque_owner(types, current_ty) {
            return true;
        }
        current_ty = projection_result_type(types, current_ty, projection).unwrap_or(current_ty);
        if raw_identity_type_is_opaque_owner(types, current_ty) {
            return true;
        }
        if index + 1 == suffix.len()
            && raw_identity_type_is_structural_owner_carrier(types, current_ty)
        {
            return true;
        }
    }
    false
}
