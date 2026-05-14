use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;

use crate::types::{TypeCtx, TypeId, TypeKind};

use super::model::{RestrictedStructConstructor, StructConstructorPolicy, StructInfo};

pub(super) fn target_is_compiler_owner_token(
    ctx: &TypeCtx,
    structs: &BTreeMap<String, StructInfo>,
    ty: TypeId,
) -> bool {
    let resolved = ctx.resolve_id(ty);
    match ctx.get_ref(resolved) {
        TypeKind::Apply { base, .. } => target_is_compiler_owner_token(ctx, structs, *base),
        TypeKind::Named(_) => {
            let named = ctx.resolve_named_type_id(resolved);
            named != resolved && target_is_compiler_owner_token(ctx, structs, named)
        }
        TypeKind::Struct { name, .. } => structs.get(name).is_some_and(|info| {
            ctx.same_type(info.ty, resolved)
                && matches!(
                    info.constructor_policy,
                    StructConstructorPolicy::RawMemoryBoundaryOnly(
                        RestrictedStructConstructor::OwnerToken
                    )
                )
        }),
        _ => false,
    }
}

pub(super) fn mark_owner_backed_aggregate_constructor_policies(
    ctx: &TypeCtx,
    structs: &mut BTreeMap<String, StructInfo>,
) {
    let owner_backed = structs
        .iter()
        .filter_map(|(name, info)| {
            (info.constructor_policy == StructConstructorPolicy::Public
                && info
                    .fields
                    .iter()
                    .any(|field| target_is_compiler_owner_token(ctx, structs, *field)))
            .then(|| name.clone())
        })
        .collect::<Vec<_>>();

    for name in owner_backed {
        if let Some(info) = structs.get_mut(&name) {
            info.constructor_policy = StructConstructorPolicy::OwnerBackedAggregateBoundaryOnly;
        }
    }
}
