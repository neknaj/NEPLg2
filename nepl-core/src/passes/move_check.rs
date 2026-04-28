extern crate alloc;

mod branch_merge;
mod provenance;
mod raw_memory;
mod raw_place;
mod state;
mod summary;
mod summary_build;

use alloc::collections::{BTreeMap, BTreeSet};
use alloc::rc::Rc;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

use crate::diagnostic::Diagnostic;
use crate::diagnostic_ids::DiagnosticId;
use crate::hir::{
    FuncRef, HirBlock, HirExpr, HirExprKind, HirFunction, HirMatchArm, HirMatchPattern, HirModule,
};
use crate::layout::{aggregate_fields_with_offsets, storage_size_bytes};
use crate::span::Span;
use crate::types::{TypeId, TypeKind};

use branch_merge::{
    changed_state_names, merge_continuing_branch_states, snapshot_top_state, BranchStateSnapshot,
};
use provenance::{
    aggregate_field_raw_alias_at, field_move_path_from_addr, field_reference_path_from_addr,
    func_ref_name, i32_const_from_value, is_field_get_name, is_region_ptr_at_name,
    raw_addr_alias_from_value, raw_bulk_copy_size_arg_bytes, raw_byte_write_size_arg_bytes,
    raw_dealloc_place_key, raw_dealloc_size_arg_bytes, raw_memory_place_key,
    raw_memory_place_key_from_region_token, raw_store_write_size_bytes,
};
use raw_memory::{raw_memory_call_kind, RawMemoryCallKind};
use raw_place::{
    combine_raw_memory_offsets, format_raw_memory_place_key_parts, parse_raw_memory_place_key,
    raw_place_key_has_unknown_offset, raw_place_ranges_overlap, RawPlaceInfo, RawPlaceState,
};
use state::{
    BorrowBinding, BorrowCount, BorrowKind, ExprBorrow, FieldMove, FieldMovePath,
    ResourceStateSnapshot, VarState,
};
use summary::{
    extend_unique_raw_memory_effects, FunctionRawAliasSummary, RawMemoryEffectSummary,
    ValueAliasSummary,
};
use summary_build::{
    block_raw_alias_summary, build_function_raw_alias_summaries, expression_raw_alias_summary,
};

struct MoveCheckContext<'m> {
    /// String literals referenced by HIR field selector expressions.
    string_literals: Vec<String>,
    /// Function parameter types after monomorphization.
    function_params: BTreeMap<String, Vec<TypeId>>,
    /// Function definitions used to specialize raw alias summaries at call sites.
    function_defs: Rc<BTreeMap<String, &'m HirFunction>>,
    /// Function return provenance summaries after monomorphization.
    function_raw_alias_summaries: BTreeMap<String, FunctionRawAliasSummary>,
    /// State of all variables currently in scope.
    /// Stack of variable states (for shadowing support).
    var_stacks: BTreeMap<String, Vec<VarState>>,
    /// Scope depth for each variable binding, aligned with `var_stacks`.
    var_depth_stacks: BTreeMap<String, Vec<usize>>,
    /// Borrow sources held by each binding, aligned with `var_stacks`.
    borrow_stacks: BTreeMap<String, Vec<Vec<BorrowBinding>>>,
    /// Non-Copy aggregate fields moved out of each binding, aligned with `var_stacks`.
    field_move_stacks: BTreeMap<String, Vec<BTreeSet<FieldMove>>>,
    /// Canonical raw-address aliases for i32 bindings, aligned with `var_stacks`.
    raw_addr_alias_stacks: BTreeMap<String, Vec<Option<String>>>,
    /// Known i32 constants for bindings, aligned with `var_stacks`.
    i32_const_stacks: BTreeMap<String, Vec<Option<i64>>>,
    /// Raw aliases held by enum payloads, aligned with `var_stacks`.
    enum_payload_raw_alias_stacks: BTreeMap<String, Vec<BTreeMap<String, String>>>,
    /// Raw aliases held by aggregate fields, aligned with `var_stacks`.
    aggregate_field_raw_alias_stacks: BTreeMap<String, Vec<BTreeMap<usize, String>>>,
    /// Function aliases held by aggregate fields, aligned with `var_stacks`.
    aggregate_field_function_alias_stacks: BTreeMap<String, Vec<BTreeMap<usize, BTreeSet<String>>>>,
    /// Aggregate-field raw aliases held by enum payloads, aligned with `var_stacks`.
    enum_payload_aggregate_field_raw_alias_stacks:
        BTreeMap<String, Vec<BTreeMap<String, BTreeMap<usize, String>>>>,
    /// Aggregate-field function aliases held by enum payloads, aligned with `var_stacks`.
    enum_payload_aggregate_field_function_alias_stacks:
        BTreeMap<String, Vec<BTreeMap<String, BTreeMap<usize, BTreeSet<String>>>>>,
    /// Function-value aliases held by enum payloads, aligned with `var_stacks`.
    enum_payload_function_alias_stacks: BTreeMap<String, Vec<BTreeMap<String, BTreeSet<String>>>>,
    /// Known function-value aliases for function-typed bindings, aligned with `var_stacks`.
    function_value_alias_stacks: BTreeMap<String, Vec<BTreeSet<String>>>,
    /// Ownership state for trackable raw memory places that carry non-Copy values.
    raw_place_states: BTreeMap<String, RawPlaceInfo>,
    /// Active borrow counts per source variable.
    borrow_counts: BTreeMap<String, BorrowCount>,
    /// Remaining variable uses in active blocks for last-use borrow release.
    use_counts: Vec<BTreeMap<String, usize>>,
    /// Diagnostics (errors) collected.
    diagnostics: Vec<Diagnostic>,
    /// Scopes for variable cleanup
    scopes: Vec<BTreeSet<String>>,
    /// Active raw alias specializations, used to stop recursive call-site expansion.
    raw_alias_specialization_stack: Vec<String>,
}

