extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;

use crate::span::Span;
use crate::types::TypeCtx;

use super::model::{Place, ResourceFunction, ResourceModule, ResourceOp};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceDropPlan {
    pub functions: Vec<ResourceDropFunctionPlan>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceDropFunctionPlan {
    pub name: String,
    pub auto_drops: Vec<ResourceAutoDrop>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceAutoDrop {
    pub place: Place,
    pub kind: ResourceAutoDropKind,
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
            span,
        })
        .collect()
}

fn compute_function_drop_plan(
    function: &ResourceFunction,
    types: &TypeCtx,
) -> ResourceDropFunctionPlan {
    let mut auto_drops = Vec::new();
    for block in &function.blocks {
        collect_auto_drops_from_ops(&block.ops, types, &mut auto_drops);
    }
    ResourceDropFunctionPlan {
        name: function.name.clone(),
        auto_drops,
    }
}

fn collect_auto_drops_from_ops(
    ops: &[ResourceOp],
    types: &TypeCtx,
    auto_drops: &mut Vec<ResourceAutoDrop>,
) {
    for op in ops {
        match op {
            ResourceOp::EndScope { locals, span, .. } => {
                auto_drops.extend(auto_drop_candidates_for_end_scope(types, locals, *span));
            }
            ResourceOp::Branch {
                then_ops, else_ops, ..
            } => {
                collect_auto_drops_from_ops(then_ops, types, auto_drops);
                collect_auto_drops_from_ops(else_ops, types, auto_drops);
            }
            ResourceOp::Loop {
                condition_ops,
                body_ops,
                ..
            } => {
                collect_auto_drops_from_ops(condition_ops, types, auto_drops);
                collect_auto_drops_from_ops(body_ops, types, auto_drops);
            }
            ResourceOp::Match { arms, .. } => {
                for arm in arms {
                    collect_auto_drops_from_ops(&arm.ops, types, auto_drops);
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
