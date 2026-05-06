extern crate alloc;

use alloc::collections::BTreeSet;
use alloc::string::String;
use alloc::vec::Vec;

use crate::hir::{HirBlock, HirExpr, HirExprKind, HirFunction, HirMatchArm, HirModule};
use crate::span::Span;

use super::drop_elaboration::ResourceDropElaborationPlan;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResourceDropElaborationHirBridgeError {
    MissingSourceFunction {
        function: String,
        origin_name: String,
    },
    MissingSourceBinding {
        function: String,
        origin_name: String,
        source_name: String,
        span: Span,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct HirDropBridgePoint {
    span: Span,
    source_names: BTreeSet<String>,
}

pub fn validate_resource_drop_elaboration_hir_bridge(
    module: &HirModule,
    plan: &ResourceDropElaborationPlan,
) -> Result<(), Vec<ResourceDropElaborationHirBridgeError>> {
    let mut errors = Vec::new();
    for function_plan in &plan.functions {
        if function_plan.auto_drops.is_empty() {
            continue;
        }
        let source_functions = module
            .functions
            .iter()
            .filter(|function| function.origin_name == function_plan.origin_name)
            .collect::<Vec<_>>();
        if source_functions.is_empty() {
            errors.push(
                ResourceDropElaborationHirBridgeError::MissingSourceFunction {
                    function: function_plan.name.clone(),
                    origin_name: function_plan.origin_name.clone(),
                },
            );
            continue;
        }

        let mut bridge_points = Vec::new();
        for function in source_functions {
            collect_function_bridge_points(function, &mut bridge_points);
        }
        for point in &function_plan.drop_points {
            for drop in &point.auto_drops {
                if !bridge_points.iter().any(|bridge| {
                    bridge.span == point.span && bridge.source_names.contains(&drop.source_name)
                }) {
                    errors.push(
                        ResourceDropElaborationHirBridgeError::MissingSourceBinding {
                            function: function_plan.name.clone(),
                            origin_name: function_plan.origin_name.clone(),
                            source_name: drop.source_name.clone(),
                            span: point.span,
                        },
                    );
                }
            }
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn collect_function_bridge_points(function: &HirFunction, out: &mut Vec<HirDropBridgePoint>) {
    let crate::hir::HirBody::Block(block) = &function.body else {
        return;
    };
    if !function.params.is_empty() {
        out.push(HirDropBridgePoint {
            span: block.span,
            source_names: function
                .params
                .iter()
                .map(|param| param.name.clone())
                .collect(),
        });
    }
    collect_block_bridge_points(block, out);
}

fn collect_block_bridge_points(block: &HirBlock, out: &mut Vec<HirDropBridgePoint>) {
    let mut source_names = BTreeSet::new();
    for line in &block.lines {
        collect_expr_bridge_points(&line.expr, out);
        if let HirExprKind::Let { name, .. } = &line.expr.kind {
            source_names.insert(name.clone());
        }
    }
    if !source_names.is_empty() {
        out.push(HirDropBridgePoint {
            span: block.span,
            source_names,
        });
    }
}

fn collect_expr_bridge_points(expr: &HirExpr, out: &mut Vec<HirDropBridgePoint>) {
    match &expr.kind {
        HirExprKind::If {
            cond,
            then_branch,
            else_branch,
        } => {
            collect_expr_bridge_points(cond, out);
            collect_expr_bridge_points(then_branch, out);
            collect_expr_bridge_points(else_branch, out);
        }
        HirExprKind::While { cond, body } => {
            collect_expr_bridge_points(cond, out);
            collect_expr_bridge_points(body, out);
        }
        HirExprKind::Match { scrutinee, arms } => {
            collect_expr_bridge_points(scrutinee, out);
            for arm in arms {
                collect_match_arm_bridge_points(arm, out);
            }
        }
        HirExprKind::Call { args, .. } | HirExprKind::Intrinsic { args, .. } => {
            for arg in args {
                collect_expr_bridge_points(arg, out);
            }
        }
        HirExprKind::CallIndirect { callee, args, .. } => {
            collect_expr_bridge_points(callee, out);
            for arg in args {
                collect_expr_bridge_points(arg, out);
            }
        }
        HirExprKind::EnumConstruct { payload, .. } => {
            if let Some(payload) = payload {
                collect_expr_bridge_points(payload, out);
            }
        }
        HirExprKind::StructConstruct { fields, .. } => {
            for field in fields {
                collect_expr_bridge_points(field, out);
            }
        }
        HirExprKind::TupleConstruct { items } => {
            for item in items {
                collect_expr_bridge_points(item, out);
            }
        }
        HirExprKind::Block(block) => collect_block_bridge_points(block, out),
        HirExprKind::Let { value, .. } | HirExprKind::Set { value, .. } => {
            collect_expr_bridge_points(value, out);
        }
        HirExprKind::AddrOf(inner) | HirExprKind::Deref(inner) => {
            collect_expr_bridge_points(inner, out);
        }
        HirExprKind::FnValue(_)
        | HirExprKind::Var(_)
        | HirExprKind::LiteralI32(_)
        | HirExprKind::LiteralF32(_)
        | HirExprKind::LiteralBool(_)
        | HirExprKind::LiteralStr(_)
        | HirExprKind::Unit
        | HirExprKind::Drop { .. } => {}
    }
}

fn collect_match_arm_bridge_points(arm: &HirMatchArm, out: &mut Vec<HirDropBridgePoint>) {
    if let Some(name) = &arm.bind_local {
        out.push(HirDropBridgePoint {
            span: arm.body.span,
            source_names: [name.clone()].into_iter().collect(),
        });
    }
    collect_expr_bridge_points(&arm.body, out);
}
