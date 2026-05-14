extern crate alloc;

use alloc::collections::VecDeque;
use alloc::vec;
use alloc::vec::Vec;

use super::model::ResourceModule;
use super::summary_dependency::build_function_summary_dependents;
use super::summary_worklist_order::initial_summary_order;

pub(super) struct SummaryWorklist {
    dependents: Vec<Vec<usize>>,
    pending: VecDeque<usize>,
    queued: Vec<bool>,
    max_recomputations: usize,
    recomputations: usize,
}

impl SummaryWorklist {
    pub(super) fn new(module: &ResourceModule) -> Self {
        let mut pending = VecDeque::new();
        let mut queued = vec![false; module.functions.len()];
        for index in initial_summary_order(module) {
            pending.push_back(index);
            queued[index] = true;
        }
        let max_recomputations = module
            .functions
            .len()
            .saturating_mul(module.functions.len().saturating_add(1));
        Self {
            dependents: build_function_summary_dependents(module),
            pending,
            queued,
            max_recomputations,
            recomputations: 0,
        }
    }

    pub(super) fn pop(&mut self) -> Option<usize> {
        if self.recomputations >= self.max_recomputations {
            return None;
        }
        let index = self.pending.pop_front()?;
        self.queued[index] = false;
        self.recomputations += 1;
        Some(index)
    }

    pub(super) fn notify_changed(&mut self, function_index: usize) {
        for dependent in &self.dependents[function_index] {
            if !self.queued[*dependent] {
                self.pending.push_back(*dependent);
                self.queued[*dependent] = true;
            }
        }
    }

    #[cfg(all(not(target_os = "none"), not(target_arch = "wasm32")))]
    pub(super) fn recomputations(&self) -> usize {
        self.recomputations
    }
}

#[cfg(test)]
#[path = "summary_worklist_tests.rs"]
mod tests;
