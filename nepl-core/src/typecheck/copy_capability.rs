use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;

use crate::resource_primitives::type_is_owner_token;
use crate::types::{TypeCtx, TypeId, TypeKind};

use super::model::{StructConstructorPolicy, StructInfo};

pub(super) fn target_is_compiler_owner_token(ctx: &TypeCtx, ty: TypeId) -> bool {
    type_is_owner_token(ctx, ty)
}

pub(super) fn mark_owner_backed_aggregate_constructor_policies(
    ctx: &TypeCtx,
    structs: &mut BTreeMap<String, StructInfo>,
) {
    loop {
        let owner_backed = structs
            .iter()
            .filter_map(|(name, info)| {
                (info.constructor_policy == StructConstructorPolicy::Public
                    && info
                        .fields
                        .iter()
                        .any(|field| target_contains_owner_backed_aggregate(ctx, structs, *field)))
                .then(|| name.clone())
            })
            .collect::<Vec<_>>();

        if owner_backed.is_empty() {
            break;
        }

        for name in owner_backed {
            if let Some(info) = structs.get_mut(&name) {
                info.constructor_policy = StructConstructorPolicy::OwnerBackedAggregateBoundaryOnly;
            }
        }
    }
}

pub(super) fn target_contains_owner_backed_aggregate(
    ctx: &TypeCtx,
    structs: &BTreeMap<String, StructInfo>,
    ty: TypeId,
) -> bool {
    target_contains_owner_backed_aggregate_mapped(
        ctx,
        structs,
        ty,
        &BTreeMap::new(),
        &mut Vec::new(),
    )
}

fn target_contains_owner_backed_aggregate_mapped(
    ctx: &TypeCtx,
    structs: &BTreeMap<String, StructInfo>,
    ty: TypeId,
    mapping: &BTreeMap<TypeId, TypeId>,
    seen: &mut Vec<TypeId>,
) -> bool {
    if let Some(mapped) = mapping.get(&ty) {
        return target_contains_owner_backed_aggregate_mapped(ctx, structs, *mapped, mapping, seen);
    }

    if target_is_compiler_owner_token(ctx, ty) {
        return true;
    }

    let resolved = ctx.resolve_id(ty);
    if let Some(mapped) = mapping.get(&resolved) {
        return target_contains_owner_backed_aggregate_mapped(ctx, structs, *mapped, mapping, seen);
    }
    if seen.contains(&resolved) {
        return false;
    }
    seen.push(resolved);

    let contains = match ctx.get_ref(resolved) {
        TypeKind::Apply { base, args } => {
            target_apply_contains_owner_backed_aggregate(ctx, structs, *base, args, mapping, seen)
        }
        TypeKind::Named(_) => {
            let named = ctx.resolve_named_type_id(resolved);
            named != resolved
                && target_contains_owner_backed_aggregate_mapped(ctx, structs, named, mapping, seen)
        }
        TypeKind::Struct { name, fields, .. } => {
            structs.get(name).is_some_and(|info| {
                ctx.same_type(info.ty, resolved)
                    && info.constructor_policy
                        == StructConstructorPolicy::OwnerBackedAggregateBoundaryOnly
            }) || fields.iter().any(|field| {
                target_contains_owner_backed_aggregate_mapped(ctx, structs, *field, mapping, seen)
            })
        }
        TypeKind::Enum { variants, .. } => variants.iter().any(|variant| {
            variant.payload.is_some_and(|payload| {
                target_contains_owner_backed_aggregate_mapped(ctx, structs, payload, mapping, seen)
            })
        }),
        TypeKind::Tuple { items } => items.iter().any(|item| {
            target_contains_owner_backed_aggregate_mapped(ctx, structs, *item, mapping, seen)
        }),
        TypeKind::Box(inner) => {
            target_contains_owner_backed_aggregate_mapped(ctx, structs, *inner, mapping, seen)
        }
        TypeKind::Unit
        | TypeKind::I32
        | TypeKind::U8
        | TypeKind::F32
        | TypeKind::Bool
        | TypeKind::Char
        | TypeKind::Str
        | TypeKind::Never
        | TypeKind::Function { .. }
        | TypeKind::Var(_)
        | TypeKind::Reference(_, _) => false,
    };

    seen.pop();
    contains
}

fn target_apply_contains_owner_backed_aggregate(
    ctx: &TypeCtx,
    structs: &BTreeMap<String, StructInfo>,
    base: TypeId,
    args: &[TypeId],
    mapping: &BTreeMap<TypeId, TypeId>,
    seen: &mut Vec<TypeId>,
) -> bool {
    let resolved_base = ctx.resolve_id(base);
    match ctx.get_ref(resolved_base) {
        TypeKind::Struct {
            name,
            type_params,
            fields,
            ..
        } => {
            if structs.get(name).is_some_and(|info| {
                ctx.same_type(info.ty, resolved_base)
                    && info.constructor_policy
                        == StructConstructorPolicy::OwnerBackedAggregateBoundaryOnly
            }) {
                return true;
            }
            let scoped_mapping = apply_mapping(mapping, type_params, args);
            fields.iter().any(|field| {
                target_contains_owner_backed_aggregate_mapped(
                    ctx,
                    structs,
                    *field,
                    &scoped_mapping,
                    seen,
                )
            })
        }
        TypeKind::Enum {
            type_params,
            variants,
            ..
        } => {
            let scoped_mapping = apply_mapping(mapping, type_params, args);
            variants.iter().any(|variant| {
                variant.payload.is_some_and(|payload| {
                    target_contains_owner_backed_aggregate_mapped(
                        ctx,
                        structs,
                        payload,
                        &scoped_mapping,
                        seen,
                    )
                })
            })
        }
        _ => {
            target_contains_owner_backed_aggregate_mapped(ctx, structs, base, mapping, seen)
                || args.iter().any(|arg| {
                    target_contains_owner_backed_aggregate_mapped(ctx, structs, *arg, mapping, seen)
                })
        }
    }
}

fn apply_mapping(
    mapping: &BTreeMap<TypeId, TypeId>,
    type_params: &[TypeId],
    args: &[TypeId],
) -> BTreeMap<TypeId, TypeId> {
    let mut scoped_mapping = mapping.clone();
    for (param, arg) in type_params.iter().zip(args.iter()) {
        scoped_mapping.insert(*param, *arg);
    }
    scoped_mapping
}
