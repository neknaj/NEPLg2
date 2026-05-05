extern crate alloc;

use alloc::vec;
use alloc::vec::Vec;

use crate::hir::{HirExpr, HirExprKind};

use super::lower::{func_ref_base_name, place_from_expr_skeleton, LoweringContext};
use super::model::{Place, PlaceRoot, ResourceConditionFact};

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
