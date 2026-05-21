use crate::span::Span;

use super::drop_point_path::{ResourceDropPointPath, ResourceDropPointStep};
use super::model::{Place, ResourceBlockId, ResourceFunction, ResourceOp};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResourceDropPointEndScope<'a> {
    pub locals: &'a [Place],
    pub result: Option<&'a Place>,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResourceDropPointOpKind {
    Leaf,
    Branch,
    Loop,
    Match,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResourceDropPointResolutionError {
    BlockNotFound {
        block: ResourceBlockId,
    },
    OpIndexOutOfBounds {
        index: usize,
        len: usize,
    },
    ConsecutiveOpStep {
        index: usize,
    },
    ContainerStepWithoutSelectedOp {
        step: ResourceDropPointStep,
    },
    ContainerStepDoesNotMatchOp {
        step: ResourceDropPointStep,
        actual: ResourceDropPointOpKind,
    },
    MatchArmIndexOutOfBounds {
        index: usize,
        len: usize,
    },
    PathDoesNotSelectOp,
    PathDoesNotSelectEndScope {
        actual: ResourceDropPointOpKind,
    },
    PathDoesNotSelectAssignment {
        actual: ResourceDropPointOpKind,
    },
}

pub fn resolve_resource_drop_point_path<'a>(
    function: &'a ResourceFunction,
    path: &ResourceDropPointPath,
) -> Result<&'a ResourceOp, ResourceDropPointResolutionError> {
    let block = function
        .blocks
        .iter()
        .find(|block| block.id == path.block)
        .ok_or(ResourceDropPointResolutionError::BlockNotFound { block: path.block })?;
    resolve_steps(&block.ops, &path.steps)
}

pub fn resolve_resource_drop_point_end_scope<'a>(
    function: &'a ResourceFunction,
    path: &ResourceDropPointPath,
) -> Result<ResourceDropPointEndScope<'a>, ResourceDropPointResolutionError> {
    match resolve_resource_drop_point_path(function, path)? {
        ResourceOp::EndScope {
            locals,
            result,
            span,
        } => Ok(ResourceDropPointEndScope {
            locals,
            result: result.as_ref(),
            span: *span,
        }),
        op => Err(
            ResourceDropPointResolutionError::PathDoesNotSelectEndScope {
                actual: op_kind(op),
            },
        ),
    }
}

fn resolve_steps<'a>(
    mut ops: &'a [ResourceOp],
    steps: &[ResourceDropPointStep],
) -> Result<&'a ResourceOp, ResourceDropPointResolutionError> {
    let mut selected = None;
    for step in steps {
        match *step {
            ResourceDropPointStep::Op { index } => {
                if selected.is_some() {
                    return Err(ResourceDropPointResolutionError::ConsecutiveOpStep { index });
                }
                selected = Some(ops.get(index).ok_or(
                    ResourceDropPointResolutionError::OpIndexOutOfBounds {
                        index,
                        len: ops.len(),
                    },
                )?);
            }
            ResourceDropPointStep::BranchThen => {
                ops = enter_branch_ops(selected.take(), *step, |then_ops, _| then_ops)?;
            }
            ResourceDropPointStep::BranchElse => {
                ops = enter_branch_ops(selected.take(), *step, |_, else_ops| else_ops)?;
            }
            ResourceDropPointStep::LoopCondition => {
                ops = enter_loop_ops(selected.take(), *step, |condition_ops, _| condition_ops)?;
            }
            ResourceDropPointStep::LoopBody => {
                ops = enter_loop_ops(selected.take(), *step, |_, body_ops| body_ops)?;
            }
            ResourceDropPointStep::MatchArm { index } => {
                ops = enter_match_arm_ops(selected.take(), *step, index)?;
            }
        }
    }
    selected.ok_or(ResourceDropPointResolutionError::PathDoesNotSelectOp)
}

fn enter_branch_ops<'a>(
    selected: Option<&'a ResourceOp>,
    step: ResourceDropPointStep,
    choose: impl FnOnce(&'a [ResourceOp], &'a [ResourceOp]) -> &'a [ResourceOp],
) -> Result<&'a [ResourceOp], ResourceDropPointResolutionError> {
    match selected
        .ok_or(ResourceDropPointResolutionError::ContainerStepWithoutSelectedOp { step })?
    {
        ResourceOp::Branch {
            then_ops, else_ops, ..
        } => Ok(choose(then_ops, else_ops)),
        op => Err(
            ResourceDropPointResolutionError::ContainerStepDoesNotMatchOp {
                step,
                actual: op_kind(op),
            },
        ),
    }
}

fn enter_loop_ops<'a>(
    selected: Option<&'a ResourceOp>,
    step: ResourceDropPointStep,
    choose: impl FnOnce(&'a [ResourceOp], &'a [ResourceOp]) -> &'a [ResourceOp],
) -> Result<&'a [ResourceOp], ResourceDropPointResolutionError> {
    match selected
        .ok_or(ResourceDropPointResolutionError::ContainerStepWithoutSelectedOp { step })?
    {
        ResourceOp::Loop {
            condition_ops,
            body_ops,
            ..
        } => Ok(choose(condition_ops, body_ops)),
        op => Err(
            ResourceDropPointResolutionError::ContainerStepDoesNotMatchOp {
                step,
                actual: op_kind(op),
            },
        ),
    }
}

fn enter_match_arm_ops<'a>(
    selected: Option<&'a ResourceOp>,
    step: ResourceDropPointStep,
    index: usize,
) -> Result<&'a [ResourceOp], ResourceDropPointResolutionError> {
    match selected
        .ok_or(ResourceDropPointResolutionError::ContainerStepWithoutSelectedOp { step })?
    {
        ResourceOp::Match { arms, .. } => arms.get(index).map(|arm| arm.ops.as_slice()).ok_or(
            ResourceDropPointResolutionError::MatchArmIndexOutOfBounds {
                index,
                len: arms.len(),
            },
        ),
        op => Err(
            ResourceDropPointResolutionError::ContainerStepDoesNotMatchOp {
                step,
                actual: op_kind(op),
            },
        ),
    }
}

pub(super) fn op_kind(op: &ResourceOp) -> ResourceDropPointOpKind {
    match op {
        ResourceOp::Branch { .. } => ResourceDropPointOpKind::Branch,
        ResourceOp::Loop { .. } => ResourceDropPointOpKind::Loop,
        ResourceOp::Match { .. } => ResourceDropPointOpKind::Match,
        ResourceOp::Expr { .. }
        | ResourceOp::DeclareLocal { .. }
        | ResourceOp::Read { .. }
        | ResourceOp::Assign { .. }
        | ResourceOp::Borrow { .. }
        | ResourceOp::Move { .. }
        | ResourceOp::Drop { .. }
        | ResourceOp::EndScope { .. }
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
        | ResourceOp::Construct { .. } => ResourceDropPointOpKind::Leaf,
    }
}
