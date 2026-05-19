use crate::types::{TypeCtx, TypeId, TypeKind};

pub(super) fn return_type_may_have_variant_param_summary(types: &TypeCtx, ty: TypeId) -> bool {
    let resolved = types.resolve_named_type_id(types.resolve_id(ty));
    match types.get_ref(resolved) {
        TypeKind::Enum { .. } => true,
        TypeKind::Apply { base, .. } => type_application_base_may_be_enum(types, *base),
        TypeKind::Named(_) | TypeKind::Var(_) => true,
        TypeKind::Unit
        | TypeKind::I32
        | TypeKind::U8
        | TypeKind::F32
        | TypeKind::Bool
        | TypeKind::Char
        | TypeKind::Str
        | TypeKind::Never
        | TypeKind::Struct { .. }
        | TypeKind::Tuple { .. }
        | TypeKind::Function { .. }
        | TypeKind::Box(_)
        | TypeKind::Reference(_, _) => false,
    }
}

fn type_application_base_may_be_enum(types: &TypeCtx, base: TypeId) -> bool {
    let resolved = types.resolve_named_type_id(types.resolve_id(base));
    match types.get_ref(resolved) {
        TypeKind::Enum { .. } => true,
        TypeKind::Named(_) | TypeKind::Var(_) => true,
        _ => false,
    }
}
