use crate::types::{TypeCtx, TypeId, TypeKind};

pub(super) fn enum_variant_tag(ctx: &TypeCtx, enum_ty: TypeId, variant: &str) -> u32 {
    let name = crate::qualified_name::member_tail(variant);
    let enum_ty = ctx.resolve_named_type_id(enum_ty);
    match ctx.get(enum_ty) {
        TypeKind::Enum { variants, .. } => variants
            .iter()
            .position(|v| v.name == name)
            .map(|i| i as u32)
            .unwrap_or(0),
        TypeKind::Apply { base, .. } => enum_variant_tag(ctx, base, name),
        _ => 0,
    }
}

pub(super) fn enum_match_type(ctx: &TypeCtx, ty: TypeId) -> Option<TypeId> {
    let ty = ctx.resolve_named_type_id(ctx.resolve_id(ty));
    match ctx.get(ty) {
        TypeKind::Enum { .. } => Some(ty),
        TypeKind::Apply { base, .. } => {
            if matches!(
                ctx.get(ctx.resolve_named_type_id(base)),
                TypeKind::Enum { .. }
            ) {
                Some(ty)
            } else {
                None
            }
        }
        TypeKind::Reference(target, _) => enum_match_type(ctx, target),
        _ => None,
    }
}

pub(super) fn enum_variant_payload(
    ctx: &TypeCtx,
    enum_ty: TypeId,
    variant: &str,
) -> Option<TypeId> {
    let name = crate::qualified_name::member_tail(variant);
    let enum_ty = ctx.resolve_named_type_id(enum_ty);
    match ctx.get(enum_ty) {
        TypeKind::Enum { variants, .. } => variants
            .iter()
            .find(|v| v.name == name)
            .and_then(|v| v.payload),
        TypeKind::Apply { base, args } => match ctx.get(ctx.resolve_named_type_id(base)) {
            TypeKind::Enum {
                variants,
                type_params,
                ..
            } => {
                let payload = variants
                    .iter()
                    .find(|v| v.name == name)
                    .and_then(|v| v.payload);
                payload.map(|pty| {
                    if let Some(pos) = type_params.iter().position(|tp| *tp == pty) {
                        if let Some(arg) = args.get(pos) {
                            return *arg;
                        }
                    }
                    pty
                })
            }
            _ => None,
        },
        _ => None,
    }
}
