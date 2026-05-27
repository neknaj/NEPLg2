extern crate alloc;

use alloc::collections::BTreeMap;
use alloc::vec::Vec;

use crate::layout::{extend_type_mapping, mapped_type_id};
use crate::resource_primitives::type_is_raw_pointer;
use crate::types::{TypeCtx, TypeId, TypeKind};

use super::collection_slot_owner_carrier::type_carries_collection_slot_owner;

pub(super) fn type_can_carry_collection_slot_storage(types: &TypeCtx, ty: TypeId) -> bool {
    type_can_carry_collection_slot_storage_mapped(types, ty, &BTreeMap::new(), &mut Vec::new())
}

fn type_can_carry_collection_slot_storage_mapped(
    types: &TypeCtx,
    ty: TypeId,
    mapping: &BTreeMap<TypeId, TypeId>,
    seen: &mut Vec<TypeId>,
) -> bool {
    let mapped = mapped_type_id(types, ty, mapping);
    let resolved = types.resolve_named_type_id(types.resolve_id(mapped));
    if type_carries_collection_slot_owner(types, resolved) {
        return true;
    }
    if type_is_raw_pointer(types, resolved) {
        return false;
    }
    if seen.contains(&resolved) {
        return false;
    }
    seen.push(resolved);
    let carries = match types.get_ref(resolved) {
        TypeKind::Struct { fields, .. } => fields.iter().any(|field| {
            type_can_carry_collection_slot_storage_mapped(types, *field, mapping, seen)
        }),
        TypeKind::Enum { variants, .. } => variants.iter().any(|variant| {
            variant.payload.is_some_and(|payload| {
                type_can_carry_collection_slot_storage_mapped(types, payload, mapping, seen)
            })
        }),
        TypeKind::Tuple { items } => items
            .iter()
            .any(|item| type_can_carry_collection_slot_storage_mapped(types, *item, mapping, seen)),
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
                        type_can_carry_collection_slot_storage_mapped(
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
                            type_can_carry_collection_slot_storage_mapped(
                                types,
                                payload,
                                &nested_mapping,
                                seen,
                            )
                        })
                    })
                }
                TypeKind::Tuple { items } => items.iter().any(|item| {
                    type_can_carry_collection_slot_storage_mapped(types, *item, mapping, seen)
                }),
                _ => false,
            }
        }
        TypeKind::Box(target) => {
            type_can_carry_collection_slot_storage_mapped(types, *target, mapping, seen)
        }
        TypeKind::Named(_) => {
            let named = types.resolve_named_type_id(resolved);
            named != resolved
                && type_can_carry_collection_slot_storage_mapped(types, named, mapping, seen)
        }
        TypeKind::Var(var) => var.binding.map_or(true, |binding| {
            type_can_carry_collection_slot_storage_mapped(types, binding, mapping, seen)
        }),
        TypeKind::Reference(_, _)
        | TypeKind::Unit
        | TypeKind::I32
        | TypeKind::U8
        | TypeKind::F32
        | TypeKind::Bool
        | TypeKind::Char
        | TypeKind::Str
        | TypeKind::Never
        | TypeKind::Function { .. } => false,
    };
    seen.pop();
    carries
}

#[cfg(test)]
mod tests {
    use alloc::string::ToString;
    use alloc::vec;

    use crate::source_map::CompilerMemoryType;

    use super::*;

    fn register_empty_struct(types: &mut TypeCtx, name: &str) -> TypeId {
        types.register_named(
            name.to_string(),
            TypeKind::Struct {
                name: name.to_string(),
                type_params: vec![],
                fields: vec![],
                field_names: vec![],
            },
        )
    }

    fn register_region_token(types: &mut TypeCtx) -> TypeId {
        let raw_ty = types.i32();
        let value_ty = types.fresh_var(Some("T".to_string()));
        let region_token_ty = types.register_named(
            "RegionToken".to_string(),
            TypeKind::Struct {
                name: "RegionToken".to_string(),
                type_params: vec![value_ty],
                fields: vec![raw_ty, raw_ty],
                field_names: vec!["raw".to_string(), "size".to_string()],
            },
        );
        types.mark_compiler_memory_type(region_token_ty, CompilerMemoryType::OwnerToken);
        region_token_ty
    }

    #[test]
    fn plain_non_copy_aggregate_does_not_force_collection_slot_storage() {
        let mut types = TypeCtx::new();
        types.set_copy_trait_enabled(true);
        types.register_copy_impl_target(types.unit());
        let storage_ty = register_empty_struct(&mut types, "PlainNonCopy");

        assert!(!type_can_carry_collection_slot_storage(&types, storage_ty));
    }

    #[test]
    fn owner_token_with_copy_payload_does_not_carry_collection_slots() {
        let mut types = TypeCtx::new();
        types.set_copy_trait_enabled(true);
        types.register_copy_impl_target(types.unit());
        types.register_copy_impl_target(types.u8());
        let region_token = register_region_token(&mut types);
        let byte_region = types.apply(region_token, vec![types.u8()]);

        assert!(!type_can_carry_collection_slot_storage(&types, byte_region));
    }

    #[test]
    fn owner_token_with_non_copy_payload_carries_collection_slots() {
        let mut types = TypeCtx::new();
        types.set_copy_trait_enabled(true);
        types.register_copy_impl_target(types.unit());
        let payload_ty = register_empty_struct(&mut types, "OwnedPayload");
        let region_token = register_region_token(&mut types);
        let owned_region = types.apply(region_token, vec![payload_ty]);

        assert!(type_can_carry_collection_slot_storage(&types, owned_region));
    }

    #[test]
    fn copy_scalar_and_raw_pointer_do_not_carry_collection_slot_storage() {
        let mut types = TypeCtx::new();
        types.set_copy_trait_enabled(true);
        types.register_copy_impl_target(types.unit());
        types.register_copy_impl_target(types.i32());
        let mem_ptr_ty = register_empty_struct(&mut types, "MemPtr");
        types.mark_compiler_memory_type(mem_ptr_ty, CompilerMemoryType::RawPointer);

        assert!(!type_can_carry_collection_slot_storage(&types, types.i32()));
        assert!(!type_can_carry_collection_slot_storage(&types, mem_ptr_ty));
    }
}
