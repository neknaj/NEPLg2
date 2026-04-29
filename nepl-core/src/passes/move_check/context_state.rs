use alloc::collections::{BTreeMap, BTreeSet};
use alloc::rc::Rc;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

use crate::diagnostic::Diagnostic;
use crate::diagnostic_ids::DiagnosticId;
use crate::hir::HirModule;
use crate::span::Span;
use crate::types::TypeId;

use super::state::{
    BorrowBinding, BorrowCount, BorrowKind, ExprBorrow, FieldMove, FieldMovePath,
    ResourceStateSnapshot, VarState,
};
use super::summary::FunctionRawAliasSummary;
use super::{variant_alias, MoveCheckContext};
impl<'m> MoveCheckContext<'m> {
    pub(super) fn new(module: &'m HirModule) -> Self {
        let function_defs = module
            .functions
            .iter()
            .map(|func| (func.name.clone(), func))
            .collect();
        Self {
            string_literals: module.string_literals.clone(),
            function_params: BTreeMap::new(),
            function_defs: Rc::new(function_defs),
            function_raw_alias_summaries: BTreeMap::new(),
            var_stacks: BTreeMap::new(),
            var_depth_stacks: BTreeMap::new(),
            borrow_stacks: BTreeMap::new(),
            field_move_stacks: BTreeMap::new(),
            raw_addr_alias_stacks: BTreeMap::new(),
            i32_const_stacks: BTreeMap::new(),
            enum_payload_raw_alias_stacks: BTreeMap::new(),
            aggregate_field_raw_alias_stacks: BTreeMap::new(),
            aggregate_field_function_alias_stacks: BTreeMap::new(),
            enum_payload_aggregate_field_raw_alias_stacks: BTreeMap::new(),
            enum_payload_aggregate_field_function_alias_stacks: BTreeMap::new(),
            enum_payload_function_alias_stacks: BTreeMap::new(),
            function_value_alias_stacks: BTreeMap::new(),
            raw_place_states: BTreeMap::new(),
            borrow_counts: BTreeMap::new(),
            use_counts: Vec::new(),
            diagnostics: Vec::new(),
            scopes: Vec::new(),
            raw_alias_specialization_stack: Vec::new(),
        }
    }

    pub(super) fn snapshot_resource_state(&self) -> ResourceStateSnapshot {
        ResourceStateSnapshot {
            var_stacks: self.var_stacks.clone(),
            var_depth_stacks: self.var_depth_stacks.clone(),
            borrow_stacks: self.borrow_stacks.clone(),
            field_move_stacks: self.field_move_stacks.clone(),
            raw_addr_alias_stacks: self.raw_addr_alias_stacks.clone(),
            i32_const_stacks: self.i32_const_stacks.clone(),
            enum_payload_raw_alias_stacks: self.enum_payload_raw_alias_stacks.clone(),
            aggregate_field_raw_alias_stacks: self.aggregate_field_raw_alias_stacks.clone(),
            aggregate_field_function_alias_stacks: self
                .aggregate_field_function_alias_stacks
                .clone(),
            enum_payload_aggregate_field_raw_alias_stacks: self
                .enum_payload_aggregate_field_raw_alias_stacks
                .clone(),
            enum_payload_aggregate_field_function_alias_stacks: self
                .enum_payload_aggregate_field_function_alias_stacks
                .clone(),
            enum_payload_function_alias_stacks: self.enum_payload_function_alias_stacks.clone(),
            function_value_alias_stacks: self.function_value_alias_stacks.clone(),
            raw_place_states: self.raw_place_states.clone(),
            borrow_counts: self.borrow_counts.clone(),
        }
    }

    pub(super) fn restore_resource_state(&mut self, snapshot: &ResourceStateSnapshot) {
        self.var_stacks = snapshot.var_stacks.clone();
        self.var_depth_stacks = snapshot.var_depth_stacks.clone();
        self.borrow_stacks = snapshot.borrow_stacks.clone();
        self.field_move_stacks = snapshot.field_move_stacks.clone();
        self.raw_addr_alias_stacks = snapshot.raw_addr_alias_stacks.clone();
        self.i32_const_stacks = snapshot.i32_const_stacks.clone();
        self.enum_payload_raw_alias_stacks = snapshot.enum_payload_raw_alias_stacks.clone();
        self.aggregate_field_raw_alias_stacks = snapshot.aggregate_field_raw_alias_stacks.clone();
        self.aggregate_field_function_alias_stacks =
            snapshot.aggregate_field_function_alias_stacks.clone();
        self.enum_payload_aggregate_field_raw_alias_stacks = snapshot
            .enum_payload_aggregate_field_raw_alias_stacks
            .clone();
        self.enum_payload_aggregate_field_function_alias_stacks = snapshot
            .enum_payload_aggregate_field_function_alias_stacks
            .clone();
        self.enum_payload_function_alias_stacks =
            snapshot.enum_payload_function_alias_stacks.clone();
        self.function_value_alias_stacks = snapshot.function_value_alias_stacks.clone();
        self.raw_place_states = snapshot.raw_place_states.clone();
        self.borrow_counts = snapshot.borrow_counts.clone();
    }

