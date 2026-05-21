extern crate alloc;

use alloc::vec::Vec;

use crate::hir::{FuncRef, HirExpr, HirExprKind};
use crate::span::Span;
use crate::types::TypeKind;

use super::lower::{place_from_expr_skeleton, LoweringContext, LoweringEnvironment};
use super::model::{Place, ResourceOp};

pub(super) fn push_source_drop_call_resource_proof(
    callee: &FuncRef,
    args: &[HirExpr],
    ops: &mut Vec<ResourceOp>,
    ctx: &LoweringContext,
    env: &LoweringEnvironment,
    span: Span,
) {
    if let Some(place) = source_drop_call_place(callee, args, ctx, env) {
        ops.push(ResourceOp::Drop { place, span });
    }
}

fn source_drop_call_place(
    callee: &FuncRef,
    args: &[HirExpr],
    ctx: &LoweringContext,
    env: &LoweringEnvironment,
) -> Option<Place> {
    let place = first_addr_of_arg_place(args, ctx)?;
    if !env.call_is_explicit_drop(callee) {
        return None;
    }
    if let FuncRef::Trait { self_ty, .. } = callee {
        if !env.types.same_type(place.ty, *self_ty) {
            return None;
        }
        return Some(place);
    }
    let first = args.first()?;
    let ref_ty = env
        .types
        .resolve_named_type_id(env.types.resolve_id(first.ty));
    match env.types.get_ref(ref_ty) {
        TypeKind::Reference(inner, _) if env.types.same_type(place.ty, *inner) => Some(place),
        _ => None,
    }
}

fn first_addr_of_arg_place(args: &[HirExpr], ctx: &LoweringContext) -> Option<Place> {
    let first = args.first()?;
    let HirExprKind::AddrOf(inner) = &first.kind else {
        return None;
    };
    let place = place_from_expr_skeleton(inner, ctx);
    if matches!(&place.root, super::model::PlaceRoot::Unknown) {
        return None;
    }
    Some(place)
}
