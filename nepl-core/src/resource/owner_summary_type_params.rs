use alloc::vec::Vec;

use crate::types::{TypeCtx, TypeId, TypeKind};

use super::model::ResourceFunction;

pub(super) fn owner_summary_type_params(
    types: &TypeCtx,
    function: &ResourceFunction,
) -> Vec<TypeId> {
    let mut out = function.type_params.clone();
    for param in &function.params {
        collect_type_vars(types, param.ty, &mut out, 0);
    }
    collect_type_vars(types, function.result, &mut out, 0);
    out
}

fn collect_type_vars(types: &TypeCtx, ty: TypeId, out: &mut Vec<TypeId>, depth: usize) {
    if depth > 32 {
        return;
    }
    match types.get_ref(ty) {
        TypeKind::Var(var) => {
            if let Some(binding) = var.binding {
                collect_type_vars(types, binding, out, depth + 1);
            } else if !out.contains(&ty) {
                out.push(ty);
            }
        }
        TypeKind::Apply { args, .. } => {
            for arg in args {
                collect_type_vars(types, *arg, out, depth + 1);
            }
        }
        TypeKind::Enum {
            type_params,
            variants,
            ..
        } => {
            for ty in type_params {
                collect_type_vars(types, *ty, out, depth + 1);
            }
            for variant in variants {
                if let Some(payload) = variant.payload {
                    collect_type_vars(types, payload, out, depth + 1);
                }
            }
        }
        TypeKind::Struct {
            type_params,
            fields,
            ..
        } => {
            for ty in type_params {
                collect_type_vars(types, *ty, out, depth + 1);
            }
            for field in fields {
                collect_type_vars(types, *field, out, depth + 1);
            }
        }
        TypeKind::Tuple { items } => {
            for item in items {
                collect_type_vars(types, *item, out, depth + 1);
            }
        }
        TypeKind::Function {
            type_params,
            params,
            result,
            ..
        } => {
            for ty in type_params {
                collect_type_vars(types, *ty, out, depth + 1);
            }
            for param in params {
                collect_type_vars(types, *param, out, depth + 1);
            }
            collect_type_vars(types, *result, out, depth + 1);
        }
        TypeKind::Box(inner) | TypeKind::Reference(inner, _) => {
            collect_type_vars(types, *inner, out, depth + 1);
        }
        TypeKind::Unit
        | TypeKind::I32
        | TypeKind::U8
        | TypeKind::F32
        | TypeKind::Bool
        | TypeKind::Char
        | TypeKind::Str
        | TypeKind::Never
        | TypeKind::Named(_) => {}
    }
}
