use alloc::vec::Vec;

use crate::types::{TypeId, TypeKind};

use super::env::BindingKind;
use super::type_expectation::TypeExpectation;
use super::{BlockChecker, StackEntry};

impl<'a> BlockChecker<'a> {
    fn unresolved_overloaded_pipe_target_input_type(
        &mut self,
        entry: &StackEntry,
        expected_result: Option<TypeId>,
    ) -> Option<(TypeId, bool)> {
        let crate::hir::HirExprKind::Var(name) = &entry.expr.kind else {
            return None;
        };
        if !entry.type_args.is_empty() {
            return None;
        }
        let bindings = self
            .env
            .lookup_all_callables(name)
            .into_iter()
            .cloned()
            .collect::<Vec<_>>();
        if bindings.len() <= 1 {
            return None;
        }
        let mut merged_input = None;
        let mut refined_input_by_expected = false;
        for binding in bindings {
            let BindingKind::Func {
                arity, captures, ..
            } = binding.kind
            else {
                continue;
            };
            if arity == 0 {
                continue;
            }
            let (inst_ty, _fresh_args, _mapping) = self.ctx.instantiate(binding.ty);
            let TypeKind::Function { params, result, .. } = self.ctx.get(inst_ty) else {
                continue;
            };
            let capture_len = captures.len();
            if params.len() < capture_len + arity {
                continue;
            }
            let input = params[capture_len];
            let input_before_expected = self.ctx.snapshot_type_var_bindings(input);
            if let Some(expected_result) = expected_result {
                let checkpoint = self.ctx.checkpoint();
                if self.ctx.unify(result, expected_result).is_ok() {
                    let input_after_expected = self.ctx.snapshot_type_var_bindings(input);
                    refined_input_by_expected |= input_after_expected != input_before_expected;
                    self.ctx.commit(checkpoint);
                } else {
                    self.ctx.rollback(checkpoint);
                }
            }
            merged_input = match merged_input {
                Some(current) => Some(self.merge_expected_argument_type(current, input)?),
                None => Some(input),
            };
        }
        merged_input.map(|input| (self.ctx.resolve_id(input), refined_input_by_expected))
    }

    pub(super) fn pipe_target_input_type(
        &mut self,
        entry: &StackEntry,
        expected_result: Option<TypeId>,
    ) -> Option<(TypeId, bool)> {
        if let Some(input) =
            self.unresolved_overloaded_pipe_target_input_type(entry, expected_result)
        {
            return Some(input);
        }
        let Some((params, result, _effect)) = self.function_signature_for_entry(entry) else {
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
        // A pipe target can be generic in the same way as a normal outer
        // consumer. The final chain expectation is useful for the left segment
        // only when solving the target result also binds variables that appear
        // in the target input. Unresolved overload placeholders have unrelated
        // fresh input/result variables, so they must fall back to the explicit
        // chain expectation instead of replacing it with an unconstrained input.
        let input_before_expected = self.ctx.snapshot_type_var_bindings(params[arg_idx]);
        let mut refined_input_by_expected = false;
        if let Some(expected_result) = expected_result {
            let checkpoint = self.ctx.checkpoint();
            if self.ctx.unify(result, expected_result).is_ok() {
                let input_after_expected = self.ctx.snapshot_type_var_bindings(params[arg_idx]);
                refined_input_by_expected = input_after_expected != input_before_expected;
                self.ctx.commit(checkpoint);
            } else {
                self.ctx.rollback(checkpoint);
            }
        }
        Some((
            self.ctx.resolve_id(params[arg_idx]),
            refined_input_by_expected,
        ))
    }

    fn expected_result_can_stand_for_pipe_input(
        &mut self,
        target_input: TypeId,
        expected_result: TypeId,
    ) -> bool {
        if matches!(
            self.ctx.get(self.ctx.resolve_id(target_input)),
            TypeKind::Var(var) if var.binding.is_none()
        ) {
            return false;
        }
        let checkpoint = self.ctx.checkpoint();
        let compatible = self.ctx.unify(target_input, expected_result).is_ok();
        self.ctx.rollback(checkpoint);
        compatible
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
        let target_input = self.pipe_target_input_type(target, fallback_expected);
        let expected_input = target_input.and_then(|(target_input, refined_by_expected)| {
            if refined_by_expected || self.is_concrete_type(target_input) {
                return Some(target_input);
            }
            fallback_expected
                .filter(|expected| {
                    self.expected_result_can_stand_for_pipe_input(target_input, *expected)
                })
                .map(|t| self.ctx.resolve_id(t))
        });
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
