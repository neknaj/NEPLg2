use alloc::collections::{BTreeMap, BTreeSet};
use alloc::vec::Vec;

use crate::layout::mapped_type_id;
use crate::types::{EnumVariantInfo, TypeCtx, TypeId, TypeKind};

use super::model::PlaceProjection;
use super::owner_summary_leaf::{
    owner_leaf_projections_mapped, push_nested_owner_leaf_projections, OwnerLeafProjection,
};

pub(super) fn enum_owner_leaf_projections(
    types: &TypeCtx,
    variants: &[EnumVariantInfo],
    mapping: &BTreeMap<TypeId, TypeId>,
    seen: &mut BTreeSet<TypeId>,
) -> Vec<OwnerLeafProjection> {
    let mut out = Vec::new();
    for variant in variants {
        let Some(payload) = variant.payload else {
            continue;
        };
        let payload_ty = mapped_type_id(types, payload, mapping);
        let projection = PlaceProjection::EnumPayload {
            variant: variant.name.clone(),
        };
        let mut payload_leaves = owner_leaf_projections_mapped(types, payload_ty, mapping, seen);
        if payload_leaves.is_empty() && scalar_enum_payload_can_carry_owner(types, payload_ty) {
            payload_leaves.push(OwnerLeafProjection {
                suffix: Vec::new(),
                ty: payload_ty,
            });
        }
        push_nested_owner_leaf_projections(&mut out, projection, payload_leaves);
    }
    out
}

fn scalar_enum_payload_can_carry_owner(types: &TypeCtx, ty: TypeId) -> bool {
    matches!(types.get_ref(types.resolve_id(ty)), TypeKind::I32)
}
