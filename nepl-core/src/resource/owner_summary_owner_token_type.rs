use alloc::collections::{BTreeMap, BTreeSet};

use crate::layout::{aggregate_fields_with_offsets, extend_type_mapping, mapped_type_id};
use crate::resource_primitives::type_is_owner_token;
use crate::types::{TypeCtx, TypeId, TypeKind};

pub(super) fn type_contains_owner_token(types: &TypeCtx, ty: TypeId) -> bool {
    type_contains_owner_token_mapped(types, ty, &BTreeMap::new(), &mut BTreeSet::new())
}

fn type_contains_owner_token_mapped(
    types: &TypeCtx,
    ty: TypeId,
    mapping: &BTreeMap<TypeId, TypeId>,
    seen: &mut BTreeSet<TypeId>,
) -> bool {
    let mapped = mapped_type_id(types, ty, mapping);
    if type_is_owner_token(types, mapped) {
        return true;
    }
    if !seen.insert(mapped) {
        return false;
    }
    let contains = match types.get_ref(mapped) {
        TypeKind::Struct { .. } | TypeKind::Tuple { .. } => {
            aggregate_fields_with_offsets(types, mapped)
                .into_iter()
                .any(|field| type_contains_owner_token_mapped(types, field.ty, mapping, seen))
        }
        TypeKind::Enum { variants, .. } => variants.iter().any(|variant| {
            variant.payload.is_some_and(|payload| {
                type_contains_owner_token_mapped(types, payload, mapping, seen)
            })
        }),
        TypeKind::Apply { base, args } => {
            apply_contains_owner_token(types, *base, args, mapping, seen)
        }
        TypeKind::Var(var) => var
            .binding
            .is_some_and(|binding| type_contains_owner_token_mapped(types, binding, mapping, seen)),
        TypeKind::Box(inner) | TypeKind::Reference(inner, _) => {
            type_contains_owner_token_mapped(types, *inner, mapping, seen)
        }
        TypeKind::Unit
        | TypeKind::Never
        | TypeKind::I32
        | TypeKind::U8
        | TypeKind::F32
        | TypeKind::Bool
        | TypeKind::Char
        | TypeKind::Str
        | TypeKind::Function { .. } => false,
        TypeKind::Named(_) => {
            let named = types.resolve_named_type_id(mapped);
            named != mapped && type_contains_owner_token_mapped(types, named, mapping, seen)
        }
    };
    seen.remove(&mapped);
    contains
}

fn apply_contains_owner_token(
    types: &TypeCtx,
    base: TypeId,
    args: &[TypeId],
    mapping: &BTreeMap<TypeId, TypeId>,
    seen: &mut BTreeSet<TypeId>,
) -> bool {
    let base = types.resolve_named_type_id(base);
    match types.get_ref(base) {
        TypeKind::Struct { type_params, .. } => {
            let nested_mapping = extend_type_mapping(types, mapping, type_params, args);
            aggregate_fields_with_offsets(types, base)
                .into_iter()
                .any(|field| {
                    type_contains_owner_token_mapped(types, field.ty, &nested_mapping, seen)
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
                    type_contains_owner_token_mapped(types, payload, &nested_mapping, seen)
                })
            })
        }
        TypeKind::Tuple { .. } => aggregate_fields_with_offsets(types, base)
            .into_iter()
            .any(|field| type_contains_owner_token_mapped(types, field.ty, mapping, seen)),
        _ => type_contains_owner_token_mapped(types, base, mapping, seen),
    }
}

#[cfg(test)]
mod tests {
    use alloc::string::ToString;
    use alloc::vec;

    use crate::resource_primitives::{CompilerMemoryFieldSpec, OWNER_TOKEN_TYPE_NAME};
    use crate::source_map::CompilerMemoryType;
    use crate::types::{TypeCtx, TypeKind};

    use super::type_contains_owner_token;

    #[test]
    fn type_contains_owner_token_resolves_named_aliases() {
        let mut types = TypeCtx::new();
        let type_param = types.fresh_var(Some("T".to_string()));
        let i32_ty = types.i32();
        let token_ty = types.register_named(
            OWNER_TOKEN_TYPE_NAME.to_string(),
            TypeKind::Struct {
                name: OWNER_TOKEN_TYPE_NAME.to_string(),
                type_params: vec![type_param],
                fields: vec![i32_ty, i32_ty],
                field_names: vec![
                    CompilerMemoryFieldSpec::RawI32.name().to_string(),
                    CompilerMemoryFieldSpec::SizeI32.name().to_string(),
                ],
            },
        );
        types.mark_compiler_memory_type(token_ty, CompilerMemoryType::OwnerToken);
        let alias_ty = types.register_named(
            "AliasToken".to_string(),
            TypeKind::Named(OWNER_TOKEN_TYPE_NAME.to_string()),
        );

        assert!(type_contains_owner_token(&types, alias_ty));
    }
}
