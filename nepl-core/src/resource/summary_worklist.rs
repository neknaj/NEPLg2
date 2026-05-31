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
    relevant: Vec<bool>,
    max_recomputations: usize,
    recomputations: usize,
}

impl SummaryWorklist {
    pub(super) fn new(module: &ResourceModule) -> Self {
        Self::new_filtered(module, vec![true; module.functions.len()])
    }

    pub(super) fn new_filtered(module: &ResourceModule, relevant: Vec<bool>) -> Self {
        Self::new_filtered_with_initial_skips(module, relevant, vec![false; module.functions.len()])
    }

    pub(super) fn new_filtered_with_initial_skips(
        module: &ResourceModule,
        relevant: Vec<bool>,
        initially_skipped: Vec<bool>,
    ) -> Self {
        debug_assert_eq!(relevant.len(), module.functions.len());
        debug_assert_eq!(initially_skipped.len(), module.functions.len());
        let mut pending = VecDeque::new();
        let mut queued = vec![false; module.functions.len()];
        for index in initial_summary_order(module) {
            if relevant[index] && !initially_skipped[index] {
                pending.push_back(index);
                queued[index] = true;
            }
        }
        let relevant_function_count = relevant.iter().filter(|is_relevant| **is_relevant).count();
        let max_recomputations = module
            .functions
            .len()
            .max(relevant_function_count)
            .saturating_mul(relevant_function_count.saturating_add(1));
        Self {
            dependents: build_function_summary_dependents(module),
            pending,
            queued,
            relevant,
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
            if self.relevant[*dependent] && !self.queued[*dependent] {
                self.pending.push_back(*dependent);
                self.queued[*dependent] = true;
            }
        }
    }

    pub(super) fn recomputations(&self) -> usize {
        self.recomputations
    }
}

#[cfg(test)]
#[path = "summary_worklist_tests.rs"]
mod tests;
