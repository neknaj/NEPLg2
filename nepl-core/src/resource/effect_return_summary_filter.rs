extern crate alloc;

use alloc::collections::BTreeMap;
use alloc::vec::Vec;

use crate::layout::{extend_type_mapping, mapped_type_id};
use crate::types::{TypeCtx, TypeId, TypeKind};

use super::effect_return_escape::raw_identity_projection_has_owner_protection;
use super::model::{Place, PlaceProjection};
use super::place_utils::is_mem_ptr_type;

pub(super) fn raw_identity_return_projection_requires_summary(
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
    raw_identity_type_can_propagate_public_escape(
        types,
        projection_ty,
        &BTreeMap::new(),
        &mut Vec::new(),
    )
}

fn raw_identity_type_can_propagate_public_escape(
    types: &TypeCtx,
    ty: TypeId,
    mapping: &BTreeMap<TypeId, TypeId>,
    seen: &mut Vec<TypeId>,
) -> bool {
    let resolved = mapped_type_id(types, ty, mapping);
    if is_mem_ptr_type(types, resolved) {
        return true;
    }
    if seen.contains(&resolved) {
        return false;
    }
    seen.push(resolved);
    let result = match types.get_ref(resolved) {
        TypeKind::I32 => true,
        TypeKind::Struct { fields, .. } => fields.iter().any(|field| {
            raw_identity_type_can_propagate_public_escape(types, *field, mapping, seen)
        }),
        TypeKind::Enum { variants, .. } => variants.iter().any(|variant| {
            variant.payload.is_some_and(|payload| {
                raw_identity_type_can_propagate_public_escape(types, payload, mapping, seen)
            })
        }),
        TypeKind::Tuple { items } => items
            .iter()
            .any(|item| raw_identity_type_can_propagate_public_escape(types, *item, mapping, seen)),
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
                        raw_identity_type_can_propagate_public_escape(
                            types,
                            *field,
                            &nested_mapping,
                            seen,
                        )
                    })
                }
                TypeKind::Enum {
                    type_params,
                    variants,
                    ..
                } => {
                    let nested_mapping = extend_type_mapping(types, mapping, type_params, args);
                    variants.iter().any(|variant| {
                        variant.payload.is_some_and(|payload| {
                            raw_identity_type_can_propagate_public_escape(
                                types,
                                payload,
                                &nested_mapping,
                                seen,
                            )
                        })
                    })
                }
                TypeKind::Tuple { items } => items.iter().any(|item| {
                    raw_identity_type_can_propagate_public_escape(types, *item, mapping, seen)
                }),
                _ => false,
            }
        }
        TypeKind::Reference(target, _) | TypeKind::Box(target) => {
            raw_identity_type_can_propagate_public_escape(types, *target, mapping, seen)
        }
        TypeKind::Named(_) => {
            let named = types.resolve_named_type_id(resolved);
            named != resolved
                && raw_identity_type_can_propagate_public_escape(types, named, mapping, seen)
        }
        TypeKind::Var(_) => true,
        TypeKind::Unit
        | TypeKind::U8
        | TypeKind::F32
        | TypeKind::Bool
        | TypeKind::Char
        | TypeKind::Str
        | TypeKind::Never
        | TypeKind::Function { .. } => false,
    };
    seen.pop();
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resource::model::ResourceId;
    use alloc::string::ToString;
    use alloc::vec;

    fn returned_place(ty: TypeId) -> Place {
        Place::temporary(ResourceId(0), ty)
    }

    #[test]
    fn summary_filter_skips_owner_protected_string_returns() {
        let types = TypeCtx::new();
        let returned = returned_place(types.str());

        assert!(!raw_identity_return_projection_requires_summary(
            Some(&types),
            &returned,
            &[],
            types.str(),
        ));
    }

    #[test]
    fn summary_filter_keeps_public_raw_identity_returns() {
        let mut types = TypeCtx::new();
        let i32_ty = types.i32();
        let mem_ptr_ty = types.register_named(
            "MemPtr".to_string(),
            TypeKind::Struct {
                doc: None,
                name: "MemPtr".to_string(),
                type_params: vec![],
                fields: vec![i32_ty],
                field_names: vec!["raw".to_string()],
            },
        );

        assert!(raw_identity_return_projection_requires_summary(
            Some(&types),
            &returned_place(i32_ty),
            &[],
            i32_ty,
        ));
        assert!(raw_identity_return_projection_requires_summary(
            Some(&types),
            &returned_place(mem_ptr_ty),
            &[],
            mem_ptr_ty,
        ));
    }

    #[test]
    fn summary_filter_respects_reference_target_identity() {
        let mut types = TypeCtx::new();
        let str_ref_ty = types.reference(types.str(), false);
        let i32_ref_ty = types.reference(types.i32(), false);

        assert!(!raw_identity_return_projection_requires_summary(
            Some(&types),
            &returned_place(str_ref_ty),
            &[],
            str_ref_ty,
        ));
        assert!(raw_identity_return_projection_requires_summary(
            Some(&types),
            &returned_place(i32_ref_ty),
            &[],
            i32_ref_ty,
        ));
    }

    #[test]
    fn summary_filter_skips_region_token_owner_returns() {
        let mut types = TypeCtx::new();
        let i32_ty = types.i32();
        let region_token_ty = types.register_named(
            "RegionToken".to_string(),
            TypeKind::Struct {
                doc: None,
                name: "RegionToken".to_string(),
                type_params: vec![],
                fields: vec![i32_ty],
                field_names: vec!["raw".to_string()],
            },
        );

        assert!(!raw_identity_return_projection_requires_summary(
            Some(&types),
            &returned_place(region_token_ty),
            &[],
            region_token_ty,
        ));
    }
}
