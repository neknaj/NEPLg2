use crate::types::{TypeCtx, TypeId, TypeKind};

pub(super) fn type_contains_unbound_var(types: &TypeCtx, ty: TypeId) -> bool {
    type_contains_unbound_var_inner(types, ty, 0)
}

fn type_contains_unbound_var_inner(types: &TypeCtx, ty: TypeId, depth: usize) -> bool {
    if depth > 32 {
        return true;
    }
    match types.get_ref(ty) {
        TypeKind::Var(var) => var
            .binding
            .map(|binding| type_contains_unbound_var_inner(types, binding, depth + 1))
            .unwrap_or(true),
        TypeKind::Apply { args, .. } => args
            .iter()
            .any(|arg| type_contains_unbound_var_inner(types, *arg, depth + 1)),
        TypeKind::Enum {
            type_params,
            variants,
            ..
        } => {
            type_params
                .iter()
                .any(|ty| type_contains_unbound_var_inner(types, *ty, depth + 1))
                || variants.iter().any(|variant| {
                    variant
                        .payload
                        .is_some_and(|ty| type_contains_unbound_var_inner(types, ty, depth + 1))
                })
        }
        TypeKind::Struct {
            type_params,
            fields,
            ..
        } => {
            type_params
                .iter()
                .any(|ty| type_contains_unbound_var_inner(types, *ty, depth + 1))
                || fields
                    .iter()
                    .any(|ty| type_contains_unbound_var_inner(types, *ty, depth + 1))
        }
        TypeKind::Tuple { items } => items
            .iter()
            .any(|ty| type_contains_unbound_var_inner(types, *ty, depth + 1)),
        TypeKind::Function {
            type_params,
            params,
            result,
            ..
        } => {
            type_params
                .iter()
                .any(|ty| type_contains_unbound_var_inner(types, *ty, depth + 1))
                || params
                    .iter()
                    .any(|ty| type_contains_unbound_var_inner(types, *ty, depth + 1))
                || type_contains_unbound_var_inner(types, *result, depth + 1)
        }
        TypeKind::Box(inner) | TypeKind::Reference(inner, _) => {
            type_contains_unbound_var_inner(types, *inner, depth + 1)
        }
        TypeKind::Unit
        | TypeKind::I32
        | TypeKind::U8
        | TypeKind::F32
        | TypeKind::Bool
        | TypeKind::Char
        | TypeKind::Str
        | TypeKind::Never
        | TypeKind::Named(_) => false,
    }
}
