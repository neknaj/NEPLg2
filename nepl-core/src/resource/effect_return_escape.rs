extern crate alloc;

use alloc::collections::BTreeMap;
use alloc::vec::Vec;

use crate::layout::{extend_type_mapping, mapped_type_id};
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

fn raw_identity_type_is_opaque_owner(types: &TypeCtx, ty: TypeId) -> bool {
    if types.resolve_named_type_id(types.resolve_id(ty)) == types.str() {
        return true;
    }
    is_region_token_type(types, ty)
}

fn raw_identity_type_is_structural_owner_carrier(types: &TypeCtx, ty: TypeId) -> bool {
    raw_identity_struct_type_contains_opaque_owner(types, ty, &BTreeMap::new(), &mut Vec::new())
}

fn raw_identity_struct_type_contains_opaque_owner(
    types: &TypeCtx,
    ty: TypeId,
    mapping: &BTreeMap<TypeId, TypeId>,
    seen: &mut Vec<TypeId>,
) -> bool {
    let resolved = mapped_type_id(types, ty, mapping);
    if raw_identity_type_is_opaque_owner(types, resolved) {
        return true;
    }
    if seen.contains(&resolved) {
        return false;
    }
    seen.push(resolved);
    let result = match types.get_ref(resolved) {
        TypeKind::Struct { fields, .. } => fields
            .iter()
            .any(|field| raw_identity_type_contains_opaque_owner(types, *field, mapping, seen)),
        TypeKind::Apply { base, args } => {
            let base = types.resolve_named_type_id(*base);
            match types.get_ref(base) {
                TypeKind::Struct {
                    type_params,
                    fields,
                    ..
                } => {
                    let nested_mapping = extend_type_mapping(types, mapping, type_params, args);
                    fields.iter().any(|field| {
                        raw_identity_type_contains_opaque_owner(
                            types,
                            *field,
                            &nested_mapping,
                            seen,
                        )
                    })
                }
                _ => false,
            }
        }
        TypeKind::Named(_) => {
            let named = types.resolve_named_type_id(resolved);
            named != resolved
                && raw_identity_struct_type_contains_opaque_owner(types, named, mapping, seen)
        }
        _ => false,
    };
    seen.pop();
    result
}