    pub(super) fn push_scope(&mut self) {
        self.scopes.push(BTreeSet::new());
    }

    pub(super) fn pop_scope(&mut self) {
        let vars_to_pop = self.scopes.pop().unwrap_or_default();
        for name in vars_to_pop {
            self.release_borrow_binding(&name);
            if let Some(stack) = self.var_stacks.get_mut(&name) {
                stack.pop();
                if stack.is_empty() {
                    self.var_stacks.remove(&name);
                }
            }
            if let Some(stack) = self.var_depth_stacks.get_mut(&name) {
                stack.pop();
                if stack.is_empty() {
                    self.var_depth_stacks.remove(&name);
                }
            }
            if let Some(stack) = self.borrow_stacks.get_mut(&name) {
                stack.pop();
                if stack.is_empty() {
                    self.borrow_stacks.remove(&name);
                }
            }
            if let Some(stack) = self.field_move_stacks.get_mut(&name) {
                stack.pop();
                if stack.is_empty() {
                    self.field_move_stacks.remove(&name);
                }
            }
            if let Some(stack) = self.raw_addr_alias_stacks.get_mut(&name) {
                stack.pop();
                if stack.is_empty() {
                    self.raw_addr_alias_stacks.remove(&name);
                }
            }
            if let Some(stack) = self.i32_const_stacks.get_mut(&name) {
                stack.pop();
                if stack.is_empty() {
                    self.i32_const_stacks.remove(&name);
                }
            }
            if let Some(stack) = self.enum_payload_raw_alias_stacks.get_mut(&name) {
                stack.pop();
                if stack.is_empty() {
                    self.enum_payload_raw_alias_stacks.remove(&name);
                }
            }
            if let Some(stack) = self.aggregate_field_raw_alias_stacks.get_mut(&name) {
                stack.pop();
                if stack.is_empty() {
                    self.aggregate_field_raw_alias_stacks.remove(&name);
                }
            }
            if let Some(stack) = self.aggregate_field_function_alias_stacks.get_mut(&name) {
                stack.pop();
                if stack.is_empty() {
                    self.aggregate_field_function_alias_stacks.remove(&name);
                }
            }
            if let Some(stack) = self
                .enum_payload_aggregate_field_raw_alias_stacks
                .get_mut(&name)
            {
                stack.pop();
                if stack.is_empty() {
                    self.enum_payload_aggregate_field_raw_alias_stacks
                        .remove(&name);
                }
            }
            if let Some(stack) = self
                .enum_payload_aggregate_field_function_alias_stacks
                .get_mut(&name)
            {
                stack.pop();
                if stack.is_empty() {
                    self.enum_payload_aggregate_field_function_alias_stacks
                        .remove(&name);
                }
            }
            if let Some(stack) = self.enum_payload_function_alias_stacks.get_mut(&name) {
                stack.pop();
                if stack.is_empty() {
                    self.enum_payload_function_alias_stacks.remove(&name);
                }
            }
            if let Some(stack) = self.function_value_alias_stacks.get_mut(&name) {
                stack.pop();
                if stack.is_empty() {
                    self.function_value_alias_stacks.remove(&name);
                }
            }
        }
    }

    pub(super) fn declare_var(&mut self, name: String) {
        self.declare_var_with_borrows(name, Vec::new());
    }

