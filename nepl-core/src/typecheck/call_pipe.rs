use alloc::vec::Vec;

use crate::types::{TypeId, TypeKind};

use super::type_expectation::TypeExpectation;
use super::{BlockChecker, StackEntry};

impl<'a> BlockChecker<'a> {
    pub(super) fn pipe_target_input_type(&mut self, entry: &StackEntry) -> Option<TypeId> {
        let Some((params, _result, _effect)) = self.function_signature_for_entry(entry) else {
            return None;
        };
        let total_arity = params.len();
        let arity = self.user_visible_arity(&entry.expr, &params);
        if arity == 0 {
            return None;
        }
        let capture_len = total_arity.saturating_sub(arity);
        let arg_idx = capture_len;
        if arg_idx >= total_arity {
            return None;
        }
        Some(self.ctx.resolve_id(params[arg_idx]))
    }

    pub(super) fn reduce_pipe_pending_segment_with_target(
        &mut self,
        mut pending: Vec<StackEntry>,
        target: &StackEntry,
        fallback_expected: Option<TypeId>,
    ) -> Option<StackEntry> {
        if pending.is_empty() {
            return None;
        }
        let expected_input = self
            .pipe_target_input_type(target)
            .filter(|t| self.is_concrete_type(*t))
            .or(fallback_expected.map(|t| self.ctx.resolve_id(t)));
        let mut open_calls = Vec::new();
        for (i, entry) in pending.iter().enumerate() {
            let rty = self.ctx.resolve_id(entry.ty);
            if entry.auto_call && matches!(self.ctx.get(rty), TypeKind::Function { .. }) {
                open_calls.push(i);
            }
        }
        self.reduce_calls(
            &mut pending,
            &mut open_calls,
            expected_input.map(|t| TypeExpectation::outer_consumer_argument(t, 0)),
        );
        if pending.len() == 1 {
            pending.pop()
        } else {
            None
        }
    }

    pub(super) fn pipe_pending_base(
        &mut self,
        stack: &[StackEntry],
        open_calls: &[usize],
        default_base: usize,
    ) -> usize {
        if stack.len() <= default_base + 1 {
            return default_base;
        }
        let top_idx = stack.len() - 1;
        let Some(_) = open_calls
            .iter()
            .rev()
            .copied()
            .find(|&idx| idx >= default_base && idx < top_idx)
        else {
            return default_base;
        };
        if self.pipe_segment_reduces_to_single_value(stack, default_base) {
            return default_base;
        }
        for idx in open_calls.iter().copied() {
            if idx < default_base || idx >= top_idx {
                continue;
            }
            if self.pipe_segment_reduces_to_single_value(stack, idx) {
                return idx;
            }
        }
        top_idx
    }

    pub(super) fn pipe_segment_reduces_to_single_value(
        &mut self,
        stack: &[StackEntry],
        segment_base: usize,
    ) -> bool {
        if segment_base >= stack.len() {
            return false;
        }
        let checkpoint = self.ctx.checkpoint();
        let diagnostics_len = self.diagnostics.len();
        let trait_checks_len = self.pending_trait_bound_checks.len();
        let mut segment = stack[segment_base..].to_vec();
        let mut open_calls = segment
            .iter()
            .enumerate()
            .filter_map(|(idx, entry)| {
                let rty = self.ctx.resolve_id(entry.ty);
                if entry.auto_call && matches!(self.ctx.get(rty), TypeKind::Function { .. }) {
                    Some(idx)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        self.reduce_calls(&mut segment, &mut open_calls, None);
        let reduced = segment.len() == 1;
        self.pending_trait_bound_checks.truncate(trait_checks_len);
        self.diagnostics.truncate(diagnostics_len);
        self.ctx.rollback(checkpoint);
        reduced
    }
}
