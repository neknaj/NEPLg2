extern crate alloc;

use alloc::collections::{BTreeMap, BTreeSet};
use alloc::vec::Vec;

use crate::layout::{aggregate_fields_with_offsets, extend_type_mapping, mapped_type_id};
use crate::types::{TypeCtx, TypeId, TypeKind};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceDropField {
    pub offset: usize,
    pub ty: TypeId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResourceDropRequirement {
    StateOnly,
    WholeValue,
    DynamicEnumPayload,
    Structural {
        fields: Vec<ResourceDropField>,
        dynamic_enum_fields: Vec<ResourceDropField>,
    },
}

pub fn resource_drop_requirement_for_type(types: &TypeCtx, ty: TypeId) -> ResourceDropRequirement {
    drop_requirement_inner(types, ty, 0, &mut BTreeSet::new())
}

pub fn resource_type_needs_drop_code(types: &TypeCtx, ty: TypeId) -> bool {
    !matches!(
        resource_drop_requirement_for_type(types, ty),
        ResourceDropRequirement::StateOnly
    )
}

fn drop_requirement_inner(
    types: &TypeCtx,
    ty: TypeId,
    base_offset: usize,
    visiting: &mut BTreeSet<TypeId>,
) -> ResourceDropRequirement {
    let resolved = types.resolve_named_type_id(types.resolve_id(ty));
    if types.has_drop_impl_target(resolved) {
        return ResourceDropRequirement::WholeValue;
    }
    if unbound_drop_capability_type_var_needs_drop(types, resolved) {
        return ResourceDropRequirement::WholeValue;
    }
    if !visiting.insert(resolved) {
        return ResourceDropRequirement::StateOnly;
    }

    let requirement = if let Some(variants) = enum_drop_variants(types, ty) {
        let needs_payload_drop = variants
            .into_iter()
            .filter_map(|variant| variant.payload)
            .any(|payload| {
                !matches!(
                    drop_requirement_inner(types, payload, 0, visiting),
                    ResourceDropRequirement::StateOnly
                )
            });
        if needs_payload_drop {
            ResourceDropRequirement::DynamicEnumPayload
        } else {
            ResourceDropRequirement::StateOnly
        }
    } else {
        structural_drop_requirement(types, resolved, base_offset, visiting)
    };

    visiting.remove(&resolved);
    requirement
}

fn unbound_drop_capability_type_var_needs_drop(types: &TypeCtx, ty: TypeId) -> bool {
    matches!(
        types.get_ref(ty),
        TypeKind::Var(var) if var.binding.is_none() && var.drop_cap
    )
}

fn structural_drop_requirement(
    types: &TypeCtx,
    ty: TypeId,
    base_offset: usize,
    visiting: &mut BTreeSet<TypeId>,
) -> ResourceDropRequirement {
    let mut fields = Vec::new();
    let mut dynamic_enum_fields = Vec::new();
    for field in aggregate_fields_with_offsets(types, ty) {
        let offset = base_offset + field.offset;
        match drop_requirement_inner(types, field.ty, offset, visiting) {
            ResourceDropRequirement::StateOnly => {}
            ResourceDropRequirement::WholeValue => fields.push(ResourceDropField {
                offset,
                ty: types.resolve_id(field.ty),
            }),
            ResourceDropRequirement::DynamicEnumPayload => {
                dynamic_enum_fields.push(ResourceDropField {
                    offset,
                    ty: types.resolve_id(field.ty),
                });
            }
            ResourceDropRequirement::Structural {
                fields: nested_fields,
                dynamic_enum_fields: nested_dynamic,
            } => {
                fields.extend(nested_fields);
                dynamic_enum_fields.extend(nested_dynamic);
            }
        }
    }
    if fields.is_empty() && dynamic_enum_fields.is_empty() {
        ResourceDropRequirement::StateOnly
    } else {
        ResourceDropRequirement::Structural {
            fields,
            dynamic_enum_fields,
        }
    }
}

#[derive(Debug, Clone)]
struct EnumDropVariant {
    payload: Option<TypeId>,
}

fn enum_drop_variants(types: &TypeCtx, ty: TypeId) -> Option<Vec<EnumDropVariant>> {
    let resolved = types.resolve_named_type_id(ty);
    match types.get_ref(resolved).clone() {
        TypeKind::Enum { variants, .. } => Some(
            variants
                .into_iter()
                .map(|variant| EnumDropVariant {
                    payload: variant.payload,
                })
                .collect(),
        ),
        TypeKind::Apply { base, args } => {
            let base = types.resolve_named_type_id(base);
            match types.get_ref(base).clone() {
                TypeKind::Enum {
                    type_params,
                    variants,
                    ..
                } => {
                    let mapping = extend_type_mapping(types, &BTreeMap::new(), &type_params, &args);
                    Some(
                        variants
                            .into_iter()
                            .map(|variant| EnumDropVariant {
                                payload: variant
                                    .payload
                                    .map(|payload| mapped_type_id(types, payload, &mapping)),
                            })
                            .collect(),
                    )
                }
                _ => None,
            }
        }
        _ => None,
    }
}
