use alloc::vec::Vec;

use crate::span::Span;
use crate::types::TypeCtx;

use super::drop_model::{
    ResourceAutoDrop, ResourceAutoDropKind, ResourceDropFunctionPlan, ResourceDropPlan,
    ResourceDropPoint,
};
use super::drop_plan_assignment::assignment_overwrite_drop_point;
use super::drop_point_path::{ResourceDropPointPath, ResourceDropPointStep};
use super::drop_requirement::resource_drop_requirement_for_type;
use super::model::{Place, ResourceFunction, ResourceModule, ResourceOp};

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
        let path = ResourceDropPointPath {
            block: block.id,
            steps: Vec::new(),
        };
        collect_drop_points_from_ops(&block.ops, types, path, &mut drop_points);
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
    path: ResourceDropPointPath,
    drop_points: &mut Vec<ResourceDropPoint>,
) {
    for (index, op) in ops.iter().enumerate() {
        let op_path = path.clone().with_step(ResourceDropPointStep::Op { index });
        match op {
            ResourceOp::EndScope { locals, span, .. } => {
                let auto_drops = auto_drop_candidates_for_end_scope(types, locals, *span);
                if !auto_drops.is_empty() {
                    drop_points.push(ResourceDropPoint {
                        path: op_path,
                        span: *span,
                        auto_drops,
                    });
                }
            }
            ResourceOp::Assign { target, span, .. } => {
                if let Some(drop_point) =
                    assignment_overwrite_drop_point(types, target, op_path, *span)
                {
                    drop_points.push(drop_point);
                }
            }
            ResourceOp::Branch {
                then_ops, else_ops, ..
            } => {
                collect_drop_points_from_ops(
                    then_ops,
                    types,
                    op_path.clone().with_step(ResourceDropPointStep::BranchThen),
                    drop_points,
                );
                collect_drop_points_from_ops(
                    else_ops,
                    types,
                    op_path.with_step(ResourceDropPointStep::BranchElse),
                    drop_points,
                );
            }
            ResourceOp::Loop {
                condition_ops,
                body_ops,
                ..
            } => {
                collect_drop_points_from_ops(
                    condition_ops,
                    types,
                    op_path
                        .clone()
                        .with_step(ResourceDropPointStep::LoopCondition),
                    drop_points,
                );
                collect_drop_points_from_ops(
                    body_ops,
                    types,
                    op_path.with_step(ResourceDropPointStep::LoopBody),
                    drop_points,
                );
            }
            ResourceOp::Match { arms, .. } => {
                for (arm_index, arm) in arms.iter().enumerate() {
                    collect_drop_points_from_ops(
                        &arm.ops,
                        types,
                        op_path
                            .clone()
                            .with_step(ResourceDropPointStep::MatchArm { index: arm_index }),
                        drop_points,
                    );
                }
            }
            ResourceOp::Expr { .. }
            | ResourceOp::DeclareLocal { .. }
            | ResourceOp::Read { .. }
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
            | ResourceOp::StorageOrigin { .. }
            | ResourceOp::CollectionSlotLifecycle { .. }
            | ResourceOp::CollectionStorageRelocate { .. }
            | ResourceOp::CollectionSlotDropTraversal { .. }
            | ResourceOp::Construct { .. } => {}
        }
    }
}
