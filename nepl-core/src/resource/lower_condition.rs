use alloc::vec;
use alloc::vec::Vec;

use crate::hir::{HirExpr, HirExprKind};

use super::lower::{func_ref_base_name, LoweringContext};
use super::model::{Place, PlaceProjection, PlaceRoot, ResourceConditionFact, ResourceOffset};

pub(super) fn resource_condition_fact(
    cond: &HirExpr,
    ctx: &LoweringContext,
) -> Option<ResourceConditionFact> {
    let cond = condition_value_expr(cond)?;
    let HirExprKind::Call { callee, args } = &cond.kind else {
        return None;
    };
    let comparison = func_ref_base_name(callee)?;
    let [left, right] = args.as_slice() else {
        return None;
    };
    match comparison {
        "or" => binary_condition_fact(left, right, ctx, ResourceConditionFact::Any),
        "and" => binary_condition_fact(left, right, ctx, ResourceConditionFact::All),
        "eq" => zero_comparison_fact(left, right, ctx, |place| ResourceConditionFact::EqZero {
            place,
        }),
        "ne" => zero_comparison_fact(left, right, ctx, |place| ResourceConditionFact::NeZero {
            place,
        }),
        "lt" if literal_i32_is_zero(left) => {
            condition_place(right, ctx).map(|place| ResourceConditionFact::Positive { place })
        }
        "lt" if literal_i32_is_zero(right) => {
            condition_place(left, ctx).map(|place| ResourceConditionFact::Negative { place })
        }
        "lt" if literal_i32_is_one(right) => {
            condition_place(left, ctx).map(|place| ResourceConditionFact::NonPositive { place })
        }
        "le" if literal_i32_is_zero(right) => {
            condition_place(left, ctx).map(|place| ResourceConditionFact::NonPositive { place })
        }
        "le" if literal_i32_is_zero(left) => {
            condition_place(right, ctx).map(|place| ResourceConditionFact::NonNegative { place })
        }
        "gt" if literal_i32_is_zero(left) => {
            condition_place(right, ctx).map(|place| ResourceConditionFact::Negative { place })
        }
        "gt" if literal_i32_is_zero(right) => {
            condition_place(left, ctx).map(|place| ResourceConditionFact::Positive { place })
        }
        "ge" if literal_i32_is_one(right) => {
            condition_place(left, ctx).map(|place| ResourceConditionFact::Positive { place })
        }
        "ge" if literal_i32_is_zero(right) => {
            condition_place(left, ctx).map(|place| ResourceConditionFact::NonNegative { place })
        }
        _ => None,
    }
}

pub(super) fn place_from_expr_skeleton(expr: &HirExpr, ctx: &LoweringContext) -> Place {
    match &expr.kind {
        HirExprKind::Var(name) => ctx.local_place(name, expr.ty),
        HirExprKind::Deref(inner) => {
            let source = place_from_expr_skeleton(inner, ctx);
            if matches!(&source.root, PlaceRoot::Unknown) {
                Place::unknown(expr.ty)
            } else {
                source.with_projection(PlaceProjection::Deref, expr.ty)
            }
        }
        HirExprKind::Intrinsic { name, args, .. } if name == "add" && !args.is_empty() => {
            let source = place_from_expr_skeleton(&args[0], ctx);
            if matches!(&source.root, PlaceRoot::Unknown) {
                return Place::unknown(expr.ty);
            }
            let bytes = args.get(1).and_then(|offset| match &offset.kind {
                HirExprKind::LiteralI32(value) if *value >= 0 => Some(*value as usize),
                _ => None,
            });
            source.with_projection(
                PlaceProjection::StorageOffset(ResourceOffset { bytes }),
                expr.ty,
            )
        }
        _ => Place::unknown(expr.ty),
    }
}

fn binary_condition_fact(
    left: &HirExpr,
    right: &HirExpr,
    ctx: &LoweringContext,
    fact: fn(Vec<ResourceConditionFact>) -> ResourceConditionFact,
) -> Option<ResourceConditionFact> {
    let left = resource_condition_fact(left, ctx)?;
    let right = resource_condition_fact(right, ctx)?;
    Some(fact(vec![left, right]))
}

fn zero_comparison_fact(
    left: &HirExpr,
    right: &HirExpr,
    ctx: &LoweringContext,
    fact: fn(Place) -> ResourceConditionFact,
) -> Option<ResourceConditionFact> {
    if literal_i32_is_zero(left) {
        condition_place(right, ctx).map(fact)
    } else if literal_i32_is_zero(right) {
        condition_place(left, ctx).map(fact)
    } else {
        None
    }
}

fn condition_place(expr: &HirExpr, ctx: &LoweringContext) -> Option<Place> {
    let place = place_from_expr_skeleton(expr, ctx);
    (!matches!(place.root, PlaceRoot::Unknown)).then_some(place)
}

fn literal_i32_is_zero(expr: &HirExpr) -> bool {
    matches!(expr.kind, HirExprKind::LiteralI32(0))
}

fn literal_i32_is_one(expr: &HirExpr) -> bool {
    matches!(expr.kind, HirExprKind::LiteralI32(1))
}

fn condition_value_expr(expr: &HirExpr) -> Option<&HirExpr> {
    match &expr.kind {
        HirExprKind::Block(block) => block
            .lines
            .iter()
            .rev()
            .find(|line| !line.drop_result)
            .and_then(|line| condition_value_expr(&line.expr)),
        _ => Some(expr),
    }
}