    pub(super) fn declare_var_with_borrows(&mut self, name: String, borrows: Vec<BorrowBinding>) {
        let depth = self.current_scope_depth();
        self.var_stacks
            .entry(name.clone())
            .or_default()
            .push(VarState::Valid);
        self.var_depth_stacks
            .entry(name.clone())
            .or_default()
            .push(depth);
        self.borrow_stacks
            .entry(name.clone())
            .or_default()
            .push(borrows);
        self.field_move_stacks
            .entry(name.clone())
            .or_default()
            .push(BTreeSet::new());
        self.raw_addr_alias_stacks
            .entry(name.clone())
            .or_default()
            .push(None);
        self.i32_const_stacks
            .entry(name.clone())
            .or_default()
            .push(None);
        self.enum_payload_raw_alias_stacks
            .entry(name.clone())
            .or_default()
            .push(BTreeMap::new());
        self.aggregate_field_raw_alias_stacks
            .entry(name.clone())
            .or_default()
            .push(BTreeMap::new());
        self.aggregate_field_function_alias_stacks
            .entry(name.clone())
            .or_default()
            .push(BTreeMap::new());
        self.enum_payload_aggregate_field_raw_alias_stacks
            .entry(name.clone())
            .or_default()
            .push(BTreeMap::new());
        self.enum_payload_aggregate_field_function_alias_stacks
            .entry(name.clone())
            .or_default()
            .push(BTreeMap::new());
        self.enum_payload_function_alias_stacks
            .entry(name.clone())
            .or_default()
            .push(BTreeMap::new());
        self.function_value_alias_stacks
            .entry(name.clone())
            .or_default()
            .push(BTreeSet::new());
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name);
        }
    }

    // For function params
    pub(super) fn declare_param(&mut self, name: String) {
        self.declare_var(name);
    }

    pub(super) fn get_state(&self, name: &str) -> Option<VarState> {
        self.var_stacks.get(name).and_then(|s| s.last().copied())
    }

    pub(super) fn current_scope_depth(&self) -> usize {
        self.scopes.len()
    }

    pub(super) fn scope_depth_of(&self, name: &str) -> Option<usize> {
        self.var_depth_stacks
            .get(name)
            .and_then(|stack| stack.last().copied())
    }

    pub(super) fn borrow_bindings(&self, name: &str) -> Vec<BorrowBinding> {
        self.borrow_stacks
            .get(name)
            .and_then(|stack| stack.last())
            .cloned()
            .unwrap_or_default()
    }

    pub(super) fn set_borrow_bindings(&mut self, name: &str, bindings: Vec<BorrowBinding>) {
        self.release_borrow_binding(name);
        if let Some(stack) = self.borrow_stacks.get_mut(name) {
            if let Some(slot) = stack.last_mut() {
                *slot = bindings;
            }
        }
    }

    pub(super) fn set_field_moves(&mut self, name: &str, moves: BTreeSet<FieldMove>) {
        if let Some(stack) = self.field_move_stacks.get_mut(name) {
            if let Some(slot) = stack.last_mut() {
                *slot = moves;
            }
        }
    }

    pub(super) fn clear_field_moves(&mut self, name: &str) {
        if let Some(stack) = self.field_move_stacks.get_mut(name) {
            if let Some(slot) = stack.last_mut() {
                slot.clear();
            }
        }
    }

    pub(super) fn raw_addr_alias(&self, name: &str) -> Option<&str> {
        self.raw_addr_alias_stacks
            .get(name)
            .and_then(|stack| stack.last())
            .and_then(|slot| slot.as_deref())
    }

    pub(super) fn set_raw_addr_alias(&mut self, name: &str, alias: Option<String>) {
        if let Some(stack) = self.raw_addr_alias_stacks.get_mut(name) {
            if let Some(slot) = stack.last_mut() {
                *slot = alias;
            }
        }
    }

    pub(super) fn i32_const_alias(&self, name: &str) -> Option<i64> {
        self.i32_const_stacks
            .get(name)
            .and_then(|stack| stack.last())
            .and_then(|slot| *slot)
    }

    pub(super) fn set_i32_const_alias(&mut self, name: &str, value: Option<i64>) {
        if let Some(stack) = self.i32_const_stacks.get_mut(name) {
            if let Some(slot) = stack.last_mut() {
                *slot = value;
            }
        }
    }

    pub(super) fn function_value_aliases(&self, name: &str) -> BTreeSet<String> {
        self.function_value_alias_stacks
            .get(name)
            .and_then(|stack| stack.last())
            .cloned()
            .unwrap_or_default()
    }

    pub(super) fn set_function_value_aliases(&mut self, name: &str, aliases: BTreeSet<String>) {
        if let Some(stack) = self.function_value_alias_stacks.get_mut(name) {
            if let Some(slot) = stack.last_mut() {
                *slot = aliases;
            }
        }
    }

    pub(super) fn enum_payload_raw_alias(&self, name: &str, variant: &str) -> Option<&str> {
        let aliases = self
            .enum_payload_raw_alias_stacks
            .get(name)
            .and_then(|stack| stack.last())?;
        variant_alias(aliases, variant).map(String::as_str)
    }

    pub(super) fn set_enum_payload_raw_aliases(
        &mut self,
        name: &str,
        aliases: BTreeMap<String, String>,
    ) {
        if let Some(stack) = self.enum_payload_raw_alias_stacks.get_mut(name) {
            if let Some(slot) = stack.last_mut() {
                *slot = aliases;
            }
        }
    }

    pub(super) fn aggregate_field_raw_alias(&self, name: &str, offset: usize) -> Option<&str> {
        self.aggregate_field_raw_alias_stacks
            .get(name)
            .and_then(|stack| stack.last())
            .and_then(|aliases| aliases.get(&offset))
            .map(String::as_str)
    }

    pub(super) fn aggregate_field_raw_aliases(&self, name: &str) -> BTreeMap<usize, String> {
        self.aggregate_field_raw_alias_stacks
            .get(name)
            .and_then(|stack| stack.last())
            .cloned()
            .unwrap_or_default()
    }

    pub(super) fn set_aggregate_field_raw_aliases(
        &mut self,
        name: &str,
        aliases: BTreeMap<usize, String>,
    ) {
        if let Some(stack) = self.aggregate_field_raw_alias_stacks.get_mut(name) {
            if let Some(slot) = stack.last_mut() {
                *slot = aliases;
            }
        }
    }

    pub(super) fn aggregate_field_function_aliases(
        &self,
        name: &str,
    ) -> BTreeMap<usize, BTreeSet<String>> {
        self.aggregate_field_function_alias_stacks
            .get(name)
            .and_then(|stack| stack.last())
            .cloned()
            .unwrap_or_default()
    }

    pub(super) fn set_aggregate_field_function_aliases(
        &mut self,
        name: &str,
        aliases: BTreeMap<usize, BTreeSet<String>>,
    ) {
        if let Some(stack) = self.aggregate_field_function_alias_stacks.get_mut(name) {
            if let Some(slot) = stack.last_mut() {
                *slot = aliases;
            }
        }
    }

    pub(super) fn enum_payload_aggregate_field_raw_aliases(
        &self,
        name: &str,
        variant: &str,
    ) -> BTreeMap<usize, String> {
        let Some(aliases) = self
            .enum_payload_aggregate_field_raw_alias_stacks
            .get(name)
            .and_then(|stack| stack.last())
        else {
            return BTreeMap::new();
        };
        variant_alias(aliases, variant).cloned().unwrap_or_default()
    }

    pub(super) fn set_enum_payload_aggregate_field_raw_aliases(
        &mut self,
        name: &str,
        aliases: BTreeMap<String, BTreeMap<usize, String>>,
    ) {
        if let Some(stack) = self
            .enum_payload_aggregate_field_raw_alias_stacks
            .get_mut(name)
        {
            if let Some(slot) = stack.last_mut() {
                *slot = aliases;
            }
        }
    }

    pub(super) fn enum_payload_aggregate_field_function_aliases(
        &self,
        name: &str,
        variant: &str,
    ) -> BTreeMap<usize, BTreeSet<String>> {
        let Some(aliases) = self
            .enum_payload_aggregate_field_function_alias_stacks
            .get(name)
            .and_then(|stack| stack.last())
        else {
            return BTreeMap::new();
        };
        variant_alias(aliases, variant).cloned().unwrap_or_default()
    }

    pub(super) fn set_enum_payload_aggregate_field_function_aliases(
        &mut self,
        name: &str,
        aliases: BTreeMap<String, BTreeMap<usize, BTreeSet<String>>>,
    ) {
        if let Some(stack) = self
            .enum_payload_aggregate_field_function_alias_stacks
            .get_mut(name)
        {
            if let Some(slot) = stack.last_mut() {
                *slot = aliases;
            }
        }
    }

    pub(super) fn enum_payload_function_aliases_for_variant(
        &self,
        name: &str,
        variant: &str,
    ) -> BTreeSet<String> {
        let aliases = self
            .enum_payload_function_alias_stacks
            .get(name)
            .and_then(|stack| stack.last());
        aliases
            .and_then(|aliases| variant_alias(aliases, variant))
            .cloned()
            .unwrap_or_default()
    }

    pub(super) fn set_enum_payload_function_aliases(
        &mut self,
        name: &str,
        aliases: BTreeMap<String, BTreeSet<String>>,
    ) {
        if let Some(stack) = self.enum_payload_function_alias_stacks.get_mut(name) {
            if let Some(slot) = stack.last_mut() {
                *slot = aliases;
            }
        }
    }

    pub(super) fn string_literal(&self, id: u32) -> Option<&str> {
        self.string_literals.get(id as usize).map(String::as_str)
    }

    pub(super) fn has_field_moves(&self, name: &str) -> bool {
        self.field_move_stacks
            .get(name)
            .and_then(|stack| stack.last())
            .map(|moves| !moves.is_empty())
            .unwrap_or(false)
    }

    pub(super) fn mark_field_moved(&mut self, path: &FieldMovePath) {
        if let Some(stack) = self.field_move_stacks.get_mut(path.owner.as_str()) {
            if let Some(slot) = stack.last_mut() {
                slot.insert(FieldMove {
                    field_index: path.field_index,
                    offset: path.offset,
                    ty: path.field_ty,
                });
            }
        }
    }

    pub(super) fn field_is_moved(&self, path: &FieldMovePath) -> bool {
        self.field_move_stacks
            .get(path.owner.as_str())
            .and_then(|stack| stack.last())
            .map(|moves| {
                moves
                    .iter()
                    .any(|field_move| field_moves_overlap(field_move, path))
            })
            .unwrap_or(false)
    }

    pub(super) fn set_state(&mut self, name: &str, state: VarState) {
        if let Some(stack) = self.var_stacks.get_mut(name) {
            if let Some(last) = stack.last_mut() {
                if *last == state {
                    return;
                }
                *last = state;
            }
        }
    }

    pub(super) fn push_use_counts(&mut self, counts: BTreeMap<String, usize>) {
        self.use_counts.push(counts);
    }

    pub(super) fn pop_use_counts(&mut self) {
        self.use_counts.pop();
    }

    pub(super) fn remaining_uses(&self, name: &str) -> usize {
        self.use_counts
            .iter()
            .filter_map(|counts| counts.get(name))
            .sum()
    }

    pub(super) fn note_var_use(&mut self, name: &str) {
        for counts in &mut self.use_counts {
            if let Some(count) = counts.get_mut(name) {
                *count = count.saturating_sub(1);
            }
        }
        if self.remaining_uses(name) == 0 {
            self.release_borrow_binding(name);
        }
    }

    pub(super) fn increment_borrow_count(&mut self, name: &str, kind: BorrowKind) {
        let count = self.borrow_counts.entry(name.to_string()).or_default();
        match kind {
            BorrowKind::Shared => count.shared += 1,
            BorrowKind::Unique => count.unique += 1,
        }
    }

    pub(super) fn release_borrow_binding(&mut self, name: &str) {
        let bindings = self
            .borrow_stacks
            .get_mut(name)
            .and_then(|stack| stack.last_mut())
            .map(core::mem::take)
            .unwrap_or_default();
        for binding in bindings {
            self.release_source_borrow(binding.source.as_str(), binding.kind);
        }
    }

    pub(super) fn release_borrow_bindings(&mut self, bindings: &[BorrowBinding]) {
        for binding in bindings {
            self.release_source_borrow(binding.source.as_str(), binding.kind);
        }
    }

    pub(super) fn release_source_borrow(&mut self, source: &str, kind: BorrowKind) {
        let Some(count) = self.borrow_counts.get_mut(source) else {
            return;
        };
        match kind {
            BorrowKind::Shared => count.shared = count.shared.saturating_sub(1),
            BorrowKind::Unique => count.unique = count.unique.saturating_sub(1),
        }
        let next = if count.unique > 0 {
            Some(VarState::BorrowedUnique)
        } else if count.shared > 0 {
            Some(VarState::BorrowedShared)
        } else {
            None
        };
        if next.is_none() {
            self.borrow_counts.remove(source);
        }
        match (self.get_state(source), next) {
            (Some(VarState::BorrowedShared | VarState::BorrowedUnique), Some(state)) => {
                self.set_state(source, state);
            }
            (Some(VarState::BorrowedShared | VarState::BorrowedUnique), None) => {
                self.set_state(source, VarState::Valid);
            }
            _ => {}
        }
    }

    pub(super) fn check_borrow_escape(&mut self, source: &str, span: Span, escape_depth: usize) {
        let Some(source_depth) = self.scope_depth_of(source) else {
            return;
        };
        if source_depth <= escape_depth {
            return;
        }
        self.diagnostics.push(
            Diagnostic::error(
                alloc::format!(
                    "borrowed local value does not live long enough: `{}`",
                    source
                ),
                span,
            )
            .with_id(DiagnosticId::TypeBorrowEscapesScope),
        );
    }

    pub(super) fn check_binding_escape(
        &mut self,
        binding: &BorrowBinding,
        span: Span,
        escape_depth: usize,
    ) {
        self.check_borrow_escape(binding.source.as_str(), span, escape_depth);
    }

    pub(super) fn check_expr_borrows_escape(
        &mut self,
        borrows: &[ExprBorrow],
        span: Span,
        escape_depth: usize,
    ) {
        for borrow in borrows {
            self.check_binding_escape(&borrow.binding, span, escape_depth);
        }
    }

    pub(super) fn check_var_escape(&mut self, name: &str, span: Span, escape_depth: usize) {
        for binding in self.borrow_bindings(name) {
            self.check_binding_escape(&binding, span, escape_depth);
        }
    }

    pub(super) fn retain_expr_borrows(&mut self, borrows: Vec<ExprBorrow>) -> Vec<BorrowBinding> {
        let mut bindings = Vec::with_capacity(borrows.len());
        for borrow in borrows {
            self.retain_borrow_binding(&borrow.binding);
            bindings.push(borrow.binding);
        }
        bindings
    }

    pub(super) fn retain_borrow_binding(&mut self, binding: &BorrowBinding) {
        match (self.get_state(binding.source.as_str()), binding.kind) {
            (Some(VarState::Valid), kind) => {
                self.increment_borrow_count(binding.source.as_str(), kind);
                let next = match kind {
                    BorrowKind::Shared => VarState::BorrowedShared,
                    BorrowKind::Unique => VarState::BorrowedUnique,
                };
                self.set_state(binding.source.as_str(), next);
            }
            (Some(VarState::BorrowedShared), BorrowKind::Shared) => {
                self.increment_borrow_count(binding.source.as_str(), binding.kind);
            }
            _ => {}
        }
    }

    pub(super) fn check_use(&mut self, name: &str, span: Span, is_copy: bool) {
        // NOTE: reserved words should not be treated as variables
        if matches!(name, "if" | "while" | "let" | "set") {
            return;
        }

        match self.get_state(name) {
            Some(VarState::Valid) => {
                if !is_copy {
                    if self.has_field_moves(name) {
                        self.diagnostics.push(
                            Diagnostic::error(
                                alloc::format!("use of partially moved value: `{}`", name),
                                span,
                            )
                            .with_id(DiagnosticId::TypeUseMovedValue),
                        );
                    } else {
                        self.set_state(name, VarState::Moved);
                    }
                }
            }
            Some(VarState::BorrowedShared) => {
                if !is_copy {
                    self.diagnostics.push(
                        Diagnostic::error(
                            alloc::format!("cannot move out of shared borrowed value: `{}`", name),
                            span,
                        )
                        .with_id(DiagnosticId::TypeMoveFromSharedBorrowedValue),
                    );
                }
            }
            Some(VarState::BorrowedUnique) => {
                self.diagnostics.push(
                    Diagnostic::error(
                        alloc::format!("use of uniquely borrowed value: `{}`", name),
                        span,
                    )
                    .with_id(DiagnosticId::TypeUseUniquelyBorrowedValue),
                );
            }
            Some(VarState::Moved) => {
                self.diagnostics.push(
                    Diagnostic::error(alloc::format!("use of moved value: `{}`", name), span)
                        .with_id(DiagnosticId::TypeUseMovedValue),
                );
            }
            Some(VarState::PossiblyMoved) => {
                self.diagnostics.push(
                    Diagnostic::error(
                        alloc::format!("use of potentially moved value: `{}`", name),
                        span,
                    )
                    .with_id(DiagnosticId::TypeUsePossiblyMovedValue),
                );
            }
            None => {}
        }
    }

    pub(super) fn with_function_params(
        module: &'m HirModule,
        function_params: BTreeMap<String, Vec<TypeId>>,
        function_raw_alias_summaries: BTreeMap<String, FunctionRawAliasSummary>,
    ) -> Self {
        let mut ctx = Self::new(module);
        ctx.function_params = function_params;
        ctx.function_raw_alias_summaries = function_raw_alias_summaries;
        ctx
    }

    pub(super) fn check_assign(&mut self, name: &str, span: Span) {
        match self.get_state(name) {
            Some(VarState::BorrowedShared) => {
                self.diagnostics.push(
                    Diagnostic::error(
                        alloc::format!("cannot assign to shared borrowed value: `{}`", name),
                        span,
                    )
                    .with_id(DiagnosticId::TypeAssignSharedBorrowedValue),
                );
            }
            Some(VarState::BorrowedUnique) => {
                self.diagnostics.push(
                    Diagnostic::error(
                        alloc::format!("cannot assign to uniquely borrowed value: `{}`", name),
                        span,
                    )
                    .with_id(DiagnosticId::TypeAssignUniquelyBorrowedValue),
                );
            }
            _ => {
                self.set_state(name, VarState::Valid);
                self.clear_field_moves(name);
            }
        }
    }

    pub(super) fn check_drop(&mut self, name: &str, span: Span) {
        match self.get_state(name) {
            Some(VarState::Valid) => {
                if self.has_field_moves(name) {
                    self.diagnostics.push(
                        Diagnostic::error(
                            alloc::format!("drop of partially moved value: `{}`", name),
                            span,
                        )
                        .with_id(DiagnosticId::TypeDropMovedValue),
                    );
                } else {
                    self.set_state(name, VarState::Moved);
                }
            }
            Some(VarState::BorrowedShared) => {
                self.diagnostics.push(
                    Diagnostic::error(
                        alloc::format!("cannot drop shared borrowed value: `{}`", name),
                        span,
                    )
                    .with_id(DiagnosticId::TypeDropSharedBorrowedValue),
                );
            }
            Some(VarState::BorrowedUnique) => {
                self.diagnostics.push(
                    Diagnostic::error(
                        alloc::format!("cannot drop uniquely borrowed value: `{}`", name),
                        span,
                    )
                    .with_id(DiagnosticId::TypeDropUniquelyBorrowedValue),
                );
            }
            Some(VarState::Moved) => {
                self.diagnostics.push(
                    Diagnostic::error(alloc::format!("drop of moved value: `{}`", name), span)
                        .with_id(DiagnosticId::TypeDropMovedValue),
                );
            }
            Some(VarState::PossiblyMoved) => {
                self.diagnostics.push(
                    Diagnostic::error(
                        alloc::format!("drop of potentially moved value: `{}`", name),
                        span,
                    )
                    .with_id(DiagnosticId::TypeDropPossiblyMovedValue),
                );
            }
            None => {}
        }
    }

    pub(super) fn check_temporary_borrow(&mut self, name: &str, span: Span, kind: BorrowKind) {
        match self.get_state(name) {
            Some(VarState::Valid) => {
                if self.has_field_moves(name) {
                    self.diagnostics.push(
                        Diagnostic::error(
                            alloc::format!("borrow of partially moved value: `{}`", name),
                            span,
                        )
                        .with_id(DiagnosticId::TypeBorrowMovedValue),
                    );
                }
            }
            Some(VarState::BorrowedShared) => {
                if matches!(kind, BorrowKind::Unique) {
                    self.diagnostics.push(
                        Diagnostic::error(
                            alloc::format!(
                                "cannot uniquely borrow shared borrowed value: `{}`",
                                name
                            ),
                            span,
                        )
                        .with_id(DiagnosticId::TypeUniqueBorrowSharedBorrowedValue),
                    );
                }
            }
            Some(VarState::BorrowedUnique) => {
                self.diagnostics.push(
                    Diagnostic::error(
                        alloc::format!("cannot borrow uniquely borrowed value: `{}`", name),
                        span,
                    )
                    .with_id(DiagnosticId::TypeBorrowUniquelyBorrowedValue),
                );
            }
            Some(VarState::Moved) => {
                self.diagnostics.push(
                    Diagnostic::error(alloc::format!("borrow of moved value: `{}`", name), span)
                        .with_id(DiagnosticId::TypeBorrowMovedValue),
                );
            }
            Some(VarState::PossiblyMoved) => {
                self.diagnostics.push(
                    Diagnostic::error(
                        alloc::format!("borrow of potentially moved value: `{}`", name),
                        span,
                    )
                    .with_id(DiagnosticId::TypeBorrowPossiblyMovedValue),
                );
            }
            None => {}
        }
    }

    pub(super) fn check_field_move(&mut self, path: &FieldMovePath, span: Span) {
        match self.get_state(path.owner.as_str()) {
            Some(VarState::Valid) => {
                if self.field_is_moved(path) {
                    self.diagnostics.push(
                        Diagnostic::error(
                            alloc::format!(
                                "use of moved field at offset {} in `{}`",
                                path.offset,
                                path.owner
                            ),
                            span,
                        )
                        .with_id(DiagnosticId::TypeUseMovedValue),
                    );
                } else {
                    self.mark_field_moved(path);
                }
            }
            Some(VarState::BorrowedShared) => {
                self.diagnostics.push(
                    Diagnostic::error(
                        alloc::format!(
                            "cannot move out of shared borrowed value: `{}`",
                            path.owner
                        ),
                        span,
                    )
                    .with_id(DiagnosticId::TypeMoveFromSharedBorrowedValue),
                );
            }
            Some(VarState::BorrowedUnique) => {
                self.diagnostics.push(
                    Diagnostic::error(
                        alloc::format!("use of uniquely borrowed value: `{}`", path.owner),
                        span,
                    )
                    .with_id(DiagnosticId::TypeUseUniquelyBorrowedValue),
                );
            }
            Some(VarState::Moved) => {
                self.diagnostics.push(
                    Diagnostic::error(alloc::format!("use of moved value: `{}`", path.owner), span)
                        .with_id(DiagnosticId::TypeUseMovedValue),
                );
            }
            Some(VarState::PossiblyMoved) => {
                self.diagnostics.push(
                    Diagnostic::error(
                        alloc::format!("use of potentially moved value: `{}`", path.owner),
                        span,
                    )
                    .with_id(DiagnosticId::TypeUsePossiblyMovedValue),
                );
            }
            None => {}
        }
    }

    pub(super) fn check_field_temporary_borrow(
        &mut self,
        path: &FieldMovePath,
        span: Span,
        kind: BorrowKind,
    ) {
        match self.get_state(path.owner.as_str()) {
            Some(VarState::Valid) => {
                if self.field_is_moved(path) {
                    self.diagnostics.push(
                        Diagnostic::error(
                            alloc::format!(
                                "borrow of moved field at offset {} in `{}`",
                                path.offset,
                                path.owner
                            ),
                            span,
                        )
                        .with_id(DiagnosticId::TypeBorrowMovedValue),
                    );
                }
            }
            Some(VarState::BorrowedShared) => {
                if matches!(kind, BorrowKind::Unique) {
                    self.diagnostics.push(
                        Diagnostic::error(
                            alloc::format!(
                                "cannot uniquely borrow shared borrowed value: `{}`",
                                path.owner
                            ),
                            span,
                        )
                        .with_id(DiagnosticId::TypeUniqueBorrowSharedBorrowedValue),
                    );
                }
            }
            Some(VarState::BorrowedUnique) => {
                self.diagnostics.push(
                    Diagnostic::error(
                        alloc::format!("cannot borrow uniquely borrowed value: `{}`", path.owner),
                        span,
                    )
                    .with_id(DiagnosticId::TypeBorrowUniquelyBorrowedValue),
                );
            }
            Some(VarState::Moved) => {
                self.diagnostics.push(
                    Diagnostic::error(
                        alloc::format!("borrow of moved value: `{}`", path.owner),
                        span,
                    )
                    .with_id(DiagnosticId::TypeBorrowMovedValue),
                );
            }
            Some(VarState::PossiblyMoved) => {
                self.diagnostics.push(
                    Diagnostic::error(
                        alloc::format!("borrow of potentially moved value: `{}`", path.owner),
                        span,
                    )
                    .with_id(DiagnosticId::TypeBorrowPossiblyMovedValue),
                );
            }
            None => {}
        }
    }

    pub(super) fn merge_state_pair(a: VarState, b: VarState) -> VarState {
        use VarState::*;
        match (a, b) {
            (Valid, Valid) => Valid,
            (BorrowedShared, BorrowedShared) => BorrowedShared,
            (BorrowedUnique, BorrowedUnique) => BorrowedUnique,
            (Moved, Moved) => Moved,
            (PossiblyMoved, _) | (_, PossiblyMoved) => PossiblyMoved,
            (Moved, _) | (_, Moved) => PossiblyMoved,
            (BorrowedUnique, BorrowedShared) | (BorrowedShared, BorrowedUnique) => BorrowedShared,
            (BorrowedShared, Valid) | (Valid, BorrowedShared) => BorrowedShared,
            (BorrowedUnique, Valid) | (Valid, BorrowedUnique) => BorrowedShared,
        }
    }

    pub(super) fn merge_states(states: &[VarState]) -> VarState {
        let mut it = states.iter().copied();
        let first = it.next().unwrap_or(VarState::Valid);
        it.fold(first, Self::merge_state_pair)
    }

    pub(super) fn release_dead_borrows(&mut self) {
        let names: Vec<String> = self.borrow_stacks.keys().cloned().collect();
        for name in names {
            if self.remaining_uses(name.as_str()) == 0 {
                self.release_borrow_binding(name.as_str());
            }
        }
    }

    pub(super) fn rebuild_borrow_counts_from_bindings(&mut self) {
        let mut counts: BTreeMap<String, BorrowCount> = BTreeMap::new();
        for stack in self.borrow_stacks.values() {
            for bindings in stack {
                for binding in bindings {
                    let count = counts.entry(binding.source.clone()).or_default();
                    match binding.kind {
                        BorrowKind::Shared => count.shared += 1,
                        BorrowKind::Unique => count.unique += 1,
                    }
                }
            }
        }
        self.borrow_counts = counts;
        let borrowed_sources: Vec<(String, BorrowCount)> = self
            .borrow_counts
            .iter()
            .map(|(name, count)| (name.clone(), *count))
            .collect();
        for (source, count) in borrowed_sources {
            match self.get_state(source.as_str()) {
                Some(VarState::Valid | VarState::BorrowedShared | VarState::BorrowedUnique) => {
                    let state = if count.unique > 0 {
                        VarState::BorrowedUnique
                    } else {
                        VarState::BorrowedShared
                    };
                    self.set_state(source.as_str(), state);
                }
                _ => {}
            }
        }
    }
}

fn field_moves_overlap(field_move: &FieldMove, path: &FieldMovePath) -> bool {
    if let (Some(left), Some(right)) = (field_move.field_index, path.field_index) {
        return left == right;
    }
    if field_move.ty != path.field_ty || field_move.offset != path.offset {
        return false;
    }
    true
}
