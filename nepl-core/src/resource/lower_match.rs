use alloc::string::String;

use crate::hir::{HirMatchArm, HirMatchBindMode, HirMatchPattern};
use crate::types::{TypeCtx, TypeId, TypeKind};

use super::model::{Place, PlaceProjection};

pub(super) fn resource_match_scrutinee_place(types: &TypeCtx, scrutinee: &Place) -> (Place, bool) {
    let resolved = types.resolve_named_type_id(types.resolve_id(scrutinee.ty));
    let TypeKind::Reference(target, _) = types.get_ref(resolved) else {
        return (scrutinee.clone(), false);
    };
    let target = types.resolve_named_type_id(types.resolve_id(*target));
    if type_is_enum_like(types, target) {
        (
            scrutinee
                .clone()
                .with_projection(PlaceProjection::Deref, target),
            true,
        )
    } else {
        (scrutinee.clone(), false)
    }
}

pub(super) fn borrowed_match_payload_source(
    types: &TypeCtx,
    scrutinee: &Place,
    arm: &HirMatchArm,
    bind_local: Option<&Place>,
) -> Option<Place> {
    if !matches!(arm.bind_mode, Some(HirMatchBindMode::Borrowed { .. })) {
        return None;
    }
    let bind_local = bind_local?;
    let payload_ty = reference_target_type(types, bind_local.ty)?;
    let HirMatchPattern::Variant(variant) = &arm.pattern else {
        return None;
    };
    Some(scrutinee.clone().with_projection(
        PlaceProjection::EnumPayload {
            variant: String::from(crate::qualified_name::member_tail(variant)),
        },
        payload_ty,
    ))
}

pub(super) fn type_is_reference_to_enum(types: &TypeCtx, ty: TypeId) -> bool {
    let resolved = types.resolve_named_type_id(types.resolve_id(ty));
    let TypeKind::Reference(target, _) = types.get_ref(resolved) else {
        return false;
    };
    type_is_enum_like(types, *target)
}

fn reference_target_type(types: &TypeCtx, ty: TypeId) -> Option<TypeId> {
    let resolved = types.resolve_named_type_id(types.resolve_id(ty));
    match types.get_ref(resolved) {
        TypeKind::Reference(target, _) => {
            Some(types.resolve_named_type_id(types.resolve_id(*target)))
        }
        _ => None,
    }
}

fn type_is_enum_like(types: &TypeCtx, ty: TypeId) -> bool {
    let resolved = types.resolve_named_type_id(types.resolve_id(ty));
    match types.get_ref(resolved) {
        TypeKind::Enum { .. } => true,
        TypeKind::Apply { base, .. } => {
            matches!(
                types.get_ref(types.resolve_named_type_id(*base)),
                TypeKind::Enum { .. }
            )
        }
        _ => false,
    }
}
