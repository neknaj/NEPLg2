extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;

use crate::span::Span;
use crate::types::TypeCtx;

use super::drop_requirement::{resource_drop_requirement_for_type, ResourceDropRequirement};
use super::model::{Place, ResourceFunction, ResourceModule, ResourceOp};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceDropPlan {
    pub functions: Vec<ResourceDropFunctionPlan>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceDropFunctionPlan {
    pub name: String,
    pub auto_drops: Vec<ResourceAutoDrop>,
    pub drop_points: Vec<ResourceDropPoint>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceDropPoint {
    pub span: Span,
    pub auto_drops: Vec<ResourceAutoDrop>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceAutoDrop {
    pub place: Place,
    pub kind: ResourceAutoDropKind,
    pub requirement: ResourceDropRequirement,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResourceAutoDropKind {
    ScopeLocal,
}

pub fn compute_resource_drop_plan(module: &ResourceModule, types: &TypeCtx) -> ResourceDropPlan {
    ResourceDropPlan {
        functions: module
            .functions
            .iter()
            .map(|function| compute_function_drop_plan(function, types))
            .collect(),
    }
}

pub(super) fn auto_drop_candidates_for_end_scope(
    types: &TypeCtx,
    locals: &[Place],
    span: Span,
) -> Vec<ResourceAutoDrop> {
    locals
        .iter()
        .rev()
        .filter(|local| !types.is_copy(local.ty))
        .map(|local| ResourceAutoDrop {
            place: local.clone(),
            kind: ResourceAutoDropKind::ScopeLocal,
            requirement: resource_drop_requirement_for_type(types, local.ty),
            span,
        })
        .collect()
}

fn compute_function_drop_plan(
    function: &ResourceFunction,
    types: &TypeCtx,
) -> ResourceDropFunctionPlan {
    let mut drop_points = Vec::new();
    for block in &function.blocks {
        collect_drop_points_from_ops(&block.ops, types, &mut drop_points);
    }
    let auto_drops = drop_points
        .iter()
        .flat_map(|point| point.auto_drops.iter().cloned())
        .collect();
    ResourceDropFunctionPlan {
        name: function.name.clone(),
        auto_drops,
        drop_points,
    }
}

fn collect_drop_points_from_ops(
    ops: &[ResourceOp],
    types: &TypeCtx,
    drop_points: &mut Vec<ResourceDropPoint>,
) {
    for op in ops {
        match op {
            ResourceOp::EndScope { locals, span, .. } => {
                let auto_drops = auto_drop_candidates_for_end_scope(types, locals, *span);
                if !auto_drops.is_empty() {
                    drop_points.push(ResourceDropPoint {
                        span: *span,
                        auto_drops,
                    });
                }
            }
            ResourceOp::Branch {
                then_ops, else_ops, ..
            } => {
                collect_drop_points_from_ops(then_ops, types, drop_points);
                collect_drop_points_from_ops(else_ops, types, drop_points);
            }
            ResourceOp::Loop {
                condition_ops,
                body_ops,
                ..
            } => {
                collect_drop_points_from_ops(condition_ops, types, drop_points);
                collect_drop_points_from_ops(body_ops, types, drop_points);
            }
            ResourceOp::Match { arms, .. } => {
                for arm in arms {
                    collect_drop_points_from_ops(&arm.ops, types, drop_points);
                }
            }
            ResourceOp::Expr { .. }
            | ResourceOp::DeclareLocal { .. }
            | ResourceOp::Read { .. }
            | ResourceOp::Assign { .. }
            | ResourceOp::Borrow { .. }
            | ResourceOp::Move { .. }
            | ResourceOp::Drop { .. }
            | ResourceOp::CallEffect { .. }
            | ResourceOp::FunctionValue { .. }
            | ResourceOp::Call { .. }
            | ResourceOp::IndirectCall { .. }
            | ResourceOp::RawMemory { .. }
            | ResourceOp::RawAddressAlias { .. }
            | ResourceOp::RawAddressView { .. }
            | ResourceOp::Construct { .. } => {}
        }
    }
}
