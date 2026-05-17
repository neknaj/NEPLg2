use alloc::{vec, vec::Vec};

use crate::hir::{HirExpr, HirExprKind};

use super::lower::{place_from_expr_skeleton, LoweringContext};
use super::lower_call::func_ref_base_name;
use super::model::{Place, PlaceRoot, ResourceConditionFact, ResourceI32RelationOp};
use super::scalar_primitive::{BooleanPrimitive, I32ComparisonPrimitive};

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
    if let Some(boolean) = BooleanPrimitive::from_base_name(comparison) {
        let fact = match boolean {
            BooleanPrimitive::Or => ResourceConditionFact::Any,
            BooleanPrimitive::And => ResourceConditionFact::All,
        };
        return binary_condition_fact(left, right, ctx, fact);
    }
    let comparison = I32ComparisonPrimitive::from_base_name(comparison)?;
    comparison_value_condition_fact(comparison, left, right, ctx)
        .or_else(|| relation_condition_fact(left, right, ctx, comparison.relation_op()))
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
    comparison: I32ComparisonPrimitive,
    left: &HirExpr,
    right: &HirExpr,
    ctx: &LoweringContext,
) -> Option<ResourceConditionFact> {
    let place = if literal_i32_is_zero(left) {
        condition_place(right, ctx)?
    } else if literal_i32_is_zero(right) {
        condition_place(left, ctx)?
    } else {
        return None;
    };
    Some(match comparison {
        I32ComparisonPrimitive::Eq => ResourceConditionFact::EqZero { place },
        I32ComparisonPrimitive::Ne => ResourceConditionFact::NeZero { place },
        I32ComparisonPrimitive::Lt
        | I32ComparisonPrimitive::Le
        | I32ComparisonPrimitive::Gt
        | I32ComparisonPrimitive::Ge => return None,
    })
}

fn relation_condition_fact(
    left: &HirExpr,
    right: &HirExpr,
    ctx: &LoweringContext,
    op: ResourceI32RelationOp,
) -> Option<ResourceConditionFact> {
    let left = condition_place(left, ctx)?;
    let right = condition_place(right, ctx)?;
    Some(ResourceConditionFact::I32Relation { left, op, right })
}

fn comparison_value_condition_fact(
    comparison: I32ComparisonPrimitive,
    left: &HirExpr,
    right: &HirExpr,
    ctx: &LoweringContext,
) -> Option<ResourceConditionFact> {
    match comparison {
        I32ComparisonPrimitive::Eq | I32ComparisonPrimitive::Ne => {
            zero_comparison_fact(comparison, left, right, ctx)
        }
        I32ComparisonPrimitive::Lt if literal_i32_is_zero(left) => {
            condition_place(right, ctx).map(|place| ResourceConditionFact::Positive { place })
        }
        I32ComparisonPrimitive::Lt if literal_i32_is_zero(right) => {
            condition_place(left, ctx).map(|place| ResourceConditionFact::Negative { place })
        }
        I32ComparisonPrimitive::Lt if matches!(right.kind, HirExprKind::LiteralI32(1)) => {
            condition_place(left, ctx).map(|place| ResourceConditionFact::NonPositive { place })
        }
        I32ComparisonPrimitive::Le if literal_i32_is_zero(right) => {
            condition_place(left, ctx).map(|place| ResourceConditionFact::NonPositive { place })
        }
        I32ComparisonPrimitive::Le if literal_i32_is_zero(left) => {
            condition_place(right, ctx).map(|place| ResourceConditionFact::NonNegative { place })
        }
        I32ComparisonPrimitive::Gt if literal_i32_is_zero(left) => {
            condition_place(right, ctx).map(|place| ResourceConditionFact::Negative { place })
        }
        I32ComparisonPrimitive::Gt if literal_i32_is_zero(right) => {
            condition_place(left, ctx).map(|place| ResourceConditionFact::Positive { place })
        }
        I32ComparisonPrimitive::Ge if matches!(right.kind, HirExprKind::LiteralI32(1)) => {
            condition_place(left, ctx).map(|place| ResourceConditionFact::Positive { place })
        }
        I32ComparisonPrimitive::Ge if literal_i32_is_zero(right) => {
            condition_place(left, ctx).map(|place| ResourceConditionFact::NonNegative { place })
        }
        I32ComparisonPrimitive::Lt
        | I32ComparisonPrimitive::Le
        | I32ComparisonPrimitive::Gt
        | I32ComparisonPrimitive::Ge => None,
    }
}

fn condition_place(expr: &HirExpr, ctx: &LoweringContext) -> Option<Place> {
    let place = place_from_expr_skeleton(expr, ctx);
    (!matches!(place.root, PlaceRoot::Unknown)).then_some(place)
}

fn literal_i32_is_zero(expr: &HirExpr) -> bool {
    matches!(expr.kind, HirExprKind::LiteralI32(0))
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
