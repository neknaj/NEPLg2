extern crate alloc;

use alloc::vec::Vec;

use super::model::ResourceBlockId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceDropPointPath {
    pub block: ResourceBlockId,
    pub steps: Vec<ResourceDropPointStep>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResourceDropPointStep {
    Op { index: usize },
    BranchThen,
    BranchElse,
    LoopCondition,
    LoopBody,
    MatchArm { index: usize },
}

impl ResourceDropPointPath {
    pub(super) fn with_step(mut self, step: ResourceDropPointStep) -> Self {
        self.steps.push(step);
        self
    }
}
