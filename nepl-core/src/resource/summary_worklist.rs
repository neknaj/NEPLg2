extern crate alloc;

use alloc::collections::VecDeque;
use alloc::vec;
use alloc::vec::Vec;

use super::model::ResourceModule;
use super::summary_dependency::{
    build_function_summary_dependents, ResourceSummaryDependencyGraph,
};
use super::summary_worklist_order::initial_summary_order;

pub(super) struct SummaryWorklist {
    dependents: Vec<Vec<usize>>,
    pending: VecDeque<usize>,
    queued: Vec<bool>,
    relevant: Vec<bool>,
    recomputed: Vec<bool>,
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
        let dependents = build_function_summary_dependents(module);
        let initial_order = initial_summary_order(module);
        Self::new_filtered_with_graph_and_initial_skips(
            module,
            relevant,
            initially_skipped,
            &dependents,
            &initial_order,
        )
    }

    pub(super) fn new_filtered_with_dependency_graph(
        module: &ResourceModule,
        relevant: Vec<bool>,
        graph: &ResourceSummaryDependencyGraph,
    ) -> Self {
        Self::new_filtered_with_graph_and_initial_skips(
            module,
            relevant,
            vec![false; module.functions.len()],
            graph.dependents(),
            graph.initial_order(),
        )
    }

    /// 共有済みの依存グラフと初期 skip 情報から worklist を作る。
    ///
    /// Resource summary cache から安全に replay できた関数は `initially_skipped` で
    /// 最初の worklist 投入を省く。ただし、その後に依存先 summary が変わって
    /// `notify_changed` で再投入された場合は通常の再計算対象になる。これにより、
    /// cache hit 済み entry の重複 record を避けつつ、古い summary を必要なときに
    /// 更新できる。
    pub(super) fn new_filtered_with_dependency_graph_and_initial_skips(
        module: &ResourceModule,
        relevant: Vec<bool>,
        initially_skipped: Vec<bool>,
        graph: &ResourceSummaryDependencyGraph,
    ) -> Self {
        Self::new_filtered_with_graph_and_initial_skips(
            module,
            relevant,
            initially_skipped,
            graph.dependents(),
            graph.initial_order(),
        )
    }

    fn new_filtered_with_graph_and_initial_skips(
        module: &ResourceModule,
        relevant: Vec<bool>,
        initially_skipped: Vec<bool>,
        dependents: &[Vec<usize>],
        initial_order: &[usize],
    ) -> Self {
        debug_assert_eq!(relevant.len(), module.functions.len());
        debug_assert_eq!(initially_skipped.len(), module.functions.len());
        debug_assert_eq!(dependents.len(), module.functions.len());
        let mut pending = VecDeque::new();
        let mut queued = vec![false; module.functions.len()];
        for &index in initial_order {
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
            dependents: dependents.to_vec(),
            pending,
            queued,
            relevant,
            recomputed: vec![false; module.functions.len()],
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
        self.recomputed[index] = true;
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

    pub(super) fn unrecomputed_initial_skips(&self, initially_skipped: &[bool]) -> Vec<bool> {
        debug_assert_eq!(initially_skipped.len(), self.recomputed.len());
        initially_skipped
            .iter()
            .zip(self.recomputed.iter())
            .map(|(was_initially_skipped, was_recomputed)| {
                *was_initially_skipped && !*was_recomputed
            })
            .collect()
    }
}

#[cfg(test)]
#[path = "summary_worklist_tests.rs"]
mod tests;
