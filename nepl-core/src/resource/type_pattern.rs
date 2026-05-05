use alloc::vec::Vec;

use crate::types::{TypeCtx, TypeId, TypeKind};

pub(super) fn field_type_matches_result(
    types: &TypeCtx,
    field_ty: TypeId,
    result_ty: TypeId,
) -> bool {
    type_pattern_matches(types, field_ty, result_ty)
}

pub(super) fn type_pattern_matches(types: &TypeCtx, pattern_ty: TypeId, result_ty: TypeId) -> bool {
    types.same_type(pattern_ty, result_ty)
        || type_pattern_matches_result(types, pattern_ty, result_ty, &mut Vec::new())
}

fn type_pattern_matches_result(
    types: &TypeCtx,
    pattern_ty: TypeId,
    result_ty: TypeId,
    seen: &mut Vec<(TypeId, TypeId)>,
) -> bool {
    let pattern_ty = types.resolve_named_type_id(types.resolve_id(pattern_ty));
    let result_ty = types.resolve_named_type_id(types.resolve_id(result_ty));
    if pattern_ty == result_ty {
        return true;
    }
    if seen.contains(&(pattern_ty, result_ty)) {
        return true;
    }
    seen.push((pattern_ty, result_ty));
    let matches = match (types.get_ref(pattern_ty), types.get_ref(result_ty)) {
        (TypeKind::Var(var), _) if var.binding.is_none() => true,
        (_, TypeKind::Var(var)) if var.binding.is_none() => true,
        (TypeKind::Var(var), _) => var
            .binding
            .map(|binding| type_pattern_matches_result(types, binding, result_ty, seen))
            .unwrap_or(false),
        (_, TypeKind::Var(var)) => var
            .binding
            .map(|binding| type_pattern_matches_result(types, pattern_ty, binding, seen))
            .unwrap_or(false),
        (
            TypeKind::Apply { base, args },
            TypeKind::Apply {
                base: result_base,
                args: result_args,
            },
        ) => {
            args.len() == result_args.len()
                && type_pattern_matches_result(types, *base, *result_base, seen)
                && args
                    .iter()
                    .zip(result_args.iter())
                    .all(|(arg, result_arg)| {
                        type_pattern_matches_result(types, *arg, *result_arg, seen)
                    })
        }
        (TypeKind::Box(inner), TypeKind::Box(result_inner)) => {
            type_pattern_matches_result(types, *inner, *result_inner, seen)
        }
        (
            TypeKind::Reference(inner, mutable),
            TypeKind::Reference(result_inner, result_mutable),
        ) => {
            mutable == result_mutable
                && type_pattern_matches_result(types, *inner, *result_inner, seen)
        }
        (
            TypeKind::Tuple { items },
            TypeKind::Tuple {
                items: result_items,
            },
        ) => {
            items.len() == result_items.len()
                && items
                    .iter()
                    .zip(result_items.iter())
                    .all(|(item, result_item)| {
                        type_pattern_matches_result(types, *item, *result_item, seen)
                    })
        }
        _ => false,
    };
    seen.pop();
    matches
}
