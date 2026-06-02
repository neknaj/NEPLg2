extern crate alloc;

use alloc::borrow::Cow;
use alloc::collections::VecDeque;
use alloc::vec;
use alloc::vec::Vec;

use super::model::ResourceModule;
use super::summary_dependency::{
    build_function_summary_dependents, ResourceSummaryDependencyGraph,
};
use super::summary_worklist_order::initial_summary_order;

/// Resource summary 固定点計算で、変更された callee から caller を再投入する worklist。
///
/// 共有 `ResourceSummaryDependencyGraph` から作る経路では `dependents` を借用し、旧 API
/// から作る経路では同じ型で owned dependents を保持する。これにより、summary kind ごとに
/// 同じ逆辺リストを clone せず、既存の test helper や独立構築経路も維持できる。
pub(super) struct SummaryWorklist<'a> {
    dependents: Cow<'a, [Vec<usize>]>,
    pending: VecDeque<usize>,
    queued: Vec<bool>,
    relevant: Vec<bool>,
    recomputed: Vec<bool>,
    max_recomputations: usize,
    recomputations: usize,
}

impl<'a> SummaryWorklist<'a> {
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
            Cow::Owned(dependents),
            &initial_order,
        )
    }

    pub(super) fn new_filtered_with_dependency_graph(
        module: &ResourceModule,
        relevant: Vec<bool>,
        graph: &'a ResourceSummaryDependencyGraph,
    ) -> Self {
        Self::new_filtered_with_graph_and_initial_skips(
            module,
            relevant,
            vec![false; module.functions.len()],
            Cow::Borrowed(graph.dependents()),
            graph.initial_order(),
        )
    }

    pub(super) fn new_filtered_with_dependency_edges(
        module: &ResourceModule,
        relevant: Vec<bool>,
        dependents: &'a [Vec<usize>],
        initial_order: &[usize],
    ) -> Self {
        Self::new_filtered_with_graph_and_initial_skips(
            module,
            relevant,
            vec![false; module.functions.len()],
            Cow::Borrowed(dependents),
            initial_order,
        )
    }

    /// 共有済みの依存辺 view と初期 skip 情報から worklist を作る。
    ///
    /// Resource summary cache から安全に replay できた関数は `initially_skipped` で
    /// 最初の worklist 投入を省く。ただし、その後に依存先 summary が変わって
    /// `notify_changed` で再投入された場合は通常の再計算対象になる。caller が
    /// summary kind ごとの依存辺 view を渡せるため、summary が実際には読まない
    /// function value や facade を固定点探索と dependency closure hash から外せる。
    pub(super) fn new_filtered_with_dependency_edges_and_initial_skips(
        module: &ResourceModule,
        relevant: Vec<bool>,
        initially_skipped: Vec<bool>,
        dependents: &'a [Vec<usize>],
        initial_order: &[usize],
    ) -> Self {
        Self::new_filtered_with_graph_and_initial_skips(
            module,
            relevant,
            initially_skipped,
            Cow::Borrowed(dependents),
            initial_order,
        )
    }

    fn new_filtered_with_graph_and_initial_skips(
        module: &ResourceModule,
        relevant: Vec<bool>,
        initially_skipped: Vec<bool>,
        dependents: Cow<'a, [Vec<usize>]>,
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
            dependents,
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
