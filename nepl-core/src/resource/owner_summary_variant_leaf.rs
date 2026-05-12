use alloc::collections::{BTreeMap, BTreeSet};
use alloc::vec::Vec;

use crate::layout::mapped_type_id;
use crate::types::{EnumVariantInfo, TypeCtx, TypeId};

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
        let payload_leaves = owner_leaf_projections_mapped(types, payload_ty, mapping, seen);
        push_nested_owner_leaf_projections(&mut out, projection, payload_leaves);
    }
    out
}