fn raw_identity_type_contains_opaque_owner(
    types: &TypeCtx,
    ty: TypeId,
    mapping: &BTreeMap<TypeId, TypeId>,
    seen: &mut Vec<TypeId>,
) -> bool {
    let resolved = mapped_type_id(types, ty, mapping);
    if raw_identity_type_is_opaque_owner(types, resolved) {
        return true;
    }
    if seen.contains(&resolved) {
        return false;
    }
    seen.push(resolved);
    let result = match types.get_ref(resolved) {
        TypeKind::Struct { fields, .. } => fields
            .iter()
            .any(|field| raw_identity_type_contains_opaque_owner(types, *field, mapping, seen)),
        TypeKind::Tuple { items } => items
            .iter()
            .any(|item| raw_identity_type_contains_opaque_owner(types, *item, mapping, seen)),
        TypeKind::Apply { base, args } => {
            let base = types.resolve_named_type_id(*base);
            match types.get_ref(base) {
                TypeKind::Struct {
                    type_params,
                    fields,
                    ..
                } => {
                    let nested_mapping = extend_type_mapping(types, mapping, type_params, args);
                    fields.iter().any(|field| {
                        raw_identity_type_contains_opaque_owner(
                            types,
                            *field,
                            &nested_mapping,
                            seen,
                        )
                    })
                }
                TypeKind::Tuple { items } => items.iter().any(|item| {
                    raw_identity_type_contains_opaque_owner(types, *item, mapping, seen)
                }),
                _ => false,
            }
        }
        TypeKind::Reference(target, _) | TypeKind::Box(target) => {
            raw_identity_type_contains_opaque_owner(types, *target, mapping, seen)
        }
        TypeKind::Named(_) => {
            let named = types.resolve_named_type_id(resolved);
            named != resolved
                && raw_identity_type_contains_opaque_owner(types, named, mapping, seen)
        }
        TypeKind::Unit
        | TypeKind::I32
        | TypeKind::U8
        | TypeKind::F32
        | TypeKind::Bool
        | TypeKind::Char
        | TypeKind::Str
        | TypeKind::Never
        | TypeKind::Enum { .. }
        | TypeKind::Function { .. }
        | TypeKind::Var(_) => false,
    };
    seen.pop();
    result
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

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::string::ToString;
    use alloc::vec;

    struct OwnerCarrierTypes {
        result_owner_ty: TypeId,
        owner_ty: TypeId,
        region_ty: TypeId,
        i32_ty: TypeId,
    }

    fn owner_carrier_types() -> (TypeCtx, OwnerCarrierTypes) {
        let mut types = TypeCtx::new();
        let i32_ty = types.i32();
        let item_param = types.fresh_var(Some("T".to_string()));
        let result_ok_param = types.fresh_var(Some("T".to_string()));
        let result_err_param = types.fresh_var(Some("E".to_string()));
        let region_base = types.register_named(
            "RegionToken".to_string(),
            TypeKind::Struct {
                doc: None,
                name: "RegionToken".to_string(),
                type_params: vec![item_param],
                fields: vec![i32_ty, i32_ty],
                field_names: vec!["raw".to_string(), "size".to_string()],
            },
        );
        let region_ty = types.apply(region_base, vec![i32_ty]);
        let owner_ty = types.register_named(
            "OwnerBox".to_string(),
            TypeKind::Struct {
                doc: None,
                name: "OwnerBox".to_string(),
                type_params: vec![],
                fields: vec![i32_ty, region_ty],
                field_names: vec!["len".to_string(), "region".to_string()],
            },
        );
        let result_base = types.register_named(
            "Result".to_string(),
            TypeKind::Enum {
                doc: None,
                name: "Result".to_string(),
                type_params: vec![result_ok_param, result_err_param],
                variants: vec![
                    crate::types::EnumVariantInfo {
                        name: "Ok".to_string(),
                        payload: Some(result_ok_param),
                    },
                    crate::types::EnumVariantInfo {
                        name: "Err".to_string(),
                        payload: Some(result_err_param),
                    },
                ],
            },
        );
        let result_owner_ty = types.apply(result_base, vec![owner_ty, i32_ty]);
        (
            types,
            OwnerCarrierTypes {
                result_owner_ty,
                owner_ty,
                region_ty,
                i32_ty,
            },
        )
    }

    #[test]
    fn return_escape_protects_region_token_identity_inside_result_owner_payload() {
        let (types, tys) = owner_carrier_types();
        let returned = Place::temporary(super::super::model::ResourceId(0), tys.result_owner_ty);
        let suffix = vec![
            PlaceProjection::EnumPayload {
                variant: "Ok".to_string(),
            },
            PlaceProjection::Field {
                index: 1,
                offset_bytes: 4,
            },
            PlaceProjection::Field {
                index: 0,
                offset_bytes: 0,
            },
        ];

        assert!(!raw_identity_return_projection_is_escape(
            Some(&types),
            &returned,
            &suffix,
            tys.i32_ty,
        ));
    }

    #[test]
    fn return_escape_rejects_public_i32_field_outside_owner_token() {
        let (types, tys) = owner_carrier_types();
        let returned = Place::temporary(super::super::model::ResourceId(0), tys.result_owner_ty);
        let suffix = vec![
            PlaceProjection::EnumPayload {
                variant: "Ok".to_string(),
            },
            PlaceProjection::Field {
                index: 0,
                offset_bytes: 0,
            },
        ];

        assert!(raw_identity_return_projection_is_escape(
            Some(&types),
            &returned,
            &suffix,
            tys.i32_ty,
        ));
    }

    #[test]
    fn return_escape_treats_final_owner_carrier_payload_as_protected() {
        let (types, tys) = owner_carrier_types();
        let returned = Place::temporary(super::super::model::ResourceId(0), tys.result_owner_ty);
        let suffix = vec![PlaceProjection::EnumPayload {
            variant: "Ok".to_string(),
        }];

        assert!(raw_identity_projection_has_owner_protection(
            &types,
            returned.ty,
            &suffix,
        ));
        assert!(!raw_identity_return_projection_is_escape(
            Some(&types),
            &returned,
            &suffix,
            tys.owner_ty,
        ));
    }

    #[test]
    fn return_escape_keeps_region_token_itself_as_owner_provenance() {
        let (types, tys) = owner_carrier_types();
        let returned = Place::temporary(super::super::model::ResourceId(0), tys.region_ty);

        assert!(!raw_identity_return_projection_is_escape(
            Some(&types),
            &returned,
            &[],
            tys.region_ty,
        ));
    }
}