impl<'m> MoveCheckContext<'m> {
    fn new(module: &'m HirModule) -> Self {
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

    fn snapshot_resource_state(&self) -> ResourceStateSnapshot {
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

    fn restore_resource_state(&mut self, snapshot: &ResourceStateSnapshot) {
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

    fn push_scope(&mut self) {
        self.scopes.push(BTreeSet::new());
    }

    fn pop_scope(&mut self) {
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

    fn declare_var(&mut self, name: String) {
        self.declare_var_with_borrows(name, Vec::new());
    }

    fn declare_var_with_borrows(&mut self, name: String, borrows: Vec<BorrowBinding>) {
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
    fn declare_param(&mut self, name: String) {
        self.declare_var(name);
    }

    fn get_state(&self, name: &str) -> Option<VarState> {
        self.var_stacks.get(name).and_then(|s| s.last().copied())
    }

    fn current_scope_depth(&self) -> usize {
        self.scopes.len()
    }

    fn scope_depth_of(&self, name: &str) -> Option<usize> {
        self.var_depth_stacks
            .get(name)
            .and_then(|stack| stack.last().copied())
    }

    fn borrow_bindings(&self, name: &str) -> Vec<BorrowBinding> {
        self.borrow_stacks
            .get(name)
            .and_then(|stack| stack.last())
            .cloned()
            .unwrap_or_default()
    }

    fn set_borrow_bindings(&mut self, name: &str, bindings: Vec<BorrowBinding>) {
        self.release_borrow_binding(name);
        if let Some(stack) = self.borrow_stacks.get_mut(name) {
            if let Some(slot) = stack.last_mut() {
                *slot = bindings;
            }
        }
    }

    fn set_field_moves(&mut self, name: &str, moves: BTreeSet<FieldMove>) {
        if let Some(stack) = self.field_move_stacks.get_mut(name) {
            if let Some(slot) = stack.last_mut() {
                *slot = moves;
            }
        }
    }

    fn clear_field_moves(&mut self, name: &str) {
        if let Some(stack) = self.field_move_stacks.get_mut(name) {
            if let Some(slot) = stack.last_mut() {
                slot.clear();
            }
        }
    }

    fn raw_addr_alias(&self, name: &str) -> Option<&str> {
        self.raw_addr_alias_stacks
            .get(name)
            .and_then(|stack| stack.last())
            .and_then(|slot| slot.as_deref())
    }

    fn set_raw_addr_alias(&mut self, name: &str, alias: Option<String>) {
        if let Some(stack) = self.raw_addr_alias_stacks.get_mut(name) {
            if let Some(slot) = stack.last_mut() {
                *slot = alias;
            }
        }
    }

    fn i32_const_alias(&self, name: &str) -> Option<i64> {
        self.i32_const_stacks
            .get(name)
            .and_then(|stack| stack.last())
            .and_then(|slot| *slot)
    }

    fn set_i32_const_alias(&mut self, name: &str, value: Option<i64>) {
        if let Some(stack) = self.i32_const_stacks.get_mut(name) {
            if let Some(slot) = stack.last_mut() {
                *slot = value;
            }
        }
    }

    fn function_value_aliases(&self, name: &str) -> BTreeSet<String> {
        self.function_value_alias_stacks
            .get(name)
            .and_then(|stack| stack.last())
            .cloned()
            .unwrap_or_default()
    }

    fn set_function_value_aliases(&mut self, name: &str, aliases: BTreeSet<String>) {
        if let Some(stack) = self.function_value_alias_stacks.get_mut(name) {
            if let Some(slot) = stack.last_mut() {
                *slot = aliases;
            }
        }
    }

    fn enum_payload_raw_alias(&self, name: &str, variant: &str) -> Option<&str> {
        let aliases = self
            .enum_payload_raw_alias_stacks
            .get(name)
            .and_then(|stack| stack.last())?;
        variant_alias(aliases, variant).map(String::as_str)
    }

    fn set_enum_payload_raw_aliases(&mut self, name: &str, aliases: BTreeMap<String, String>) {
        if let Some(stack) = self.enum_payload_raw_alias_stacks.get_mut(name) {
            if let Some(slot) = stack.last_mut() {
                *slot = aliases;
            }
        }
    }

    fn aggregate_field_raw_alias(&self, name: &str, offset: usize) -> Option<&str> {
        self.aggregate_field_raw_alias_stacks
            .get(name)
            .and_then(|stack| stack.last())
            .and_then(|aliases| aliases.get(&offset))
            .map(String::as_str)
    }

    fn aggregate_field_raw_aliases(&self, name: &str) -> BTreeMap<usize, String> {
        self.aggregate_field_raw_alias_stacks
            .get(name)
            .and_then(|stack| stack.last())
            .cloned()
            .unwrap_or_default()
    }

    fn set_aggregate_field_raw_aliases(&mut self, name: &str, aliases: BTreeMap<usize, String>) {
        if let Some(stack) = self.aggregate_field_raw_alias_stacks.get_mut(name) {
            if let Some(slot) = stack.last_mut() {
                *slot = aliases;
            }
        }
    }

    fn aggregate_field_function_aliases(&self, name: &str) -> BTreeMap<usize, BTreeSet<String>> {
        self.aggregate_field_function_alias_stacks
            .get(name)
            .and_then(|stack| stack.last())
            .cloned()
            .unwrap_or_default()
    }

    fn set_aggregate_field_function_aliases(
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

    fn enum_payload_aggregate_field_raw_aliases(
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

    fn set_enum_payload_aggregate_field_raw_aliases(
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

    fn enum_payload_aggregate_field_function_aliases(
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

    fn set_enum_payload_aggregate_field_function_aliases(
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

    fn enum_payload_function_aliases_for_variant(
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

    fn set_enum_payload_function_aliases(
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

    fn string_literal(&self, id: u32) -> Option<&str> {
        self.string_literals.get(id as usize).map(String::as_str)
    }

    fn has_field_moves(&self, name: &str) -> bool {
        self.field_move_stacks
            .get(name)
            .and_then(|stack| stack.last())
            .map(|moves| !moves.is_empty())
            .unwrap_or(false)
    }

    fn mark_field_moved(&mut self, path: &FieldMovePath) {
        if let Some(stack) = self.field_move_stacks.get_mut(path.owner.as_str()) {
            if let Some(slot) = stack.last_mut() {
                slot.insert(FieldMove {
                    offset: path.offset,
                    ty: path.field_ty,
                });
            }
        }
    }

    fn field_is_moved(&self, path: &FieldMovePath) -> bool {
        self.field_move_stacks
            .get(path.owner.as_str())
            .and_then(|stack| stack.last())
            .map(|moves| {
                moves.contains(&FieldMove {
                    offset: path.offset,
                    ty: path.field_ty,
                })
            })
            .unwrap_or(false)
    }

    fn set_state(&mut self, name: &str, state: VarState) {
        if let Some(stack) = self.var_stacks.get_mut(name) {
            if let Some(last) = stack.last_mut() {
                if *last == state {
                    return;
                }
                *last = state;
            }
        }
    }

    fn push_use_counts(&mut self, counts: BTreeMap<String, usize>) {
        self.use_counts.push(counts);
    }

    fn pop_use_counts(&mut self) {
        self.use_counts.pop();
    }

    fn remaining_uses(&self, name: &str) -> usize {
        self.use_counts
            .iter()
            .filter_map(|counts| counts.get(name))
            .sum()
    }

    fn note_var_use(&mut self, name: &str) {
        for counts in &mut self.use_counts {
            if let Some(count) = counts.get_mut(name) {
                *count = count.saturating_sub(1);
            }
        }
        if self.remaining_uses(name) == 0 {
            self.release_borrow_binding(name);
        }
    }

    fn increment_borrow_count(&mut self, name: &str, kind: BorrowKind) {
        let count = self.borrow_counts.entry(name.to_string()).or_default();
        match kind {
            BorrowKind::Shared => count.shared += 1,
            BorrowKind::Unique => count.unique += 1,
        }
    }

    fn release_borrow_binding(&mut self, name: &str) {
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

    fn release_borrow_bindings(&mut self, bindings: &[BorrowBinding]) {
        for binding in bindings {
            self.release_source_borrow(binding.source.as_str(), binding.kind);
        }
    }

    fn release_source_borrow(&mut self, source: &str, kind: BorrowKind) {
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

    fn check_borrow_escape(&mut self, source: &str, span: Span, escape_depth: usize) {
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

    fn check_binding_escape(&mut self, binding: &BorrowBinding, span: Span, escape_depth: usize) {
        self.check_borrow_escape(binding.source.as_str(), span, escape_depth);
    }

    fn check_expr_borrows_escape(
        &mut self,
        borrows: &[ExprBorrow],
        span: Span,
        escape_depth: usize,
    ) {
        for borrow in borrows {
            self.check_binding_escape(&borrow.binding, span, escape_depth);
        }
    }

    fn check_var_escape(&mut self, name: &str, span: Span, escape_depth: usize) {
        for binding in self.borrow_bindings(name) {
            self.check_binding_escape(&binding, span, escape_depth);
        }
    }

    fn retain_expr_borrows(&mut self, borrows: Vec<ExprBorrow>) -> Vec<BorrowBinding> {
        let mut bindings = Vec::with_capacity(borrows.len());
        for borrow in borrows {
            self.retain_borrow_binding(&borrow.binding);
            bindings.push(borrow.binding);
        }
        bindings
    }

    fn retain_borrow_binding(&mut self, binding: &BorrowBinding) {
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

    fn check_use(&mut self, name: &str, span: Span, is_copy: bool) {
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

    fn with_function_params(
        module: &'m HirModule,
        function_params: BTreeMap<String, Vec<TypeId>>,
        function_raw_alias_summaries: BTreeMap<String, FunctionRawAliasSummary>,
    ) -> Self {
        let mut ctx = Self::new(module);
        ctx.function_params = function_params;
        ctx.function_raw_alias_summaries = function_raw_alias_summaries;
        ctx
    }

    fn check_assign(&mut self, name: &str, span: Span) {
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

    fn check_drop(&mut self, name: &str, span: Span) {
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

    fn check_temporary_borrow(&mut self, name: &str, span: Span, kind: BorrowKind) {
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

    fn check_field_move(&mut self, path: &FieldMovePath, span: Span) {
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

    fn check_field_temporary_borrow(&mut self, path: &FieldMovePath, span: Span, kind: BorrowKind) {
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

    fn merge_state_pair(a: VarState, b: VarState) -> VarState {
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

    fn merge_states(states: &[VarState]) -> VarState {
        let mut it = states.iter().copied();
        let first = it.next().unwrap_or(VarState::Valid);
        it.fold(first, Self::merge_state_pair)
    }

    fn release_dead_borrows(&mut self) {
        let names: Vec<String> = self.borrow_stacks.keys().cloned().collect();
        for name in names {
            if self.remaining_uses(name.as_str()) == 0 {
                self.release_borrow_binding(name.as_str());
            }
        }
    }

    fn rebuild_borrow_counts_from_bindings(&mut self) {
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

    fn check_raw_non_copy_load(&mut self, place: &str, size: usize, span: Span) {
        let overlapping = self.overlapping_raw_places(place, size);
        if overlapping.iter().any(|(_, info)| {
            matches!(
                info.state,
                RawPlaceState::Moved | RawPlaceState::PossiblyMoved
            )
        }) {
            self.diagnostics.push(
                Diagnostic::error(
                    alloc::format!("use of moved raw memory place: `{}`", place),
                    span,
                )
                .with_id(DiagnosticId::TypeRawMemoryOwnershipViolation),
            );
            return;
        }
        let partial_load = overlapping
            .iter()
            .any(|(key, info)| key != place && info.state == RawPlaceState::Initialized);
        if partial_load {
            for (key, _) in overlapping {
                if key != place {
                    if let Some(info) = self.raw_place_states.get_mut(key.as_str()) {
                        if info.state == RawPlaceState::Initialized {
                            info.state = RawPlaceState::PossiblyMoved;
                        }
                    }
                }
            }
        }
        self.raw_place_states.insert(
            place.to_string(),
            RawPlaceInfo {
                state: RawPlaceState::Moved,
                size,
            },
        );
    }

    fn check_raw_non_copy_store(&mut self, place: &str, size: usize, span: Span) {
        if self
            .overlapping_raw_places(place, size)
            .iter()
            .any(|(_, info)| {
                matches!(
                    info.state,
                    RawPlaceState::Initialized | RawPlaceState::PossiblyMoved
                )
            })
        {
            self.diagnostics.push(
                Diagnostic::error(
                    alloc::format!(
                        "overwrite of raw memory place containing non-Copy value: `{}`",
                        place
                    ),
                    span,
                )
                .with_id(DiagnosticId::TypeRawMemoryOwnershipViolation),
            );
            return;
        }
        self.raw_place_states.insert(
            place.to_string(),
            RawPlaceInfo {
                state: RawPlaceState::Initialized,
                size,
            },
        );
    }

    fn check_raw_non_copy_dealloc(&mut self, place: &str, size: Option<usize>, span: Span) {
        if let Some((live_place, _)) = self
            .raw_places_overlapping_dealloc(place, size)
            .into_iter()
            .find(|(_, info)| {
                matches!(
                    info.state,
                    RawPlaceState::Initialized | RawPlaceState::PossiblyMoved
                )
            })
        {
            self.diagnostics.push(
                Diagnostic::error(
                    alloc::format!(
                        "deallocating raw memory place containing non-Copy value: `{}`",
                        live_place
                    ),
                    span,
                )
                .with_id(DiagnosticId::TypeRawMemoryOwnershipViolation),
            );
        }
    }

    fn check_raw_non_copy_realloc(&mut self, place: &str, size: Option<usize>, span: Span) {
        if let Some((live_place, _)) = self
            .raw_places_overlapping_dealloc(place, size)
            .into_iter()
            .find(|(_, info)| {
                matches!(
                    info.state,
                    RawPlaceState::Initialized | RawPlaceState::PossiblyMoved
                )
            })
        {
            self.diagnostics.push(
                Diagnostic::error(
                    alloc::format!(
                        "reallocating raw memory place containing non-Copy value: `{}`",
                        live_place
                    ),
                    span,
                )
                .with_id(DiagnosticId::TypeRawMemoryOwnershipViolation),
            );
        }
    }

    fn check_raw_non_copy_byte_write(&mut self, place: &str, size: Option<usize>, span: Span) {
        if let Some((live_place, _)) = self
            .raw_places_overlapping_dealloc(place, size)
            .into_iter()
            .find(|(_, info)| {
                matches!(
                    info.state,
                    RawPlaceState::Initialized | RawPlaceState::PossiblyMoved
                )
            })
        {
            self.diagnostics.push(
                Diagnostic::error(
                    alloc::format!(
                        "overwriting raw memory place containing non-Copy value: `{}`",
                        live_place
                    ),
                    span,
                )
                .with_id(DiagnosticId::TypeRawMemoryOwnershipViolation),
            );
        }
    }

    fn check_raw_non_copy_bulk_copy(
        &mut self,
        dst: &str,
        src: &str,
        size: Option<usize>,
        span: Span,
    ) {
        if let Some((live_place, _)) = self
            .raw_places_overlapping_dealloc(src, size)
            .into_iter()
            .find(|(_, info)| {
                matches!(
                    info.state,
                    RawPlaceState::Initialized | RawPlaceState::PossiblyMoved
                )
            })
        {
            self.diagnostics.push(
                Diagnostic::error(
                    alloc::format!(
                        "copying raw memory place containing non-Copy value: `{}`",
                        live_place
                    ),
                    span,
                )
                .with_id(DiagnosticId::TypeRawMemoryOwnershipViolation),
            );
            return;
        }

        if let Some((live_place, _)) = self
            .raw_places_overlapping_dealloc(dst, size)
            .into_iter()
            .find(|(_, info)| {
                matches!(
                    info.state,
                    RawPlaceState::Initialized | RawPlaceState::PossiblyMoved
                )
            })
        {
            self.diagnostics.push(
                Diagnostic::error(
                    alloc::format!(
                        "overwriting raw memory place containing non-Copy value: `{}`",
                        live_place
                    ),
                    span,
                )
                .with_id(DiagnosticId::TypeRawMemoryOwnershipViolation),
            );
        }
    }

    fn overlapping_raw_places(&self, place: &str, size: usize) -> Vec<(String, RawPlaceInfo)> {
        self.raw_place_states
            .iter()
            .filter(|(key, info)| raw_place_ranges_overlap(place, size, key.as_str(), info.size))
            .map(|(key, info)| (key.clone(), *info))
            .collect()
    }

    fn raw_places_overlapping_dealloc(
        &self,
        place: &str,
        size: Option<usize>,
    ) -> Vec<(String, RawPlaceInfo)> {
        if let Some(size) = size {
            if size == 0 {
                return Vec::new();
            }
            return self.overlapping_raw_places(place, size);
        }

        let (base, offset) = parse_raw_memory_place_key(place);
        self.raw_place_states
            .iter()
            .filter(|(key, info)| {
                let (tracked_base, tracked_offset) = parse_raw_memory_place_key(key.as_str());
                if tracked_base != base {
                    return false;
                }
                let (Some(offset), Some(tracked_offset)) = (offset, tracked_offset) else {
                    return true;
                };
                let tracked_end = tracked_offset.saturating_add(info.size as i64);
                tracked_end > offset || tracked_offset >= offset
            })
            .map(|(key, info)| (key.clone(), *info))
            .collect()
    }
}

fn singleton_function_alias(alias: String) -> BTreeSet<String> {
    let mut aliases = BTreeSet::new();
    aliases.insert(alias);
    aliases
}

fn function_value_aliases_from_value(
    value: &HirExpr,
    ctx: &MoveCheckContext,
    tctx: &crate::types::TypeCtx,
) -> BTreeSet<String> {
    match &value.kind {
        HirExprKind::FnValue(name) => singleton_function_alias(name.clone()),
        HirExprKind::Var(name) => ctx.function_value_aliases(name),
        HirExprKind::Call { .. } => {
            let aliases = function_value_aliases_from_field_projection(value, ctx, tctx);
            if aliases.is_empty() {
                function_call_raw_alias_summary(value, ctx, tctx)
                    .map(|summary| summary.function_value_aliases)
                    .unwrap_or_default()
            } else {
                aliases
            }
        }
        HirExprKind::Intrinsic { name, args, .. } if name == "load" && args.len() == 1 => {
            function_value_aliases_from_field_load(value, ctx, tctx)
        }
        _ => BTreeSet::new(),
    }
}

fn value_alias_summary_from_value(
    value: &HirExpr,
    ctx: &MoveCheckContext,
    tctx: &crate::types::TypeCtx,
) -> ValueAliasSummary {
    ValueAliasSummary {
        raw_addr_alias: raw_addr_alias_from_value(value, ctx, tctx),
        aggregate_field_raw_aliases: aggregate_field_raw_aliases_from_value(value, ctx, tctx),
        aggregate_field_function_aliases: aggregate_field_function_aliases_from_value(
            value, ctx, tctx,
        ),
        enum_payload_raw_aliases: enum_payload_raw_aliases_from_value(value, ctx, tctx),
        enum_payload_aggregate_field_raw_aliases:
            enum_payload_aggregate_field_raw_aliases_from_value(value, ctx, tctx),
        enum_payload_aggregate_field_function_aliases:
            enum_payload_aggregate_field_function_aliases_from_value(value, ctx, tctx),
        enum_payload_function_aliases: enum_payload_function_aliases_from_value(value, ctx, tctx),
        function_value_aliases: function_value_aliases_from_value(value, ctx, tctx),
    }
}

fn expression_function_value_aliases(
    value: &HirExpr,
    ctx: &MoveCheckContext,
    tctx: &crate::types::TypeCtx,
) -> BTreeSet<String> {
    let aliases = function_value_aliases_from_value(value, ctx, tctx);
    if aliases.is_empty() {
        expression_raw_alias_summary(value, ctx, tctx).function_value_aliases
    } else {
        aliases
    }
}

fn function_param_raw_alias_key(index: usize) -> String {
    alloc::format!("$param:{}", index)
}

fn function_param_field_raw_alias_key(index: usize, offset: usize) -> String {
    alloc::format!("$param_field:{}:{}", index, offset)
}

fn function_param_field_function_alias_key(index: usize, offset: usize) -> String {
    alloc::format!("$fnparam_field:{}:{}", index, offset)
}

fn function_param_enum_payload_raw_alias_key(index: usize, variant: &str) -> String {
    alloc::format!("$param_enum_payload:{}:{}", index, variant)
}

fn function_param_enum_payload_field_raw_alias_key(
    index: usize,
    variant: &str,
    offset: usize,
) -> String {
    alloc::format!("$param_enum_payload_field:{}:{}:{}", index, offset, variant)
}

fn function_param_enum_payload_field_function_alias_key(
    index: usize,
    variant: &str,
    offset: usize,
) -> String {
    alloc::format!(
        "$fnparam_enum_payload_field:{}:{}:{}",
        index,
        offset,
        variant
    )
}

fn function_param_enum_payload_function_alias_key(index: usize, variant: &str) -> String {
    alloc::format!("$fnparam_enum_payload:{}:{}", index, variant)
}

fn function_param_function_alias_key(index: usize) -> String {
    alloc::format!("$fnparam:{}", index)
}

fn is_function_param_function_alias_key(alias: &str) -> bool {
    alias
        .strip_prefix("$fnparam:")
        .and_then(|index_text| index_text.parse::<usize>().ok())
        .is_some()
}

fn is_function_type(tctx: &crate::types::TypeCtx, ty: TypeId) -> bool {
    matches!(tctx.get_ref(tctx.resolve_id(ty)), TypeKind::Function { .. })
}

fn aggregate_field_placeholder_aliases(
    tctx: &crate::types::TypeCtx,
    ty: TypeId,
    mut make_key: impl FnMut(usize) -> String,
) -> BTreeMap<usize, String> {
    fn collect(
        tctx: &crate::types::TypeCtx,
        ty: TypeId,
        base_offset: usize,
        out: &mut BTreeMap<usize, String>,
        visiting: &mut BTreeSet<TypeId>,
        make_key: &mut dyn FnMut(usize) -> String,
    ) {
        let resolved = tctx.resolve_named_type_id(ty);
        if !visiting.insert(resolved) {
            return;
        }
        for field in aggregate_fields_with_offsets(tctx, ty) {
            let offset = base_offset.saturating_add(field.offset);
            out.insert(offset, make_key(offset));
            collect(tctx, field.ty, offset, out, visiting, make_key);
        }
        visiting.remove(&resolved);
    }

    let mut out = BTreeMap::new();
    collect(tctx, ty, 0, &mut out, &mut BTreeSet::new(), &mut make_key);
    out
}

fn aggregate_field_function_placeholder_aliases(
    tctx: &crate::types::TypeCtx,
    ty: TypeId,
    mut make_key: impl FnMut(usize) -> String,
) -> BTreeMap<usize, BTreeSet<String>> {
    fn collect(
        tctx: &crate::types::TypeCtx,
        ty: TypeId,
        base_offset: usize,
        out: &mut BTreeMap<usize, BTreeSet<String>>,
        visiting: &mut BTreeSet<TypeId>,
        make_key: &mut dyn FnMut(usize) -> String,
    ) {
        let resolved = tctx.resolve_named_type_id(ty);
        if !visiting.insert(resolved) {
            return;
        }
        for field in aggregate_fields_with_offsets(tctx, ty) {
            let offset = base_offset.saturating_add(field.offset);
            if is_function_type(tctx, field.ty) {
                out.insert(offset, singleton_function_alias(make_key(offset)));
            }
            collect(tctx, field.ty, offset, out, visiting, make_key);
        }
        visiting.remove(&resolved);
    }

    let mut out = BTreeMap::new();
    collect(tctx, ty, 0, &mut out, &mut BTreeSet::new(), &mut make_key);
    out
}

fn enum_variants_for_type(
    tctx: &crate::types::TypeCtx,
    ty: TypeId,
) -> Vec<(String, Option<TypeId>)> {
    match tctx.get_ref(tctx.resolve_named_type_id(ty)) {
        TypeKind::Enum { variants, .. } => variants
            .iter()
            .map(|variant| (variant.name.clone(), variant.payload))
            .collect(),
        TypeKind::Apply { base, args } => {
            let base = tctx.resolve_named_type_id(*base);
            match tctx.get_ref(base) {
                TypeKind::Enum {
                    type_params,
                    variants,
                    ..
                } => {
                    let mapping = crate::layout::extend_type_mapping(
                        tctx,
                        &BTreeMap::new(),
                        type_params,
                        args,
                    );
                    variants
                        .iter()
                        .map(|variant| {
                            (
                                variant.name.clone(),
                                variant.payload.map(|payload| {
                                    crate::layout::mapped_type_id(tctx, payload, &mapping)
                                }),
                            )
                        })
                        .collect()
                }
                _ => Vec::new(),
            }
        }
        _ => Vec::new(),
    }
}

fn enum_payload_raw_alias_from_value(
    value: &HirExpr,
    variant: &str,
    ctx: &MoveCheckContext,
    tctx: &crate::types::TypeCtx,
) -> Option<String> {
    match &value.kind {
        HirExprKind::Var(name) => ctx
            .enum_payload_raw_alias(name, variant)
            .map(ToString::to_string),
        _ => {
            let aliases = enum_payload_raw_aliases_from_value(value, ctx, tctx);
            variant_alias(&aliases, variant).cloned()
        }
    }
}

fn enum_payload_aggregate_field_raw_aliases_from_expr(
    value: &HirExpr,
    variant: &str,
    ctx: &MoveCheckContext,
    tctx: &crate::types::TypeCtx,
) -> BTreeMap<usize, String> {
    match &value.kind {
        HirExprKind::Var(name) => ctx.enum_payload_aggregate_field_raw_aliases(name, variant),
        _ => {
            let aliases = enum_payload_aggregate_field_raw_aliases_from_value(value, ctx, tctx);
            variant_alias(&aliases, variant)
                .cloned()
                .unwrap_or_default()
        }
    }
}

fn enum_payload_aggregate_field_function_aliases_from_expr(
    value: &HirExpr,
    variant: &str,
    ctx: &MoveCheckContext,
    tctx: &crate::types::TypeCtx,
) -> BTreeMap<usize, BTreeSet<String>> {
    match &value.kind {
        HirExprKind::Var(name) => ctx.enum_payload_aggregate_field_function_aliases(name, variant),
        _ => {
            let aliases =
                enum_payload_aggregate_field_function_aliases_from_value(value, ctx, tctx);
            variant_alias(&aliases, variant)
                .cloned()
                .unwrap_or_default()
        }
    }
}

fn enum_payload_function_aliases_from_expr(
    value: &HirExpr,
    variant: &str,
    ctx: &MoveCheckContext,
    tctx: &crate::types::TypeCtx,
) -> BTreeSet<String> {
    match &value.kind {
        HirExprKind::Var(name) => ctx.enum_payload_function_aliases_for_variant(name, variant),
        _ => {
            let aliases = enum_payload_function_aliases_from_value(value, ctx, tctx);
            variant_alias(&aliases, variant)
                .cloned()
                .unwrap_or_default()
        }
    }
}

fn instantiate_raw_alias_base(
    base: &str,
    args: &[HirExpr],
    ctx: &MoveCheckContext,
    tctx: &crate::types::TypeCtx,
) -> Option<String> {
    if let Some(index_text) = base.strip_prefix("$param:") {
        let index = index_text.parse::<usize>().ok()?;
        return args
            .get(index)
            .and_then(|arg| raw_addr_alias_from_value(arg, ctx, tctx));
    }
    if let Some(rest) = base.strip_prefix("$param_field:") {
        let (index_text, offset_text) = rest.split_once(':')?;
        let index = index_text.parse::<usize>().ok()?;
        let offset = offset_text.parse::<usize>().ok()?;
        return args
            .get(index)
            .and_then(|arg| aggregate_field_raw_alias_at(arg, offset, ctx, tctx));
    }
    if let Some(rest) = base.strip_prefix("$param_enum_payload:") {
        let (index_text, variant) = rest.split_once(':')?;
        let index = index_text.parse::<usize>().ok()?;
        return args
            .get(index)
            .and_then(|arg| enum_payload_raw_alias_from_value(arg, variant, ctx, tctx));
    }
    if let Some(rest) = base.strip_prefix("$param_enum_payload_field:") {
        let (index_text, rest) = rest.split_once(':')?;
        let (offset_text, variant) = rest.split_once(':')?;
        let index = index_text.parse::<usize>().ok()?;
        let offset = offset_text.parse::<usize>().ok()?;
        return args.get(index).and_then(|arg| {
            enum_payload_aggregate_field_raw_aliases_from_expr(arg, variant, ctx, tctx)
                .get(&offset)
                .cloned()
        });
    }
    Some(base.to_string())
}

fn instantiate_raw_alias_key(
    key: &str,
    args: &[HirExpr],
    ctx: &MoveCheckContext,
    tctx: &crate::types::TypeCtx,
) -> Option<String> {
    let (base, offset) = parse_raw_memory_place_key(key);
    let instantiated = instantiate_raw_alias_base(base.as_str(), args, ctx, tctx)?;
    let (instantiated_base, instantiated_offset) =
        parse_raw_memory_place_key(instantiated.as_str());
    Some(format_raw_memory_place_key_parts(
        instantiated_base.as_str(),
        combine_raw_memory_offsets(instantiated_offset, offset),
    ))
}

fn instantiate_raw_alias_base_from_value_summaries(
    base: &str,
    args: &[ValueAliasSummary],
) -> Option<String> {
    if let Some(index_text) = base.strip_prefix("$param:") {
        let index = index_text.parse::<usize>().ok()?;
        return args.get(index)?.raw_addr_alias.clone();
    }
    if let Some(rest) = base.strip_prefix("$param_field:") {
        let (index_text, offset_text) = rest.split_once(':')?;
        let index = index_text.parse::<usize>().ok()?;
        let offset = offset_text.parse::<usize>().ok()?;
        return args
            .get(index)?
            .aggregate_field_raw_aliases
            .get(&offset)
            .cloned();
    }
    if let Some(rest) = base.strip_prefix("$param_enum_payload:") {
        let (index_text, variant) = rest.split_once(':')?;
        let index = index_text.parse::<usize>().ok()?;
        let aliases = &args.get(index)?.enum_payload_raw_aliases;
        return variant_alias(aliases, variant).cloned();
    }
    if let Some(rest) = base.strip_prefix("$param_enum_payload_field:") {
        let (index_text, rest) = rest.split_once(':')?;
        let (offset_text, variant) = rest.split_once(':')?;
        let index = index_text.parse::<usize>().ok()?;
        let offset = offset_text.parse::<usize>().ok()?;
        let aliases = &args.get(index)?.enum_payload_aggregate_field_raw_aliases;
        return variant_alias(aliases, variant)
            .and_then(|field_aliases| field_aliases.get(&offset))
            .cloned();
    }
    Some(base.to_string())
}

fn instantiate_raw_alias_key_from_value_summaries(
    key: &str,
    args: &[ValueAliasSummary],
) -> Option<String> {
    let (base, offset) = parse_raw_memory_place_key(key);
    let instantiated = instantiate_raw_alias_base_from_value_summaries(base.as_str(), args)?;
    let (instantiated_base, instantiated_offset) =
        parse_raw_memory_place_key(instantiated.as_str());
    Some(format_raw_memory_place_key_parts(
        instantiated_base.as_str(),
        combine_raw_memory_offsets(instantiated_offset, offset),
    ))
}

fn instantiate_function_value_alias_key(
    alias: &str,
    args: &[HirExpr],
    ctx: &MoveCheckContext,
    tctx: &crate::types::TypeCtx,
) -> BTreeSet<String> {
    if let Some(index_text) = alias.strip_prefix("$fnparam:") {
        let Some(index) = index_text.parse::<usize>().ok() else {
            return BTreeSet::new();
        };
        return args
            .get(index)
            .map(|arg| function_value_aliases_from_value(arg, ctx, tctx))
            .unwrap_or_default();
    }
    if let Some(rest) = alias.strip_prefix("$fnparam_field:") {
        let Some((index_text, offset_text)) = rest.split_once(':') else {
            return BTreeSet::new();
        };
        let Some(index) = index_text.parse::<usize>().ok() else {
            return BTreeSet::new();
        };
        let Some(offset) = offset_text.parse::<usize>().ok() else {
            return BTreeSet::new();
        };
        return args
            .get(index)
            .and_then(|arg| {
                aggregate_field_function_aliases_from_value(arg, ctx, tctx).remove(&offset)
            })
            .unwrap_or_default();
    }
    if let Some(rest) = alias.strip_prefix("$fnparam_enum_payload:") {
        let Some((index_text, variant)) = rest.split_once(':') else {
            return BTreeSet::new();
        };
        let Some(index) = index_text.parse::<usize>().ok() else {
            return BTreeSet::new();
        };
        return args
            .get(index)
            .map(|arg| enum_payload_function_aliases_from_expr(arg, variant, ctx, tctx))
            .unwrap_or_default();
    }
    if let Some(rest) = alias.strip_prefix("$fnparam_enum_payload_field:") {
        let Some((index_text, rest)) = rest.split_once(':') else {
            return BTreeSet::new();
        };
        let Some((offset_text, variant)) = rest.split_once(':') else {
            return BTreeSet::new();
        };
        let Some(index) = index_text.parse::<usize>().ok() else {
            return BTreeSet::new();
        };
        let Some(offset) = offset_text.parse::<usize>().ok() else {
            return BTreeSet::new();
        };
        return args
            .get(index)
            .and_then(|arg| {
                enum_payload_aggregate_field_function_aliases_from_expr(arg, variant, ctx, tctx)
                    .remove(&offset)
            })
            .unwrap_or_default();
    }
    singleton_function_alias(alias.to_string())
}

fn instantiate_function_value_alias_key_from_value_summaries(
    alias: &str,
    args: &[ValueAliasSummary],
) -> BTreeSet<String> {
    if let Some(index_text) = alias.strip_prefix("$fnparam:") {
        let Some(index) = index_text.parse::<usize>().ok() else {
            return BTreeSet::new();
        };
        return args
            .get(index)
            .map(|summary| summary.function_value_aliases.clone())
            .unwrap_or_default();
    }
    if let Some(rest) = alias.strip_prefix("$fnparam_field:") {
        let Some((index_text, offset_text)) = rest.split_once(':') else {
            return BTreeSet::new();
        };
        let Some(index) = index_text.parse::<usize>().ok() else {
            return BTreeSet::new();
        };
        let Some(offset) = offset_text.parse::<usize>().ok() else {
            return BTreeSet::new();
        };
        return args
            .get(index)
            .and_then(|summary| summary.aggregate_field_function_aliases.get(&offset))
            .cloned()
            .unwrap_or_default();
    }
    if let Some(rest) = alias.strip_prefix("$fnparam_enum_payload:") {
        let Some((index_text, variant)) = rest.split_once(':') else {
            return BTreeSet::new();
        };
        let Some(index) = index_text.parse::<usize>().ok() else {
            return BTreeSet::new();
        };
        let Some(aliases) = args
            .get(index)
            .map(|summary| &summary.enum_payload_function_aliases)
        else {
            return BTreeSet::new();
        };
        return variant_alias(aliases, variant).cloned().unwrap_or_default();
    }
    if let Some(rest) = alias.strip_prefix("$fnparam_enum_payload_field:") {
        let Some((index_text, rest)) = rest.split_once(':') else {
            return BTreeSet::new();
        };
        let Some((offset_text, variant)) = rest.split_once(':') else {
            return BTreeSet::new();
        };
        let Some(index) = index_text.parse::<usize>().ok() else {
            return BTreeSet::new();
        };
        let Some(offset) = offset_text.parse::<usize>().ok() else {
            return BTreeSet::new();
        };
        let Some(aliases) = args
            .get(index)
            .map(|summary| &summary.enum_payload_aggregate_field_function_aliases)
        else {
            return BTreeSet::new();
        };
        return variant_alias(aliases, variant)
            .and_then(|field_aliases| field_aliases.get(&offset))
            .cloned()
            .unwrap_or_default();
    }
    singleton_function_alias(alias.to_string())
}

fn instantiate_value_alias_summary(
    summary: &ValueAliasSummary,
    args: &[HirExpr],
    ctx: &MoveCheckContext,
    tctx: &crate::types::TypeCtx,
) -> ValueAliasSummary {
    ValueAliasSummary {
        raw_addr_alias: summary
            .raw_addr_alias
            .as_ref()
            .and_then(|alias| instantiate_raw_alias_key(alias, args, ctx, tctx)),
        aggregate_field_raw_aliases: summary
            .aggregate_field_raw_aliases
            .iter()
            .filter_map(|(offset, alias)| {
                instantiate_raw_alias_key(alias, args, ctx, tctx).map(|alias| (*offset, alias))
            })
            .collect(),
        aggregate_field_function_aliases: summary
            .aggregate_field_function_aliases
            .iter()
            .filter_map(|(offset, aliases)| {
                let mut instantiated = BTreeSet::new();
                for alias in aliases {
                    instantiated
                        .extend(instantiate_function_value_alias_key(alias, args, ctx, tctx));
                }
                if instantiated.is_empty() {
                    None
                } else {
                    Some((*offset, instantiated))
                }
            })
            .collect(),
        enum_payload_raw_aliases: summary
            .enum_payload_raw_aliases
            .iter()
            .filter_map(|(variant, alias)| {
                instantiate_raw_alias_key(alias, args, ctx, tctx)
                    .map(|alias| (variant.clone(), alias))
            })
            .collect(),
        enum_payload_aggregate_field_raw_aliases: summary
            .enum_payload_aggregate_field_raw_aliases
            .iter()
            .filter_map(|(variant, aliases)| {
                let instantiated = aliases
                    .iter()
                    .filter_map(|(offset, alias)| {
                        instantiate_raw_alias_key(alias, args, ctx, tctx)
                            .map(|alias| (*offset, alias))
                    })
                    .collect::<BTreeMap<_, _>>();
                if instantiated.is_empty() {
                    None
                } else {
                    Some((variant.clone(), instantiated))
                }
            })
            .collect(),
        enum_payload_aggregate_field_function_aliases: summary
            .enum_payload_aggregate_field_function_aliases
            .iter()
            .filter_map(|(variant, aliases)| {
                let instantiated = aliases
                    .iter()
                    .filter_map(|(offset, aliases)| {
                        let mut instantiated = BTreeSet::new();
                        for alias in aliases {
                            instantiated.extend(instantiate_function_value_alias_key(
                                alias, args, ctx, tctx,
                            ));
                        }
                        if instantiated.is_empty() {
                            None
                        } else {
                            Some((*offset, instantiated))
                        }
                    })
                    .collect::<BTreeMap<_, _>>();
                if instantiated.is_empty() {
                    None
                } else {
                    Some((variant.clone(), instantiated))
                }
            })
            .collect(),
        enum_payload_function_aliases: summary
            .enum_payload_function_aliases
            .iter()
            .filter_map(|(variant, aliases)| {
                let mut instantiated = BTreeSet::new();
                for alias in aliases {
                    instantiated
                        .extend(instantiate_function_value_alias_key(alias, args, ctx, tctx));
                }
                if instantiated.is_empty() {
                    None
                } else {
                    Some((variant.clone(), instantiated))
                }
            })
            .collect(),
        function_value_aliases: {
            let mut aliases = BTreeSet::new();
            for alias in &summary.function_value_aliases {
                aliases.extend(instantiate_function_value_alias_key(alias, args, ctx, tctx));
            }
            aliases
        },
    }
}

fn instantiate_value_alias_summary_from_value_summaries(
    summary: &ValueAliasSummary,
    args: &[ValueAliasSummary],
) -> ValueAliasSummary {
    ValueAliasSummary {
        raw_addr_alias: summary
            .raw_addr_alias
            .as_ref()
            .and_then(|alias| instantiate_raw_alias_key_from_value_summaries(alias, args)),
        aggregate_field_raw_aliases: summary
            .aggregate_field_raw_aliases
            .iter()
            .filter_map(|(offset, alias)| {
                instantiate_raw_alias_key_from_value_summaries(alias, args)
                    .map(|alias| (*offset, alias))
            })
            .collect(),
        aggregate_field_function_aliases: summary
            .aggregate_field_function_aliases
            .iter()
            .filter_map(|(offset, aliases)| {
                let mut instantiated = BTreeSet::new();
                for alias in aliases {
                    instantiated.extend(instantiate_function_value_alias_key_from_value_summaries(
                        alias, args,
                    ));
                }
                if instantiated.is_empty() {
                    None
                } else {
                    Some((*offset, instantiated))
                }
            })
            .collect(),
        enum_payload_raw_aliases: summary
            .enum_payload_raw_aliases
            .iter()
            .filter_map(|(variant, alias)| {
                instantiate_raw_alias_key_from_value_summaries(alias, args)
                    .map(|alias| (variant.clone(), alias))
            })
            .collect(),
        enum_payload_aggregate_field_raw_aliases: summary
            .enum_payload_aggregate_field_raw_aliases
            .iter()
            .filter_map(|(variant, aliases)| {
                let instantiated = aliases
                    .iter()
                    .filter_map(|(offset, alias)| {
                        instantiate_raw_alias_key_from_value_summaries(alias, args)
                            .map(|alias| (*offset, alias))
                    })
                    .collect::<BTreeMap<_, _>>();
                if instantiated.is_empty() {
                    None
                } else {
                    Some((variant.clone(), instantiated))
                }
            })
            .collect(),
        enum_payload_aggregate_field_function_aliases: summary
            .enum_payload_aggregate_field_function_aliases
            .iter()
            .filter_map(|(variant, aliases)| {
                let instantiated = aliases
                    .iter()
                    .filter_map(|(offset, aliases)| {
                        let mut instantiated = BTreeSet::new();
                        for alias in aliases {
                            instantiated.extend(
                                instantiate_function_value_alias_key_from_value_summaries(
                                    alias, args,
                                ),
                            );
                        }
                        if instantiated.is_empty() {
                            None
                        } else {
                            Some((*offset, instantiated))
                        }
                    })
                    .collect::<BTreeMap<_, _>>();
                if instantiated.is_empty() {
                    None
                } else {
                    Some((variant.clone(), instantiated))
                }
            })
            .collect(),
        enum_payload_function_aliases: summary
            .enum_payload_function_aliases
            .iter()
            .filter_map(|(variant, aliases)| {
                let mut instantiated = BTreeSet::new();
                for alias in aliases {
                    instantiated.extend(instantiate_function_value_alias_key_from_value_summaries(
                        alias, args,
                    ));
                }
                if instantiated.is_empty() {
                    None
                } else {
                    Some((variant.clone(), instantiated))
                }
            })
            .collect(),
        function_value_aliases: {
            let mut aliases = BTreeSet::new();
            for alias in &summary.function_value_aliases {
                aliases.extend(instantiate_function_value_alias_key_from_value_summaries(
                    alias, args,
                ));
            }
            aliases
        },
    }
}

fn instantiate_function_raw_alias_summary(
    summary: &FunctionRawAliasSummary,
    args: &[HirExpr],
    ctx: &MoveCheckContext,
    tctx: &crate::types::TypeCtx,
) -> FunctionRawAliasSummary {
    let value = ValueAliasSummary {
        raw_addr_alias: summary.raw_addr_alias.clone(),
        aggregate_field_raw_aliases: summary.aggregate_field_raw_aliases.clone(),
        aggregate_field_function_aliases: summary.aggregate_field_function_aliases.clone(),
        enum_payload_raw_aliases: summary.enum_payload_raw_aliases.clone(),
        enum_payload_aggregate_field_raw_aliases: summary
            .enum_payload_aggregate_field_raw_aliases
            .clone(),
        enum_payload_aggregate_field_function_aliases: summary
            .enum_payload_aggregate_field_function_aliases
            .clone(),
        enum_payload_function_aliases: summary.enum_payload_function_aliases.clone(),
        function_value_aliases: summary.function_value_aliases.clone(),
    };
    let value = instantiate_value_alias_summary(&value, args, ctx, tctx);
    let mut raw_memory_effects = Vec::new();
    let max_depth = ctx.function_raw_alias_summaries.len().saturating_add(1);
    for effect in &summary.raw_memory_effects {
        extend_unique_raw_memory_effects(
            &mut raw_memory_effects,
            instantiate_raw_memory_effect_summary(effect, args, ctx, tctx, max_depth),
        );
    }
    FunctionRawAliasSummary {
        raw_addr_alias: value.raw_addr_alias,
        aggregate_field_raw_aliases: value.aggregate_field_raw_aliases,
        aggregate_field_function_aliases: value.aggregate_field_function_aliases,
        enum_payload_raw_aliases: value.enum_payload_raw_aliases,
        enum_payload_aggregate_field_raw_aliases: value.enum_payload_aggregate_field_raw_aliases,
        enum_payload_aggregate_field_function_aliases: value
            .enum_payload_aggregate_field_function_aliases,
        enum_payload_function_aliases: value.enum_payload_function_aliases,
        function_value_aliases: value.function_value_aliases,
        raw_memory_effects,
    }
}

fn instantiate_raw_memory_effect_summary(
    effect: &RawMemoryEffectSummary,
    args: &[HirExpr],
    ctx: &MoveCheckContext,
    tctx: &crate::types::TypeCtx,
    remaining_depth: usize,
) -> Vec<RawMemoryEffectSummary> {
    match effect {
        RawMemoryEffectSummary::Load { place, size } => {
            instantiate_raw_alias_key(place, args, ctx, tctx)
                .map(|place| RawMemoryEffectSummary::Load { place, size: *size })
                .into_iter()
                .collect()
        }
        RawMemoryEffectSummary::Store { place, size } => {
            instantiate_raw_alias_key(place, args, ctx, tctx)
                .map(|place| RawMemoryEffectSummary::Store { place, size: *size })
                .into_iter()
                .collect()
        }
        RawMemoryEffectSummary::Dealloc { place, size } => {
            instantiate_raw_alias_key(place, args, ctx, tctx)
                .map(|place| RawMemoryEffectSummary::Dealloc { place, size: *size })
                .into_iter()
                .collect()
        }
        RawMemoryEffectSummary::Realloc { place, size } => {
            instantiate_raw_alias_key(place, args, ctx, tctx)
                .map(|place| RawMemoryEffectSummary::Realloc { place, size: *size })
                .into_iter()
                .collect()
        }
        RawMemoryEffectSummary::BulkCopy { dst, src, size } => {
            match (
                instantiate_raw_alias_key(dst, args, ctx, tctx),
                instantiate_raw_alias_key(src, args, ctx, tctx),
            ) {
                (Some(dst), Some(src)) => alloc::vec![RawMemoryEffectSummary::BulkCopy {
                    dst,
                    src,
                    size: *size,
                }],
                _ => Vec::new(),
            }
        }
        RawMemoryEffectSummary::ByteWrite { place, size } => {
            instantiate_raw_alias_key(place, args, ctx, tctx)
                .map(|place| RawMemoryEffectSummary::ByteWrite { place, size: *size })
                .into_iter()
                .collect()
        }
        RawMemoryEffectSummary::IndirectCall {
            callee,
            args: call_args,
        } => {
            let callees = instantiate_function_value_alias_key(callee, args, ctx, tctx);
            if callees.is_empty() {
                return Vec::new();
            }
            let instantiated_args = call_args
                .iter()
                .map(|arg| instantiate_value_alias_summary(arg, args, ctx, tctx))
                .collect::<Vec<_>>();
            let mut out = Vec::new();
            for callee in callees {
                let effects = instantiate_known_function_raw_memory_effects(
                    callee.as_str(),
                    &instantiated_args,
                    ctx,
                    tctx,
                    remaining_depth.saturating_sub(1),
                );
                if effects.is_empty() && is_function_param_function_alias_key(callee.as_str()) {
                    extend_unique_raw_memory_effects(
                        &mut out,
                        [RawMemoryEffectSummary::IndirectCall {
                            callee,
                            args: instantiated_args.clone(),
                        }],
                    );
                } else {
                    extend_unique_raw_memory_effects(&mut out, effects);
                }
            }
            out
        }
    }
}

fn instantiate_function_raw_alias_summary_from_value_summaries(
    summary: &FunctionRawAliasSummary,
    args: &[ValueAliasSummary],
    ctx: &MoveCheckContext,
    tctx: &crate::types::TypeCtx,
    remaining_depth: usize,
) -> FunctionRawAliasSummary {
    let value = ValueAliasSummary {
        raw_addr_alias: summary.raw_addr_alias.clone(),
        aggregate_field_raw_aliases: summary.aggregate_field_raw_aliases.clone(),
        aggregate_field_function_aliases: summary.aggregate_field_function_aliases.clone(),
        enum_payload_raw_aliases: summary.enum_payload_raw_aliases.clone(),
        enum_payload_aggregate_field_raw_aliases: summary
            .enum_payload_aggregate_field_raw_aliases
            .clone(),
        enum_payload_aggregate_field_function_aliases: summary
            .enum_payload_aggregate_field_function_aliases
            .clone(),
        enum_payload_function_aliases: summary.enum_payload_function_aliases.clone(),
        function_value_aliases: summary.function_value_aliases.clone(),
    };
    let value = instantiate_value_alias_summary_from_value_summaries(&value, args);
    let mut raw_memory_effects = Vec::new();
    for effect in &summary.raw_memory_effects {
        extend_unique_raw_memory_effects(
            &mut raw_memory_effects,
            instantiate_raw_memory_effect_summary_from_value_summaries(
                effect,
                args,
                ctx,
                tctx,
                remaining_depth,
            ),
        );
    }
    FunctionRawAliasSummary {
        raw_addr_alias: value.raw_addr_alias,
        aggregate_field_raw_aliases: value.aggregate_field_raw_aliases,
        aggregate_field_function_aliases: value.aggregate_field_function_aliases,
        enum_payload_raw_aliases: value.enum_payload_raw_aliases,
        enum_payload_aggregate_field_raw_aliases: value.enum_payload_aggregate_field_raw_aliases,
        enum_payload_aggregate_field_function_aliases: value
            .enum_payload_aggregate_field_function_aliases,
        enum_payload_function_aliases: value.enum_payload_function_aliases,
        function_value_aliases: value.function_value_aliases,
        raw_memory_effects,
    }
}

fn instantiate_raw_memory_effect_summary_from_value_summaries(
    effect: &RawMemoryEffectSummary,
    args: &[ValueAliasSummary],
    ctx: &MoveCheckContext,
    tctx: &crate::types::TypeCtx,
    remaining_depth: usize,
) -> Vec<RawMemoryEffectSummary> {
    match effect {
        RawMemoryEffectSummary::Load { place, size } => {
            instantiate_raw_alias_key_from_value_summaries(place, args)
                .map(|place| RawMemoryEffectSummary::Load { place, size: *size })
                .into_iter()
                .collect()
        }
        RawMemoryEffectSummary::Store { place, size } => {
            instantiate_raw_alias_key_from_value_summaries(place, args)
                .map(|place| RawMemoryEffectSummary::Store { place, size: *size })
                .into_iter()
                .collect()
        }
        RawMemoryEffectSummary::Dealloc { place, size } => {
            instantiate_raw_alias_key_from_value_summaries(place, args)
                .map(|place| RawMemoryEffectSummary::Dealloc { place, size: *size })
                .into_iter()
                .collect()
        }
        RawMemoryEffectSummary::Realloc { place, size } => {
            instantiate_raw_alias_key_from_value_summaries(place, args)
                .map(|place| RawMemoryEffectSummary::Realloc { place, size: *size })
                .into_iter()
                .collect()
        }
        RawMemoryEffectSummary::BulkCopy { dst, src, size } => {
            match (
                instantiate_raw_alias_key_from_value_summaries(dst, args),
                instantiate_raw_alias_key_from_value_summaries(src, args),
            ) {
                (Some(dst), Some(src)) => alloc::vec![RawMemoryEffectSummary::BulkCopy {
                    dst,
                    src,
                    size: *size,
                }],
                _ => Vec::new(),
            }
        }
        RawMemoryEffectSummary::ByteWrite { place, size } => {
            instantiate_raw_alias_key_from_value_summaries(place, args)
                .map(|place| RawMemoryEffectSummary::ByteWrite { place, size: *size })
                .into_iter()
                .collect()
        }
        RawMemoryEffectSummary::IndirectCall {
            callee,
            args: call_args,
        } => {
            let callees = instantiate_function_value_alias_key_from_value_summaries(callee, args);
            if callees.is_empty() {
                return Vec::new();
            }
            let instantiated_args = call_args
                .iter()
                .map(|arg| instantiate_value_alias_summary_from_value_summaries(arg, args))
                .collect::<Vec<_>>();
            let mut out = Vec::new();
            for callee in callees {
                let effects = instantiate_known_function_raw_memory_effects(
                    callee.as_str(),
                    &instantiated_args,
                    ctx,
                    tctx,
                    remaining_depth.saturating_sub(1),
                );
                if effects.is_empty() && is_function_param_function_alias_key(callee.as_str()) {
                    extend_unique_raw_memory_effects(
                        &mut out,
                        [RawMemoryEffectSummary::IndirectCall {
                            callee,
                            args: instantiated_args.clone(),
                        }],
                    );
                } else {
                    extend_unique_raw_memory_effects(&mut out, effects);
                }
            }
            out
        }
    }
}

fn instantiate_known_function_raw_memory_effects(
    callee: &str,
    args: &[ValueAliasSummary],
    ctx: &MoveCheckContext,
    tctx: &crate::types::TypeCtx,
    remaining_depth: usize,
) -> Vec<RawMemoryEffectSummary> {
    if remaining_depth == 0 {
        return Vec::new();
    }
    let Some(summary) = ctx.function_raw_alias_summaries.get(callee) else {
        return Vec::new();
    };
    instantiate_function_raw_alias_summary_from_value_summaries(
        summary,
        args,
        ctx,
        tctx,
        remaining_depth,
    )
    .raw_memory_effects
}

fn raw_alias_summary_needs_call_site_specialization(summary: &FunctionRawAliasSummary) -> bool {
    summary
        .raw_addr_alias
        .as_deref()
        .is_some_and(raw_place_key_has_unknown_offset)
}

fn specialized_function_raw_alias_summary(
    name: &str,
    args: &[HirExpr],
    ctx: &MoveCheckContext,
    tctx: &crate::types::TypeCtx,
) -> Option<FunctionRawAliasSummary> {
    if ctx
        .raw_alias_specialization_stack
        .iter()
        .any(|active| active == name)
    {
        return None;
    }
    let func = ctx.function_defs.get(name)?;
    if func.params.len() != args.len() {
        return None;
    }
    let mut call_ctx = ctx.clone_for_alias_summary();
    call_ctx
        .raw_alias_specialization_stack
        .push(name.to_string());
    call_ctx.push_scope();
    for (param, arg) in func.params.iter().zip(args) {
        let value_summary = value_alias_summary_from_value(arg, ctx, tctx);
        let i32_const_alias = i32_const_from_value(arg, ctx, tctx);
        call_ctx.declare_var(param.name.clone());
        call_ctx.set_raw_addr_alias(&param.name, value_summary.raw_addr_alias);
        call_ctx.set_i32_const_alias(&param.name, i32_const_alias);
        call_ctx.set_enum_payload_raw_aliases(&param.name, value_summary.enum_payload_raw_aliases);
        call_ctx.set_aggregate_field_raw_aliases(
            &param.name,
            value_summary.aggregate_field_raw_aliases,
        );
        call_ctx.set_aggregate_field_function_aliases(
            &param.name,
            value_summary.aggregate_field_function_aliases,
        );
        call_ctx.set_enum_payload_aggregate_field_raw_aliases(
            &param.name,
            value_summary.enum_payload_aggregate_field_raw_aliases,
        );
        call_ctx.set_enum_payload_aggregate_field_function_aliases(
            &param.name,
            value_summary.enum_payload_aggregate_field_function_aliases,
        );
        call_ctx.set_enum_payload_function_aliases(
            &param.name,
            value_summary.enum_payload_function_aliases,
        );
        call_ctx.set_function_value_aliases(&param.name, value_summary.function_value_aliases);
    }
    match &func.body {
        crate::hir::HirBody::Block(block) => Some(block_raw_alias_summary(block, &call_ctx, tctx)),
        _ => None,
    }
}

fn function_call_raw_alias_summary(
    expr: &HirExpr,
    ctx: &MoveCheckContext,
    tctx: &crate::types::TypeCtx,
) -> Option<FunctionRawAliasSummary> {
    let HirExprKind::Call { callee, args } = &expr.kind else {
        return None;
    };
    let FuncRef::User(name, _, _) = callee else {
        return None;
    };
    let summary = ctx.function_raw_alias_summaries.get(name)?;
    let instantiated = instantiate_function_raw_alias_summary(summary, args, ctx, tctx);
    if raw_alias_summary_needs_call_site_specialization(&instantiated) {
        specialized_function_raw_alias_summary(name, args, ctx, tctx).or(Some(instantiated))
    } else {
        Some(instantiated)
    }
}

fn aggregate_field_index_by_name(
    tctx: &crate::types::TypeCtx,
    ty: TypeId,
    field_name: &str,
) -> Option<usize> {
    let ty = tctx.resolve_named_type_id(ty);
    match tctx.get_ref(ty) {
        TypeKind::Struct { field_names, .. } => {
            field_names.iter().position(|name| name == field_name)
        }
        TypeKind::Tuple { items } => field_name
            .parse::<usize>()
            .ok()
            .filter(|index| *index < items.len()),
        TypeKind::Apply { base, .. } => {
            let base = tctx.resolve_named_type_id(*base);
            match tctx.get_ref(base) {
                TypeKind::Struct { field_names, .. } => {
                    field_names.iter().position(|name| name == field_name)
                }
                TypeKind::Tuple { items } => field_name
                    .parse::<usize>()
                    .ok()
                    .filter(|index| *index < items.len()),
                _ => None,
            }
        }
        _ => None,
    }
}

fn aggregate_field_layout_from_selector(
    owner_ty: TypeId,
    selector: &HirExpr,
    ctx: &MoveCheckContext,
    tctx: &crate::types::TypeCtx,
) -> Option<(usize, TypeId)> {
    let index = match &selector.kind {
        HirExprKind::LiteralI32(value) if *value >= 0 => Some(*value as usize),
        HirExprKind::LiteralStr(id) => {
            let field_name = ctx.string_literal(*id)?;
            aggregate_field_index_by_name(tctx, owner_ty, field_name)
        }
        _ => None,
    }?;
    aggregate_fields_with_offsets(tctx, owner_ty)
        .get(index)
        .map(|field| (field.offset, field.ty))
}

fn field_get_projection<'a>(
    expr: &'a HirExpr,
    ctx: &MoveCheckContext,
    tctx: &crate::types::TypeCtx,
) -> Option<(&'a HirExpr, usize, TypeId)> {
    let HirExprKind::Call { callee, args } = &expr.kind else {
        return None;
    };
    if args.len() < 2 {
        return None;
    }
    let name = func_ref_name(callee)?;
    if !is_field_get_name(name) {
        return None;
    }
    let (offset, field_ty) = aggregate_field_layout_from_selector(args[0].ty, &args[1], ctx, tctx)?;
    Some((&args[0], offset, field_ty))
}

fn is_result_ok_variant_name(name: &str) -> bool {
    name == "Ok" || name.ends_with("::Ok")
}

fn variant_short_name(name: &str) -> &str {
    name.rsplit("::").next().unwrap_or(name)
}

fn variant_alias<'a, T>(aliases: &'a BTreeMap<String, T>, variant: &str) -> Option<&'a T> {
    aliases.get(variant).or_else(|| {
        let short = variant_short_name(variant);
        aliases.get(short).or_else(|| {
            aliases.iter().find_map(|(key, value)| {
                if variant_short_name(key.as_str()) == short {
                    Some(value)
                } else {
                    None
                }
            })
        })
    })
}

fn pattern_variant_name(arm: &HirMatchArm) -> Option<&str> {
    match &arm.pattern {
        HirMatchPattern::Variant(name) => Some(name.as_str()),
        _ => None,
    }
}

fn region_ptr_at_result_ok_raw_alias(
    scrutinee: &HirExpr,
    ctx: &MoveCheckContext,
    tctx: &crate::types::TypeCtx,
) -> Option<String> {
    let HirExprKind::Call { callee, args } = &scrutinee.kind else {
        return None;
    };
    let name = func_ref_name(callee)?;
    if !is_region_ptr_at_name(name) || args.len() < 2 {
        return None;
    }
    let key = raw_memory_place_key_from_region_token(&args[0], ctx, tctx)?;
    let offset = match &args[1].kind {
        HirExprKind::LiteralI32(value) => Some(i64::from(*value)),
        _ => None,
    };
    let (base, base_offset) = parse_raw_memory_place_key(key.as_str());
    Some(format_raw_memory_place_key_parts(
        base.as_str(),
        combine_raw_memory_offsets(base_offset, offset),
    ))
}

fn match_bind_raw_addr_alias(
    scrutinee: &HirExpr,
    arm: &HirMatchArm,
    ctx: &MoveCheckContext,
    tctx: &crate::types::TypeCtx,
) -> Option<String> {
    let variant_name = pattern_variant_name(arm)?;
    if let Some(alias) = enum_payload_raw_alias_from_value(scrutinee, variant_name, ctx, tctx) {
        return Some(alias);
    }
    if is_result_ok_variant_name(variant_name) {
        region_ptr_at_result_ok_raw_alias(scrutinee, ctx, tctx)
    } else {
        None
    }
}

fn match_bind_aggregate_field_raw_aliases(
    scrutinee: &HirExpr,
    arm: &HirMatchArm,
    ctx: &MoveCheckContext,
    tctx: &crate::types::TypeCtx,
) -> BTreeMap<usize, String> {
    let Some(variant_name) = pattern_variant_name(arm) else {
        return BTreeMap::new();
    };
    enum_payload_aggregate_field_raw_aliases_from_expr(scrutinee, variant_name, ctx, tctx)
}

fn match_bind_aggregate_field_function_aliases(
    scrutinee: &HirExpr,
    arm: &HirMatchArm,
    ctx: &MoveCheckContext,
    tctx: &crate::types::TypeCtx,
) -> BTreeMap<usize, BTreeSet<String>> {
    let Some(variant_name) = pattern_variant_name(arm) else {
        return BTreeMap::new();
    };
    enum_payload_aggregate_field_function_aliases_from_expr(scrutinee, variant_name, ctx, tctx)
}

fn match_bind_function_value_aliases(
    scrutinee: &HirExpr,
    arm: &HirMatchArm,
    ctx: &MoveCheckContext,
    tctx: &crate::types::TypeCtx,
) -> BTreeSet<String> {
    let Some(variant_name) = pattern_variant_name(arm) else {
        return BTreeSet::new();
    };
    enum_payload_function_aliases_from_expr(scrutinee, variant_name, ctx, tctx)
}

fn enum_payload_raw_aliases_from_value(
    value: &HirExpr,
    ctx: &MoveCheckContext,
    tctx: &crate::types::TypeCtx,
) -> BTreeMap<String, String> {
    let mut aliases = BTreeMap::new();
    match &value.kind {
        HirExprKind::EnumConstruct {
            variant,
            payload: Some(payload),
            ..
        } => {
            if let Some(alias) = raw_addr_alias_from_value(payload, ctx, tctx) {
                aliases.insert(variant.clone(), alias);
            }
        }
        _ => {
            if let Some(alias) = region_ptr_at_result_ok_raw_alias(value, ctx, tctx) {
                aliases.insert(String::from("Ok"), alias);
            } else if let Some(summary) = function_call_raw_alias_summary(value, ctx, tctx) {
                aliases = summary.enum_payload_raw_aliases;
            }
        }
    }
    aliases
}

fn enum_payload_aggregate_field_raw_aliases_from_value(
    value: &HirExpr,
    ctx: &MoveCheckContext,
    tctx: &crate::types::TypeCtx,
) -> BTreeMap<String, BTreeMap<usize, String>> {
    let mut aliases = BTreeMap::new();
    if let HirExprKind::EnumConstruct {
        variant,
        payload: Some(payload),
        ..
    } = &value.kind
    {
        let aggregate_aliases = aggregate_field_raw_aliases_from_value(payload, ctx, tctx);
        if !aggregate_aliases.is_empty() {
            aliases.insert(variant.clone(), aggregate_aliases);
        }
    } else if let Some(summary) = function_call_raw_alias_summary(value, ctx, tctx) {
        aliases = summary.enum_payload_aggregate_field_raw_aliases;
    }
    aliases
}

fn enum_payload_aggregate_field_function_aliases_from_value(
    value: &HirExpr,
    ctx: &MoveCheckContext,
    tctx: &crate::types::TypeCtx,
) -> BTreeMap<String, BTreeMap<usize, BTreeSet<String>>> {
    let mut aliases = BTreeMap::new();
    if let HirExprKind::EnumConstruct {
        variant,
        payload: Some(payload),
        ..
    } = &value.kind
    {
        let aggregate_aliases = aggregate_field_function_aliases_from_value(payload, ctx, tctx);
        if !aggregate_aliases.is_empty() {
            aliases.insert(variant.clone(), aggregate_aliases);
        }
    } else if let Some(summary) = function_call_raw_alias_summary(value, ctx, tctx) {
        aliases = summary.enum_payload_aggregate_field_function_aliases;
    }
    aliases
}

fn enum_payload_function_aliases_from_value(
    value: &HirExpr,
    ctx: &MoveCheckContext,
    tctx: &crate::types::TypeCtx,
) -> BTreeMap<String, BTreeSet<String>> {
    let mut aliases = BTreeMap::new();
    match &value.kind {
        HirExprKind::EnumConstruct {
            variant,
            payload: Some(payload),
            ..
        } => {
            let function_aliases = function_value_aliases_from_value(payload, ctx, tctx);
            if !function_aliases.is_empty() {
                aliases.insert(variant.clone(), function_aliases);
            }
        }
        _ => {
            if let Some(summary) = function_call_raw_alias_summary(value, ctx, tctx) {
                aliases = summary.enum_payload_function_aliases;
            }
        }
    }
    aliases
}

fn aggregate_field_raw_aliases_from_items(
    value_ty: TypeId,
    items: &[HirExpr],
    ctx: &MoveCheckContext,
    tctx: &crate::types::TypeCtx,
) -> BTreeMap<usize, String> {
    let layouts = aggregate_fields_with_offsets(tctx, value_ty);
    let mut aliases = BTreeMap::new();
    for (item, layout) in items.iter().zip(layouts.into_iter()) {
        if let Some(alias) = raw_addr_alias_from_value(item, ctx, tctx) {
            aliases.insert(layout.offset, alias);
        }
        for (nested_offset, alias) in aggregate_field_raw_aliases_from_value(item, ctx, tctx) {
            aliases.insert(layout.offset.saturating_add(nested_offset), alias);
        }
    }
    aliases
}

fn aggregate_field_raw_aliases_from_projection(
    owner: &HirExpr,
    field_offset: usize,
    field_ty: TypeId,
    ctx: &MoveCheckContext,
    tctx: &crate::types::TypeCtx,
) -> BTreeMap<usize, String> {
    let field_size = storage_size_bytes(tctx, field_ty);
    if field_size == 0 {
        return BTreeMap::new();
    }
    let field_end = field_offset.saturating_add(field_size);
    aggregate_field_raw_aliases_from_value(owner, ctx, tctx)
        .into_iter()
        .filter_map(|(offset, alias)| {
            if field_offset <= offset && offset < field_end {
                Some((offset - field_offset, alias))
            } else {
                None
            }
        })
        .collect()
}

fn aggregate_field_raw_aliases_from_field_load(
    value: &HirExpr,
    ctx: &MoveCheckContext,
    tctx: &crate::types::TypeCtx,
) -> BTreeMap<usize, String> {
    let HirExprKind::Intrinsic { name, args, .. } = &value.kind else {
        return BTreeMap::new();
    };
    if name != "load" || args.len() != 1 {
        return BTreeMap::new();
    }
    let Some(path) = field_move_path_from_addr(&args[0], value.ty, tctx) else {
        return BTreeMap::new();
    };
    let field_size = storage_size_bytes(tctx, path.field_ty);
    if field_size == 0 {
        return BTreeMap::new();
    }
    let field_end = path.offset.saturating_add(field_size);
    ctx.aggregate_field_raw_aliases(path.owner.as_str())
        .into_iter()
        .filter_map(|(offset, alias)| {
            if path.offset <= offset && offset < field_end {
                Some((offset - path.offset, alias))
            } else {
                None
            }
        })
        .collect()
}

fn aggregate_field_function_aliases_from_field_load(
    value: &HirExpr,
    ctx: &MoveCheckContext,
    tctx: &crate::types::TypeCtx,
) -> BTreeMap<usize, BTreeSet<String>> {
    let HirExprKind::Intrinsic { name, args, .. } = &value.kind else {
        return BTreeMap::new();
    };
    if name != "load" || args.len() != 1 {
        return BTreeMap::new();
    }
    let Some(path) = field_move_path_from_addr(&args[0], value.ty, tctx) else {
        return BTreeMap::new();
    };
    let field_size = storage_size_bytes(tctx, path.field_ty);
    if field_size == 0 {
        return BTreeMap::new();
    }
    let field_end = path.offset.saturating_add(field_size);
    ctx.aggregate_field_function_aliases(path.owner.as_str())
        .into_iter()
        .filter_map(|(offset, aliases)| {
            if path.offset <= offset && offset < field_end {
                Some((offset - path.offset, aliases))
            } else {
                None
            }
        })
        .collect()
}

fn aggregate_field_raw_aliases_from_value(
    value: &HirExpr,
    ctx: &MoveCheckContext,
    tctx: &crate::types::TypeCtx,
) -> BTreeMap<usize, String> {
    match &value.kind {
        HirExprKind::Var(name) => ctx.aggregate_field_raw_aliases(name),
        HirExprKind::StructConstruct { fields, .. } => {
            aggregate_field_raw_aliases_from_items(value.ty, fields, ctx, tctx)
        }
        HirExprKind::TupleConstruct { items } => {
            aggregate_field_raw_aliases_from_items(value.ty, items, ctx, tctx)
        }
        HirExprKind::Call { .. } => {
            if let Some((owner, offset, field_ty)) = field_get_projection(value, ctx, tctx) {
                aggregate_field_raw_aliases_from_projection(owner, offset, field_ty, ctx, tctx)
            } else if let Some(summary) = function_call_raw_alias_summary(value, ctx, tctx) {
                summary.aggregate_field_raw_aliases
            } else {
                BTreeMap::new()
            }
        }
        HirExprKind::Intrinsic { .. } => {
            aggregate_field_raw_aliases_from_field_load(value, ctx, tctx)
        }
        _ => BTreeMap::new(),
    }
}

fn aggregate_field_function_aliases_from_items(
    value_ty: TypeId,
    items: &[HirExpr],
    ctx: &MoveCheckContext,
    tctx: &crate::types::TypeCtx,
) -> BTreeMap<usize, BTreeSet<String>> {
    let layouts = aggregate_fields_with_offsets(tctx, value_ty);
    let mut aliases = BTreeMap::new();
    for (item, layout) in items.iter().zip(layouts.into_iter()) {
        let item_aliases = function_value_aliases_from_value(item, ctx, tctx);
        if !item_aliases.is_empty() {
            aliases.insert(layout.offset, item_aliases);
        }
        for (nested_offset, nested_aliases) in
            aggregate_field_function_aliases_from_value(item, ctx, tctx)
        {
            aliases
                .entry(layout.offset.saturating_add(nested_offset))
                .or_insert_with(BTreeSet::new)
                .extend(nested_aliases);
        }
    }
    aliases
}

fn aggregate_field_function_aliases_from_projection(
    owner: &HirExpr,
    field_offset: usize,
    field_ty: TypeId,
    ctx: &MoveCheckContext,
    tctx: &crate::types::TypeCtx,
) -> BTreeMap<usize, BTreeSet<String>> {
    let field_size = storage_size_bytes(tctx, field_ty);
    if field_size == 0 {
        return BTreeMap::new();
    }
    let field_end = field_offset.saturating_add(field_size);
    aggregate_field_function_aliases_from_value(owner, ctx, tctx)
        .into_iter()
        .filter_map(|(offset, aliases)| {
            if field_offset <= offset && offset < field_end {
                Some((offset - field_offset, aliases))
            } else {
                None
            }
        })
        .collect()
}

fn function_value_aliases_from_field_projection(
    value: &HirExpr,
    ctx: &MoveCheckContext,
    tctx: &crate::types::TypeCtx,
) -> BTreeSet<String> {
    let Some((owner, offset, _field_ty)) = field_get_projection(value, ctx, tctx) else {
        return BTreeSet::new();
    };
    aggregate_field_function_aliases_from_value(owner, ctx, tctx)
        .remove(&offset)
        .unwrap_or_default()
}

fn function_value_aliases_from_field_load(
    value: &HirExpr,
    ctx: &MoveCheckContext,
    tctx: &crate::types::TypeCtx,
) -> BTreeSet<String> {
    let HirExprKind::Intrinsic { name, args, .. } = &value.kind else {
        return BTreeSet::new();
    };
    if name != "load" || args.len() != 1 {
        return BTreeSet::new();
    }
    let Some(path) = field_move_path_from_addr(&args[0], value.ty, tctx) else {
        return BTreeSet::new();
    };
    ctx.aggregate_field_function_aliases(path.owner.as_str())
        .remove(&path.offset)
        .unwrap_or_default()
}

fn aggregate_field_function_aliases_from_value(
    value: &HirExpr,
    ctx: &MoveCheckContext,
    tctx: &crate::types::TypeCtx,
) -> BTreeMap<usize, BTreeSet<String>> {
    match &value.kind {
        HirExprKind::Var(name) => ctx.aggregate_field_function_aliases(name),
        HirExprKind::StructConstruct { fields, .. } => {
            aggregate_field_function_aliases_from_items(value.ty, fields, ctx, tctx)
        }
        HirExprKind::TupleConstruct { items } => {
            aggregate_field_function_aliases_from_items(value.ty, items, ctx, tctx)
        }
        HirExprKind::Call { .. } => {
            if let Some((owner, offset, field_ty)) = field_get_projection(value, ctx, tctx) {
                aggregate_field_function_aliases_from_projection(owner, offset, field_ty, ctx, tctx)
            } else if let Some(summary) = function_call_raw_alias_summary(value, ctx, tctx) {
                summary.aggregate_field_function_aliases
            } else {
                BTreeMap::new()
            }
        }
        HirExprKind::Intrinsic { .. } => {
            aggregate_field_function_aliases_from_field_load(value, ctx, tctx)
        }
        _ => BTreeMap::new(),
    }
}

// Logic to traverse HIR
fn collect_var_uses_block(block: &HirBlock) -> BTreeMap<String, usize> {
    let mut counts = BTreeMap::new();
    let mut stack = Vec::new();
    for line in block.lines.iter().rev() {
        stack.push(&line.expr);
    }
    while let Some(expr) = stack.pop() {
        match &expr.kind {
            HirExprKind::Var(name) => {
                *counts.entry(name.clone()).or_insert(0) += 1;
            }
            HirExprKind::Call { args, .. } => {
                for arg in args.iter().rev() {
                    stack.push(arg);
                }
            }
            HirExprKind::CallIndirect { callee, args, .. } => {
                for arg in args.iter().rev() {
                    stack.push(arg);
                }
                stack.push(callee);
            }
            HirExprKind::If {
                cond,
                then_branch,
                else_branch,
            } => {
                stack.push(else_branch);
                stack.push(then_branch);
                stack.push(cond);
            }
            HirExprKind::While { cond, body } => {
                stack.push(body);
                stack.push(cond);
            }
            HirExprKind::Match { scrutinee, arms } => {
                for arm in arms.iter().rev() {
                    stack.push(&arm.body);
                }
                stack.push(scrutinee);
            }
            HirExprKind::Block(block) => {
                for line in block.lines.iter().rev() {
                    stack.push(&line.expr);
                }
            }
            HirExprKind::Let { value, .. } | HirExprKind::Set { value, .. } => {
                stack.push(value);
            }
            HirExprKind::StructConstruct { fields, .. } => {
                for field in fields.iter().rev() {
                    stack.push(field);
                }
            }
            HirExprKind::EnumConstruct { payload, .. } => {
                if let Some(payload) = payload {
                    stack.push(payload);
                }
            }
            HirExprKind::TupleConstruct { items } | HirExprKind::Intrinsic { args: items, .. } => {
                for item in items.iter().rev() {
                    stack.push(item);
                }
            }
            HirExprKind::AddrOf(inner) | HirExprKind::Deref(inner) => {
                stack.push(inner);
            }
            HirExprKind::FnValue(_)
            | HirExprKind::Unit
            | HirExprKind::LiteralI32(_)
            | HirExprKind::LiteralF32(_)
            | HirExprKind::LiteralBool(_)
            | HirExprKind::LiteralStr(_)
            | HirExprKind::Drop { .. } => {}
        }
    }
    counts
}

fn borrow_source_name(expr: &HirExpr) -> Option<String> {
    match &expr.kind {
        HirExprKind::Var(name) => Some(name.clone()),
        HirExprKind::Deref(inner) => borrow_source_name(inner),
        _ => None,
    }
}

fn borrow_binding(expr: &HirExpr, kind: BorrowKind) -> Option<BorrowBinding> {
    borrow_source_name(expr).map(|source| BorrowBinding { source, kind })
}

fn type_contains_reference(tctx: &crate::types::TypeCtx, ty: TypeId) -> bool {
    fn inner(tctx: &crate::types::TypeCtx, ty: TypeId, visiting: &mut BTreeSet<TypeId>) -> bool {
        let resolved = tctx.resolve_id(ty);
        if !visiting.insert(resolved) {
            return false;
        }
        let contains = match tctx.get_ref(resolved) {
            TypeKind::Reference(_, _) => true,
            TypeKind::Tuple { items } => items.iter().any(|item| inner(tctx, *item, visiting)),
            TypeKind::Struct { fields, .. } => {
                fields.iter().any(|field| inner(tctx, *field, visiting))
            }
            TypeKind::Enum { variants, .. } => variants
                .iter()
                .filter_map(|variant| variant.payload)
                .any(|payload| inner(tctx, payload, visiting)),
            TypeKind::Apply { base, args } => {
                inner(tctx, *base, visiting) || args.iter().any(|arg| inner(tctx, *arg, visiting))
            }
            TypeKind::Box(inner_ty) => inner(tctx, *inner_ty, visiting),
            TypeKind::Var(var) => var
                .binding
                .map(|binding| inner(tctx, binding, visiting))
                .unwrap_or(false),
            TypeKind::Unit
            | TypeKind::I32
            | TypeKind::U8
            | TypeKind::F32
            | TypeKind::Bool
            | TypeKind::Str
            | TypeKind::Never
            | TypeKind::Named(_)
            | TypeKind::Function { .. } => false,
        };
        visiting.remove(&resolved);
        contains
    }

    inner(tctx, ty, &mut BTreeSet::new())
}

fn is_never_type(tctx: &crate::types::TypeCtx, ty: TypeId) -> bool {
    matches!(tctx.get_ref(tctx.resolve_id(ty)), TypeKind::Never)
}

fn borrow_bindings_from_place(expr: &HirExpr, ctx: &MoveCheckContext) -> Vec<BorrowBinding> {
    match &expr.kind {
        HirExprKind::Var(name) => ctx.borrow_bindings(name),
        HirExprKind::Deref(inner) => borrow_bindings_from_place(inner, ctx),
        HirExprKind::Intrinsic { name, args, .. } if name == "add" && !args.is_empty() => {
            borrow_bindings_from_place(&args[0], ctx)
        }
        _ => Vec::new(),
    }
}

fn addr_of_borrow_kind(tctx: &crate::types::TypeCtx, ty: TypeId) -> BorrowKind {
    reference_borrow_kind(tctx, ty).unwrap_or(BorrowKind::Shared)
}

fn borrow_bindings_from_reference_arg(
    arg: &HirExpr,
    ctx: &MoveCheckContext,
    tctx: &crate::types::TypeCtx,
) -> Vec<BorrowBinding> {
    match &arg.kind {
        HirExprKind::AddrOf(inner) => borrow_binding(inner, addr_of_borrow_kind(tctx, arg.ty))
            .into_iter()
            .collect(),
        _ => borrow_bindings_from_place(arg, ctx),
    }
}

fn reference_source_name(expr: &HirExpr) -> Option<String> {
    match &expr.kind {
        HirExprKind::AddrOf(inner) => borrow_source_name(inner),
        HirExprKind::Deref(inner) => reference_source_name(inner),
        HirExprKind::Intrinsic { name, args, .. } if name == "add" && !args.is_empty() => {
            reference_source_name(&args[0])
        }
        HirExprKind::Var(name) => Some(name.clone()),
        _ => None,
    }
}

fn check_non_copy_deref(
    expr: &HirExpr,
    inner: &HirExpr,
    result_borrows: &[ExprBorrow],
    ctx: &mut MoveCheckContext,
    tctx: &crate::types::TypeCtx,
) {
    if tctx.is_copy(expr.ty) {
        return;
    }
    let source = result_borrows
        .first()
        .map(|borrow| borrow.binding.source.clone())
        .or_else(|| reference_source_name(inner));
    let message = if let Some(source) = source {
        alloc::format!("cannot move out of shared borrowed value: `{}`", source)
    } else {
        "cannot move non-Copy value out of shared reference".to_string()
    };
    ctx.diagnostics.push(
        Diagnostic::error(message, expr.span)
            .with_id(DiagnosticId::TypeMoveFromSharedBorrowedValue),
    );
}

fn visit_block_with_escape(
    block: &HirBlock,
    ctx: &mut MoveCheckContext,
    tctx: &crate::types::TypeCtx,
    escape_depth: Option<usize>,
) -> Vec<ExprBorrow> {
    ctx.push_scope();
    ctx.push_use_counts(collect_var_uses_block(block));
    let mut result_borrows = Vec::new();
    let last_index = block.lines.len().saturating_sub(1);
    for (idx, line) in block.lines.iter().enumerate() {
        let line_escape = if idx == last_index && !line.drop_result {
            escape_depth
        } else {
            None
        };
        let line_borrows = visit_expr_with_escape(&line.expr, ctx, tctx, line_escape);
        if idx == last_index && !line.drop_result {
            result_borrows = line_borrows;
            if let Some(depth) = escape_depth {
                ctx.check_expr_borrows_escape(&result_borrows, line.expr.span, depth);
            }
        }
    }
    ctx.pop_use_counts();
    ctx.pop_scope();
    result_borrows
}

fn reference_borrow_kind(tctx: &crate::types::TypeCtx, ty: TypeId) -> Option<BorrowKind> {
    match tctx.get_ref(tctx.resolve_id(ty)) {
        TypeKind::Reference(_, true) => Some(BorrowKind::Unique),
        TypeKind::Reference(_, false) => Some(BorrowKind::Shared),
        _ => None,
    }
}

fn visit_reference_call_arg(
    arg: &HirExpr,
    kind: BorrowKind,
    ctx: &mut MoveCheckContext,
    tctx: &crate::types::TypeCtx,
) -> Vec<ExprBorrow> {
    let arg_escape_depth = ctx.current_scope_depth();
    let result_borrows = borrow_bindings_from_reference_arg(arg, ctx, tctx)
        .into_iter()
        .map(ExprBorrow::needs_retain)
        .collect();
    match &arg.kind {
        HirExprKind::AddrOf(inner) => visit_temporary_borrow(inner, ctx, kind),
        _ if field_reference_path_from_addr(arg, tctx).is_some() => {
            if let Some(path) = field_reference_path_from_addr(arg, tctx) {
                ctx.check_field_temporary_borrow(&path, arg.span, kind);
            }
        }
        _ => {
            visit_expr_with_escape(arg, ctx, tctx, Some(arg_escape_depth));
        }
    }
    result_borrows
}

fn visit_call_args_with_params(
    args: &[HirExpr],
    params: Option<&[TypeId]>,
    ctx: &mut MoveCheckContext,
    tctx: &crate::types::TypeCtx,
) -> Vec<ExprBorrow> {
    let mut result_borrows = Vec::new();
    let mut call_borrows = Vec::new();
    for (i, arg) in args.iter().enumerate() {
        let arg_escape_depth = ctx.current_scope_depth();
        let param_ty = params.and_then(|p| p.get(i)).copied();
        let arg_borrows =
            if let Some(kind) = param_ty.and_then(|ty| reference_borrow_kind(tctx, ty)) {
                visit_reference_call_arg(arg, kind, ctx, tctx)
            } else {
                visit_expr_with_escape(arg, ctx, tctx, Some(arg_escape_depth))
            };
        call_borrows.extend(ctx.retain_expr_borrows(arg_borrows.clone()));
        result_borrows.extend(arg_borrows);
    }
    ctx.release_borrow_bindings(&call_borrows);
    result_borrows
}

fn visit_aggregate_items_with_escape(
    items: &[HirExpr],
    aggregate: &HirExpr,
    ctx: &mut MoveCheckContext,
    tctx: &crate::types::TypeCtx,
    escape_depth: Option<usize>,
) -> Vec<ExprBorrow> {
    let mut result_borrows = Vec::new();
    let mut construction_borrows = Vec::new();
    for item in items {
        let item_borrows = visit_expr_with_escape(item, ctx, tctx, escape_depth);
        construction_borrows.extend(ctx.retain_expr_borrows(item_borrows.clone()));
        result_borrows.extend(item_borrows);
    }
    if let Some(depth) = escape_depth {
        ctx.check_expr_borrows_escape(&result_borrows, aggregate.span, depth);
    }
    ctx.release_borrow_bindings(&construction_borrows);
    result_borrows
}

fn can_visit_expr_iteratively(
    expr: &HirExpr,
    ctx: &MoveCheckContext,
    tctx: &crate::types::TypeCtx,
) -> bool {
    let mut stack = Vec::new();
    stack.push(expr);
    while let Some(expr) = stack.pop() {
        match &expr.kind {
            HirExprKind::Var(_)
            | HirExprKind::FnValue(_)
            | HirExprKind::LiteralI32(_)
            | HirExprKind::LiteralF32(_)
            | HirExprKind::LiteralBool(_)
            | HirExprKind::LiteralStr(_)
            | HirExprKind::Unit
            | HirExprKind::Drop { .. } => {}
            HirExprKind::Call { callee, args } => {
                if raw_memory_call_kind(callee, args, expr.ty, tctx).is_some() {
                    return false;
                }
                match callee {
                    FuncRef::Builtin(name) | FuncRef::User(name, _, _)
                        if name == "get" || name == "if" || name == "while" =>
                    {
                        return false;
                    }
                    _ => {}
                }
                let params = match callee {
                    FuncRef::User(name, _, _) => {
                        if ctx
                            .function_raw_alias_summaries
                            .get(name)
                            .is_some_and(|summary| !summary.raw_memory_effects.is_empty())
                        {
                            return false;
                        }
                        ctx.function_params.get(name).map(Vec::as_slice)
                    }
                    _ => None,
                };
                for (i, arg) in args.iter().enumerate().rev() {
                    let param_ty = params.and_then(|p| p.get(i)).copied();
                    if param_ty
                        .and_then(|ty| reference_borrow_kind(tctx, ty))
                        .is_some()
                    {
                        return false;
                    }
                    stack.push(arg);
                }
            }
            HirExprKind::CallIndirect {
                callee,
                params,
                args,
                ..
            } => {
                if expression_function_value_aliases(callee, ctx, tctx)
                    .iter()
                    .filter_map(|callee_alias| {
                        ctx.function_raw_alias_summaries.get(callee_alias.as_str())
                    })
                    .any(|summary| !summary.raw_memory_effects.is_empty())
                {
                    return false;
                }
                for (i, arg) in args.iter().enumerate().rev() {
                    if params
                        .get(i)
                        .copied()
                        .and_then(|ty| reference_borrow_kind(tctx, ty))
                        .is_some()
                    {
                        return false;
                    }
                    stack.push(arg);
                }
                stack.push(callee);
            }
            HirExprKind::StructConstruct { fields, .. } => {
                for field in fields.iter().rev() {
                    stack.push(field);
                }
            }
            HirExprKind::EnumConstruct { payload, .. } => {
                if let Some(payload) = payload {
                    stack.push(payload);
                }
            }
            HirExprKind::TupleConstruct { items } => {
                for item in items.iter().rev() {
                    stack.push(item);
                }
            }
            HirExprKind::Intrinsic { name, args, .. } => {
                if name == "load" || name == "store" {
                    return false;
                }
                for arg in args.iter().rev() {
                    stack.push(arg);
                }
            }
            HirExprKind::If { .. }
            | HirExprKind::While { .. }
            | HirExprKind::Match { .. }
            | HirExprKind::Block(_)
            | HirExprKind::Let { .. }
            | HirExprKind::Set { .. }
            | HirExprKind::AddrOf(_)
            | HirExprKind::Deref(_) => return false,
        }
    }
    true
}

fn visit_expr_iteratively(
    expr: &HirExpr,
    ctx: &mut MoveCheckContext,
    tctx: &crate::types::TypeCtx,
) {
    let mut stack = Vec::new();
    stack.push(expr);
    while let Some(expr) = stack.pop() {
        let is_copy = tctx.is_copy(expr.ty);
        match &expr.kind {
            HirExprKind::Var(name) => {
                ctx.check_use(name, expr.span, is_copy);
                ctx.note_var_use(name);
            }
            HirExprKind::Drop { name } => ctx.check_drop(name, expr.span),
            HirExprKind::Call { args, .. } => {
                for arg in args.iter().rev() {
                    stack.push(arg);
                }
            }
            HirExprKind::CallIndirect { callee, args, .. } => {
                for arg in args.iter().rev() {
                    stack.push(arg);
                }
                stack.push(callee);
            }
            HirExprKind::StructConstruct { fields, .. } => {
                for field in fields.iter().rev() {
                    stack.push(field);
                }
            }
            HirExprKind::EnumConstruct { payload, .. } => {
                if let Some(payload) = payload {
                    stack.push(payload);
                }
            }
            HirExprKind::TupleConstruct { items } | HirExprKind::Intrinsic { args: items, .. } => {
                for item in items.iter().rev() {
                    stack.push(item);
                }
            }
            HirExprKind::FnValue(_)
            | HirExprKind::LiteralI32(_)
            | HirExprKind::LiteralF32(_)
            | HirExprKind::LiteralBool(_)
            | HirExprKind::LiteralStr(_)
            | HirExprKind::Unit => {}
            HirExprKind::If { .. }
            | HirExprKind::While { .. }
            | HirExprKind::Match { .. }
            | HirExprKind::Block(_)
            | HirExprKind::Let { .. }
            | HirExprKind::Set { .. }
            | HirExprKind::AddrOf(_)
            | HirExprKind::Deref(_) => unreachable!("iterative move check precheck failed"),
        }
    }
}

fn visit_expr(
    expr: &HirExpr,
    ctx: &mut MoveCheckContext,
    tctx: &crate::types::TypeCtx,
) -> Vec<ExprBorrow> {
    visit_expr_with_escape(expr, ctx, tctx, None)
}

fn visit_raw_memory_call(
    kind: RawMemoryCallKind,
    callee: &FuncRef,
    expr: &HirExpr,
    args: &[HirExpr],
    ctx: &mut MoveCheckContext,
    tctx: &crate::types::TypeCtx,
) {
    match kind {
        RawMemoryCallKind::Load => {
            if let Some(addr) = args.get(0) {
                visit_expr(addr, ctx, tctx);
                if let Some(path) = field_move_path_from_addr(addr, expr.ty, tctx) {
                    ctx.check_field_move(&path, expr.span);
                } else if let Some(place) = raw_memory_place_key(addr, ctx, tctx) {
                    ctx.check_raw_non_copy_load(
                        place.as_str(),
                        storage_size_bytes(tctx, expr.ty),
                        expr.span,
                    );
                }
            }
        }
        RawMemoryCallKind::Store => {
            if let Some(addr) = args.get(0) {
                visit_expr(addr, ctx, tctx);
                if let Some(value) = args.get(1) {
                    visit_expr(value, ctx, tctx);
                }
                if let Some(place) = raw_dealloc_place_key(addr, ctx, tctx) {
                    if args.get(1).is_some_and(|value| !tctx.is_copy(value.ty)) {
                        let size = args
                            .get(1)
                            .map(|value| storage_size_bytes(tctx, value.ty))
                            .unwrap_or(0);
                        ctx.check_raw_non_copy_store(place.as_str(), size, expr.span);
                    } else {
                        ctx.check_raw_non_copy_byte_write(
                            place.as_str(),
                            raw_store_write_size_bytes(callee, args.get(1), tctx),
                            expr.span,
                        );
                    }
                }
            }
        }
        RawMemoryCallKind::Dealloc => {
            for arg in args {
                visit_expr(arg, ctx, tctx);
            }
            if let Some(addr) = args.get(0) {
                if let Some(place) = raw_dealloc_place_key(addr, ctx, tctx) {
                    ctx.check_raw_non_copy_dealloc(
                        place.as_str(),
                        raw_dealloc_size_arg_bytes(args.get(1), tctx),
                        expr.span,
                    );
                }
            }
        }
        RawMemoryCallKind::Realloc => {
            for arg in args {
                visit_expr(arg, ctx, tctx);
            }
            if let Some(addr) = args.get(0) {
                if let Some(place) = raw_dealloc_place_key(addr, ctx, tctx) {
                    ctx.check_raw_non_copy_realloc(
                        place.as_str(),
                        raw_dealloc_size_arg_bytes(args.get(1), tctx),
                        expr.span,
                    );
                }
            }
        }
        RawMemoryCallKind::BulkCopy => {
            for arg in args {
                visit_expr(arg, ctx, tctx);
            }
            if let (Some(dst), Some(src)) = (args.get(0), args.get(1)) {
                if let (Some(dst_place), Some(src_place)) = (
                    raw_dealloc_place_key(dst, ctx, tctx),
                    raw_dealloc_place_key(src, ctx, tctx),
                ) {
                    ctx.check_raw_non_copy_bulk_copy(
                        dst_place.as_str(),
                        src_place.as_str(),
                        raw_bulk_copy_size_arg_bytes(args, tctx),
                        expr.span,
                    );
                }
            }
        }
        RawMemoryCallKind::ByteWrite => {
            for arg in args {
                visit_expr(arg, ctx, tctx);
            }
            if let Some(addr) = args.get(0) {
                if let Some(place) = raw_dealloc_place_key(addr, ctx, tctx) {
                    ctx.check_raw_non_copy_byte_write(
                        place.as_str(),
                        raw_byte_write_size_arg_bytes(callee, args, tctx),
                        expr.span,
                    );
                }
            }
        }
    }
}

fn apply_raw_memory_effect_summary(
    effect: &RawMemoryEffectSummary,
    span: Span,
    ctx: &mut MoveCheckContext,
) {
    match effect {
        RawMemoryEffectSummary::Load { place, size } => {
            ctx.check_raw_non_copy_load(place.as_str(), *size, span);
        }
        RawMemoryEffectSummary::Store { place, size } => {
            ctx.check_raw_non_copy_store(place.as_str(), *size, span);
        }
        RawMemoryEffectSummary::Dealloc { place, size } => {
            ctx.check_raw_non_copy_dealloc(place.as_str(), *size, span);
        }
        RawMemoryEffectSummary::Realloc { place, size } => {
            ctx.check_raw_non_copy_realloc(place.as_str(), *size, span);
        }
        RawMemoryEffectSummary::BulkCopy { dst, src, size } => {
            ctx.check_raw_non_copy_bulk_copy(dst.as_str(), src.as_str(), *size, span);
        }
        RawMemoryEffectSummary::ByteWrite { place, size } => {
            ctx.check_raw_non_copy_byte_write(place.as_str(), *size, span);
        }
        RawMemoryEffectSummary::IndirectCall { .. } => {}
    }
}

fn apply_function_raw_memory_effects(
    callee: &FuncRef,
    args: &[HirExpr],
    span: Span,
    ctx: &mut MoveCheckContext,
    tctx: &crate::types::TypeCtx,
) {
    let FuncRef::User(name, _, _) = callee else {
        return;
    };
    let Some(summary) = ctx.function_raw_alias_summaries.get(name).cloned() else {
        return;
    };
    let instantiated = instantiate_function_raw_alias_summary(&summary, args, ctx, tctx);
    for effect in &instantiated.raw_memory_effects {
        apply_raw_memory_effect_summary(effect, span, ctx);
    }
}

fn apply_indirect_function_raw_memory_effects(
    callee: &HirExpr,
    args: &[HirExpr],
    span: Span,
    ctx: &mut MoveCheckContext,
    tctx: &crate::types::TypeCtx,
) {
    let callee_aliases = expression_function_value_aliases(callee, ctx, tctx);
    if callee_aliases.is_empty() {
        return;
    }
    let arg_summaries = args
        .iter()
        .map(|arg| value_alias_summary_from_value(arg, ctx, tctx))
        .collect::<Vec<_>>();
    for callee_alias in callee_aliases {
        let effects = instantiate_known_function_raw_memory_effects(
            callee_alias.as_str(),
            &arg_summaries,
            ctx,
            tctx,
            ctx.function_raw_alias_summaries.len().saturating_add(1),
        );
        for effect in &effects {
            apply_raw_memory_effect_summary(effect, span, ctx);
        }
    }
}

fn visit_expr_with_escape(
    expr: &HirExpr,
    ctx: &mut MoveCheckContext,
    tctx: &crate::types::TypeCtx,
    escape_depth: Option<usize>,
) -> Vec<ExprBorrow> {
    if !type_contains_reference(tctx, expr.ty) && can_visit_expr_iteratively(expr, ctx, tctx) {
        visit_expr_iteratively(expr, ctx, tctx);
        return Vec::new();
    }

    let is_copy = tctx.is_copy(expr.ty);
    // ctx.diagnostics.push(Diagnostic::warning(alloc::format!("DEBUG: visiting kind {:?}", expr.kind), expr.span));

    match &expr.kind {
        HirExprKind::Var(name) => {
            let result_borrows = ctx
                .borrow_bindings(name)
                .into_iter()
                .map(ExprBorrow::needs_retain)
                .collect();
            ctx.check_use(name, expr.span, is_copy);
            if let Some(depth) = escape_depth {
                ctx.check_var_escape(name, expr.span, depth);
            }
            ctx.note_var_use(name);
            result_borrows
        }
        HirExprKind::FnValue(_) => Vec::new(),
        HirExprKind::Call { callee, args } => {
            if let Some(kind) = raw_memory_call_kind(callee, args, expr.ty, tctx) {
                visit_raw_memory_call(kind, callee, expr, args, ctx, tctx);
                return Vec::new();
            }
            match callee {
                FuncRef::Builtin(name) | FuncRef::User(name, _, _) if name == "get" => {
                    let result_borrows = if type_contains_reference(tctx, expr.ty) {
                        args.first()
                            .map(|base| {
                                borrow_bindings_from_place(base, ctx)
                                    .into_iter()
                                    .map(ExprBorrow::needs_retain)
                                    .collect()
                            })
                            .unwrap_or_default()
                    } else {
                        Vec::new()
                    };
                    if let Some(base) = args.get(0) {
                        if tctx.is_copy(expr.ty) {
                            visit_temporary_borrow(base, ctx, BorrowKind::Shared);
                        } else if !visit_field_move_source(base, expr.ty, ctx, tctx) {
                            visit_expr(base, ctx, tctx);
                        }
                    }
                    for arg in args.iter().skip(1) {
                        visit_expr(arg, ctx, tctx);
                    }
                    if let Some(depth) = escape_depth {
                        ctx.check_expr_borrows_escape(&result_borrows, expr.span, depth);
                    }
                    result_borrows
                }
                FuncRef::Builtin(name) | FuncRef::User(name, _, _) if name == "if" => {
                    if args.len() == 3 {
                        visit_expr(&args[0], ctx, tctx);

                        let saved = ctx.snapshot_resource_state();
                        let then_borrows =
                            visit_expr_with_escape(&args[1], ctx, tctx, escape_depth);
                        let then_state = ctx.snapshot_resource_state();
                        ctx.restore_resource_state(&saved);

                        let else_borrows =
                            visit_expr_with_escape(&args[2], ctx, tctx, escape_depth);
                        let else_state = ctx.snapshot_resource_state();
                        ctx.restore_resource_state(&saved);

                        let then_continues = !is_never_type(tctx, args[1].ty);
                        let else_continues = !is_never_type(tctx, args[2].ty);
                        let branches = [
                            BranchStateSnapshot {
                                continues: then_continues,
                                state: then_state,
                            },
                            BranchStateSnapshot {
                                continues: else_continues,
                                state: else_state,
                            },
                        ];
                        merge_continuing_branch_states(ctx, &saved, &branches);
                        let mut result_borrows = Vec::new();
                        if then_continues {
                            result_borrows.extend(then_borrows);
                        }
                        if else_continues {
                            result_borrows.extend(else_borrows);
                        }
                        result_borrows
                    } else {
                        Vec::new()
                    }
                }
                FuncRef::Builtin(name) | FuncRef::User(name, _, _) if name == "while" => {
                    if args.len() == 2 {
                        visit_expr(&args[0], ctx, tctx);

                        let saved = ctx.snapshot_resource_state();
                        visit_expr(&args[1], ctx, tctx);
                        let body_state = ctx.snapshot_resource_state();

                        for name in changed_state_names(&saved, &body_state) {
                            let start_state = snapshot_top_state(&saved, name.as_str())
                                .unwrap_or(VarState::Valid);
                            let end_state = snapshot_top_state(&body_state, name.as_str())
                                .unwrap_or(start_state);
                            let merged = MoveCheckContext::merge_state_pair(start_state, end_state);
                            if matches!(merged, VarState::PossiblyMoved)
                                && matches!(
                                    start_state,
                                    VarState::Valid
                                        | VarState::BorrowedShared
                                        | VarState::BorrowedUnique
                                )
                                && matches!(end_state, VarState::Moved | VarState::PossiblyMoved)
                            {
                                ctx.diagnostics.push(
                                    Diagnostic::error(
                                        alloc::format!("potentially moved value: `{}`", name),
                                        args[1].span,
                                    )
                                    .with_id(DiagnosticId::TypeLoopPotentiallyMovedValue),
                                );
                            }
                        }
                        let branches = [
                            BranchStateSnapshot {
                                continues: true,
                                state: saved.clone(),
                            },
                            BranchStateSnapshot {
                                continues: true,
                                state: body_state,
                            },
                        ];
                        merge_continuing_branch_states(ctx, &saved, &branches);
                        visit_expr(&args[0], ctx, tctx);
                    }
                    Vec::new()
                }
                _ => {
                    let params = match callee {
                        FuncRef::User(name, _, _) => ctx.function_params.get(name).cloned(),
                        _ => None,
                    };
                    let result_borrows =
                        visit_call_args_with_params(args, params.as_deref(), ctx, tctx);
                    apply_function_raw_memory_effects(callee, args, expr.span, ctx, tctx);
                    if type_contains_reference(tctx, expr.ty) {
                        if let Some(depth) = escape_depth {
                            ctx.check_expr_borrows_escape(&result_borrows, expr.span, depth);
                        }
                        result_borrows
                    } else {
                        Vec::new()
                    }
                }
            }
        }
        HirExprKind::CallIndirect {
            callee,
            params,
            args,
            ..
        } => {
            visit_expr(callee, ctx, tctx);
            let result_borrows =
                visit_call_args_with_params(args, Some(params.as_slice()), ctx, tctx);
            apply_indirect_function_raw_memory_effects(callee, args, expr.span, ctx, tctx);
            if type_contains_reference(tctx, expr.ty) {
                if let Some(depth) = escape_depth {
                    ctx.check_expr_borrows_escape(&result_borrows, expr.span, depth);
                }
                result_borrows
            } else {
                Vec::new()
            }
        }
        HirExprKind::If {
            cond,
            then_branch,
            else_branch,
        } => {
            visit_expr(cond, ctx, tctx);

            let saved = ctx.snapshot_resource_state();
            let then_borrows = visit_expr_with_escape(then_branch, ctx, tctx, escape_depth);
            let then_state = ctx.snapshot_resource_state();
            ctx.restore_resource_state(&saved);

            let else_borrows = visit_expr_with_escape(else_branch, ctx, tctx, escape_depth);
            let else_state = ctx.snapshot_resource_state();
            ctx.restore_resource_state(&saved);

            let then_continues = !is_never_type(tctx, then_branch.ty);
            let else_continues = !is_never_type(tctx, else_branch.ty);
            let branches = [
                BranchStateSnapshot {
                    continues: then_continues,
                    state: then_state,
                },
                BranchStateSnapshot {
                    continues: else_continues,
                    state: else_state,
                },
            ];
            merge_continuing_branch_states(ctx, &saved, &branches);
            let mut result_borrows = Vec::new();
            if then_continues {
                result_borrows.extend(then_borrows);
            }
            if else_continues {
                result_borrows.extend(else_borrows);
            }
            result_borrows
        }
        HirExprKind::While { cond, body } => {
            visit_expr(cond, ctx, tctx);
            let saved = ctx.snapshot_resource_state();
            visit_expr(body, ctx, tctx);
            let body_state = ctx.snapshot_resource_state();

            for name in changed_state_names(&saved, &body_state) {
                let start_state =
                    snapshot_top_state(&saved, name.as_str()).unwrap_or(VarState::Valid);
                let end_state =
                    snapshot_top_state(&body_state, name.as_str()).unwrap_or(start_state);
                let merged = MoveCheckContext::merge_state_pair(start_state, end_state);
                if matches!(merged, VarState::PossiblyMoved)
                    && matches!(
                        start_state,
                        VarState::Valid | VarState::BorrowedShared | VarState::BorrowedUnique
                    )
                    && matches!(end_state, VarState::Moved | VarState::PossiblyMoved)
                {
                    ctx.diagnostics.push(
                        Diagnostic::error(
                            alloc::format!("potentially moved value: `{}`", name),
                            expr.span,
                        )
                        .with_id(DiagnosticId::TypeLoopPotentiallyMovedValue),
                    );
                }
            }
            let branches = [
                BranchStateSnapshot {
                    continues: true,
                    state: saved.clone(),
                },
                BranchStateSnapshot {
                    continues: true,
                    state: body_state,
                },
            ];
            merge_continuing_branch_states(ctx, &saved, &branches);
            visit_expr(cond, ctx, tctx);
            Vec::new()
        }
        HirExprKind::Match { scrutinee, arms } => {
            let scrutinee_borrows = visit_expr(scrutinee, ctx, tctx);

            let mut branch_states = Vec::new();
            let mut result_borrows = Vec::new();
            let saved = ctx.snapshot_resource_state();

            for arm in arms {
                ctx.restore_resource_state(&saved);
                ctx.push_scope();
                if let Some(bind) = &arm.bind_local {
                    let retained_borrows = ctx.retain_expr_borrows(scrutinee_borrows.clone());
                    let raw_addr_alias = match_bind_raw_addr_alias(scrutinee, arm, ctx, tctx);
                    let aggregate_field_raw_aliases =
                        match_bind_aggregate_field_raw_aliases(scrutinee, arm, ctx, tctx);
                    let aggregate_field_function_aliases =
                        match_bind_aggregate_field_function_aliases(scrutinee, arm, ctx, tctx);
                    let function_value_aliases =
                        match_bind_function_value_aliases(scrutinee, arm, ctx, tctx);
                    ctx.declare_var_with_borrows(bind.clone(), retained_borrows);
                    ctx.set_raw_addr_alias(bind, raw_addr_alias);
                    ctx.set_aggregate_field_raw_aliases(bind, aggregate_field_raw_aliases);
                    ctx.set_aggregate_field_function_aliases(
                        bind,
                        aggregate_field_function_aliases,
                    );
                    ctx.set_function_value_aliases(bind, function_value_aliases);
                }
                let arm_borrows = visit_expr_with_escape(&arm.body, ctx, tctx, escape_depth);
                ctx.pop_scope();
                let arm_state = ctx.snapshot_resource_state();
                let continues = !is_never_type(tctx, arm.body.ty);
                if continues {
                    result_borrows.extend(arm_borrows);
                }
                branch_states.push(BranchStateSnapshot {
                    continues,
                    state: arm_state,
                });
            }
            ctx.restore_resource_state(&saved);

            merge_continuing_branch_states(ctx, &saved, &branch_states);
            result_borrows
        }
        HirExprKind::Block(b) => visit_block_with_escape(b, ctx, tctx, escape_depth),
        // HirExprKind::Let { name, value, .. } => {
        //     visit_expr(value, ctx, tctx);
        //     ctx.declare_var(name.clone());
        // }
        HirExprKind::Set { value, name } => {
            let target_depth = ctx
                .scope_depth_of(name)
                .unwrap_or_else(|| ctx.current_scope_depth());
            let raw_addr_alias = raw_addr_alias_from_value(value, ctx, tctx);
            let i32_const_alias = i32_const_from_value(value, ctx, tctx);
            let enum_payload_raw_aliases = enum_payload_raw_aliases_from_value(value, ctx, tctx);
            let aggregate_field_raw_aliases =
                aggregate_field_raw_aliases_from_value(value, ctx, tctx);
            let aggregate_field_function_aliases =
                aggregate_field_function_aliases_from_value(value, ctx, tctx);
            let enum_payload_aggregate_field_raw_aliases =
                enum_payload_aggregate_field_raw_aliases_from_value(value, ctx, tctx);
            let enum_payload_aggregate_field_function_aliases =
                enum_payload_aggregate_field_function_aliases_from_value(value, ctx, tctx);
            let enum_payload_function_aliases =
                enum_payload_function_aliases_from_value(value, ctx, tctx);
            let function_value_aliases = expression_function_value_aliases(value, ctx, tctx);
            let value_borrows = visit_expr_with_escape(value, ctx, tctx, Some(target_depth));
            ctx.check_assign(name, expr.span);
            let retained_borrows = ctx.retain_expr_borrows(value_borrows);
            ctx.set_borrow_bindings(name, retained_borrows);
            ctx.set_raw_addr_alias(name, raw_addr_alias);
            ctx.set_i32_const_alias(name, i32_const_alias);
            ctx.set_enum_payload_raw_aliases(name, enum_payload_raw_aliases);
            ctx.set_aggregate_field_raw_aliases(name, aggregate_field_raw_aliases);
            ctx.set_aggregate_field_function_aliases(name, aggregate_field_function_aliases);
            ctx.set_enum_payload_aggregate_field_raw_aliases(
                name,
                enum_payload_aggregate_field_raw_aliases,
            );
            ctx.set_enum_payload_aggregate_field_function_aliases(
                name,
                enum_payload_aggregate_field_function_aliases,
            );
            ctx.set_enum_payload_function_aliases(name, enum_payload_function_aliases);
            ctx.set_function_value_aliases(name, function_value_aliases);
            Vec::new()
        }
        HirExprKind::Let { name, value, .. } => {
            let storage_depth = ctx.current_scope_depth();
            let raw_addr_alias = raw_addr_alias_from_value(value, ctx, tctx);
            let i32_const_alias = i32_const_from_value(value, ctx, tctx);
            let enum_payload_raw_aliases = enum_payload_raw_aliases_from_value(value, ctx, tctx);
            let aggregate_field_raw_aliases =
                aggregate_field_raw_aliases_from_value(value, ctx, tctx);
            let aggregate_field_function_aliases =
                aggregate_field_function_aliases_from_value(value, ctx, tctx);
            let enum_payload_aggregate_field_raw_aliases =
                enum_payload_aggregate_field_raw_aliases_from_value(value, ctx, tctx);
            let enum_payload_aggregate_field_function_aliases =
                enum_payload_aggregate_field_function_aliases_from_value(value, ctx, tctx);
            let enum_payload_function_aliases =
                enum_payload_function_aliases_from_value(value, ctx, tctx);
            let function_value_aliases = expression_function_value_aliases(value, ctx, tctx);
            let value_borrows = visit_expr_with_escape(value, ctx, tctx, Some(storage_depth));
            let retained_borrows = ctx.retain_expr_borrows(value_borrows);
            ctx.declare_var_with_borrows(name.clone(), retained_borrows);
            ctx.set_raw_addr_alias(name, raw_addr_alias);
            ctx.set_i32_const_alias(name, i32_const_alias);
            ctx.set_enum_payload_raw_aliases(name, enum_payload_raw_aliases);
            ctx.set_aggregate_field_raw_aliases(name, aggregate_field_raw_aliases);
            ctx.set_aggregate_field_function_aliases(name, aggregate_field_function_aliases);
            ctx.set_enum_payload_aggregate_field_raw_aliases(
                name,
                enum_payload_aggregate_field_raw_aliases,
            );
            ctx.set_enum_payload_aggregate_field_function_aliases(
                name,
                enum_payload_aggregate_field_function_aliases,
            );
            ctx.set_enum_payload_function_aliases(name, enum_payload_function_aliases);
            ctx.set_function_value_aliases(name, function_value_aliases);
            ctx.set_state(name, VarState::Valid);
            if ctx.remaining_uses(name) == 0 {
                ctx.release_borrow_binding(name);
            }
            Vec::new()
        }
        HirExprKind::StructConstruct { fields, .. } => {
            visit_aggregate_items_with_escape(fields, expr, ctx, tctx, escape_depth)
        }
        HirExprKind::EnumConstruct { payload, .. } => {
            let mut result_borrows = Vec::new();
            if let Some(p) = payload {
                result_borrows.extend(visit_expr_with_escape(p, ctx, tctx, escape_depth));
            }
            if let Some(depth) = escape_depth {
                ctx.check_expr_borrows_escape(&result_borrows, expr.span, depth);
            }
            result_borrows
        }
        HirExprKind::TupleConstruct { items } => {
            visit_aggregate_items_with_escape(items, expr, ctx, tctx, escape_depth)
        }
        HirExprKind::Intrinsic {
            name,
            type_args,
            args,
        } => match name.as_str() {
            "load" => {
                let is_copy_load = type_args
                    .get(0)
                    .map(|ty| tctx.is_copy(*ty))
                    .unwrap_or(false);
                if let Some(addr) = args.get(0) {
                    if is_copy_load {
                        visit_temporary_borrow(addr, ctx, BorrowKind::Shared);
                    } else if !visit_field_move_source(addr, expr.ty, ctx, tctx) {
                        visit_expr(addr, ctx, tctx);
                        if let Some(place) = raw_memory_place_key(addr, ctx, tctx) {
                            ctx.check_raw_non_copy_load(
                                place.as_str(),
                                storage_size_bytes(tctx, expr.ty),
                                expr.span,
                            );
                        }
                    }
                }
                if is_copy_load && type_contains_reference(tctx, expr.ty) {
                    let result_borrows = args
                        .first()
                        .map(|addr| {
                            borrow_bindings_from_place(addr, ctx)
                                .into_iter()
                                .map(ExprBorrow::needs_retain)
                                .collect::<Vec<_>>()
                        })
                        .unwrap_or_default();
                    if let Some(depth) = escape_depth {
                        ctx.check_expr_borrows_escape(&result_borrows, expr.span, depth);
                    }
                    result_borrows
                } else {
                    Vec::new()
                }
            }
            "store" => {
                if let Some(addr) = args.get(0) {
                    visit_expr(addr, ctx, tctx);
                }
                if let Some(val) = args.get(1) {
                    visit_expr(val, ctx, tctx);
                }
                if let (Some(addr), Some(val)) = (args.get(0), args.get(1)) {
                    if let Some(place) = raw_memory_place_key(addr, ctx, tctx) {
                        if !tctx.is_copy(val.ty) {
                            ctx.check_raw_non_copy_store(
                                place.as_str(),
                                storage_size_bytes(tctx, val.ty),
                                expr.span,
                            );
                        } else {
                            ctx.check_raw_non_copy_byte_write(
                                place.as_str(),
                                Some(storage_size_bytes(tctx, val.ty)),
                                expr.span,
                            );
                        }
                    }
                }
                Vec::new()
            }
            _ => {
                let mut result_borrows = Vec::new();
                for arg in args {
                    result_borrows.extend(visit_expr(arg, ctx, tctx));
                }
                if type_contains_reference(tctx, expr.ty) {
                    if let Some(depth) = escape_depth {
                        ctx.check_expr_borrows_escape(&result_borrows, expr.span, depth);
                    }
                    result_borrows
                } else {
                    Vec::new()
                }
            }
        },
        HirExprKind::AddrOf(inner) => {
            let kind = addr_of_borrow_kind(tctx, expr.ty);
            let binding = borrow_binding(inner, kind);
            if let (Some(depth), Some(binding)) = (escape_depth, binding.as_ref()) {
                ctx.check_binding_escape(binding, expr.span, depth);
            }
            visit_temporary_borrow(inner, ctx, kind);
            binding.map(ExprBorrow::needs_retain).into_iter().collect()
        }
        HirExprKind::Deref(inner) => {
            let result_borrows = visit_expr(inner, ctx, tctx);
            check_non_copy_deref(expr, inner, &result_borrows, ctx, tctx);
            if type_contains_reference(tctx, expr.ty) {
                if let Some(depth) = escape_depth {
                    ctx.check_expr_borrows_escape(&result_borrows, expr.span, depth);
                }
                result_borrows
            } else {
                Vec::new()
            }
        }
        HirExprKind::Drop { name } => {
            ctx.check_drop(name, expr.span);
            Vec::new()
        }
        HirExprKind::LiteralI32(_)
        | HirExprKind::LiteralF32(_)
        | HirExprKind::LiteralBool(_)
        | HirExprKind::LiteralStr(_)
        | HirExprKind::Unit => Vec::new(),
    }
}

fn visit_temporary_borrow(expr: &HirExpr, ctx: &mut MoveCheckContext, kind: BorrowKind) {
    match &expr.kind {
        HirExprKind::Var(name) => {
            ctx.check_temporary_borrow(name, expr.span, kind);
        }
        HirExprKind::Deref(inner) => {
            visit_temporary_borrow(inner, ctx, kind);
        }
        HirExprKind::Intrinsic { args, .. } => {
            for arg in args {
                visit_temporary_borrow(arg, ctx, kind);
            }
        }
        _ => {}
    }
}

fn visit_field_move_source(
    expr: &HirExpr,
    field_ty: TypeId,
    ctx: &mut MoveCheckContext,
    tctx: &crate::types::TypeCtx,
) -> bool {
    if let Some(path) = field_move_path_from_addr(expr, field_ty, tctx) {
        ctx.check_field_move(&path, expr.span);
        return true;
    }
    match &expr.kind {
        HirExprKind::Var(name) => {
            if !tctx.is_copy(expr.ty) {
                ctx.check_use(name, expr.span, false);
                return true;
            }
            false
        }
        HirExprKind::Intrinsic { name, args, .. } if name == "add" && !args.is_empty() => {
            visit_field_move_source(&args[0], field_ty, ctx, tctx)
        }
        _ => false,
    }
}

impl<'m> MoveCheckContext<'m> {
    fn clone_for_alias_summary(&self) -> Self {
        Self {
            string_literals: self.string_literals.clone(),
            function_params: self.function_params.clone(),
            function_defs: self.function_defs.clone(),
            function_raw_alias_summaries: self.function_raw_alias_summaries.clone(),
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
            use_counts: Vec::new(),
            diagnostics: Vec::new(),
            scopes: self.scopes.clone(),
            raw_alias_specialization_stack: self.raw_alias_specialization_stack.clone(),
        }
    }
}

pub fn run(module: &HirModule, types: &crate::types::TypeCtx) -> Vec<Diagnostic> {
    let function_params: BTreeMap<String, Vec<TypeId>> = module
        .functions
        .iter()
        .map(|func| {
            (
                func.name.clone(),
                func.params.iter().map(|param| param.ty).collect(),
            )
        })
        .collect();
    let function_raw_alias_summaries = build_function_raw_alias_summaries(module, types);
    let mut diagnostics = Vec::new();

    for func in &module.functions {
        let mut f_ctx = MoveCheckContext::with_function_params(
            module,
            function_params.clone(),
            function_raw_alias_summaries.clone(),
        );
        for param in &func.params {
            f_ctx.declare_param(param.name.clone());
        }

        match &func.body {
            crate::hir::HirBody::Block(b) => {
                visit_block_with_escape(b, &mut f_ctx, types, Some(0));
            }
            _ => {}
        }

        diagnostics.extend(f_ctx.diagnostics);
    }

    diagnostics
}
