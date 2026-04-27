extern crate alloc;

use alloc::collections::{BTreeMap, BTreeSet};
use alloc::string::{String, ToString};
use alloc::vec::Vec;

use crate::diagnostic::Diagnostic;
use crate::diagnostic_ids::DiagnosticId;
use crate::hir::{FuncRef, HirBlock, HirExpr, HirExprKind, HirModule};
use crate::span::Span;
use crate::types::{TypeId, TypeKind};

/// Tracks ownership state of variables.
#[derive(Debug, Clone, PartialEq, Eq, Copy)]
enum VarState {
    Valid,
    BorrowedShared,
    BorrowedUnique,
    Moved,
    PossiblyMoved,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BorrowKind {
    Shared,
    Unique,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct BorrowBinding {
    source: String,
    kind: BorrowKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ExprBorrow {
    binding: BorrowBinding,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct FieldMove {
    offset: usize,
    ty: TypeId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct FieldMovePath {
    owner: String,
    offset: usize,
    field_ty: TypeId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RawPlaceState {
    Initialized,
    Moved,
    PossiblyMoved,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RawMemoryCallKind {
    Load,
    Store,
}

impl ExprBorrow {
    fn needs_retain(binding: BorrowBinding) -> Self {
        Self { binding }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
struct BorrowCount {
    shared: usize,
    unique: usize,
}

struct MoveCheckContext {
    /// Function parameter types after monomorphization.
    function_params: BTreeMap<String, Vec<TypeId>>,
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
    /// Ownership state for trackable raw memory places that carry non-Copy values.
    raw_place_states: BTreeMap<String, RawPlaceState>,
    /// Active borrow counts per source variable.
    borrow_counts: BTreeMap<String, BorrowCount>,
    /// Remaining variable uses in active blocks for last-use borrow release.
    use_counts: Vec<BTreeMap<String, usize>>,
    /// Diagnostics (errors) collected.
    diagnostics: Vec<Diagnostic>,
    /// Scopes for variable cleanup
    scopes: Vec<BTreeSet<String>>,
}

#[derive(Clone)]
struct ResourceStateSnapshot {
    var_stacks: BTreeMap<String, Vec<VarState>>,
    var_depth_stacks: BTreeMap<String, Vec<usize>>,
    borrow_stacks: BTreeMap<String, Vec<Vec<BorrowBinding>>>,
    field_move_stacks: BTreeMap<String, Vec<BTreeSet<FieldMove>>>,
    raw_addr_alias_stacks: BTreeMap<String, Vec<Option<String>>>,
    raw_place_states: BTreeMap<String, RawPlaceState>,
    borrow_counts: BTreeMap<String, BorrowCount>,
}

impl MoveCheckContext {
    fn new() -> Self {
        Self {
            function_params: BTreeMap::new(),
            var_stacks: BTreeMap::new(),
            var_depth_stacks: BTreeMap::new(),
            borrow_stacks: BTreeMap::new(),
            field_move_stacks: BTreeMap::new(),
            raw_addr_alias_stacks: BTreeMap::new(),
            raw_place_states: BTreeMap::new(),
            borrow_counts: BTreeMap::new(),
            use_counts: Vec::new(),
            diagnostics: Vec::new(),
            scopes: Vec::new(),
        }
    }

    fn snapshot_resource_state(&self) -> ResourceStateSnapshot {
        ResourceStateSnapshot {
            var_stacks: self.var_stacks.clone(),
            var_depth_stacks: self.var_depth_stacks.clone(),
            borrow_stacks: self.borrow_stacks.clone(),
            field_move_stacks: self.field_move_stacks.clone(),
            raw_addr_alias_stacks: self.raw_addr_alias_stacks.clone(),
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

    fn with_function_params(function_params: BTreeMap<String, Vec<TypeId>>) -> Self {
        let mut ctx = Self::new();
        ctx.function_params = function_params;
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

    fn check_raw_non_copy_load(&mut self, place: &str, span: Span) {
        match self.raw_place_states.get(place).copied() {
            Some(RawPlaceState::Moved) | Some(RawPlaceState::PossiblyMoved) => {
                self.diagnostics.push(
                    Diagnostic::error(
                        alloc::format!("use of moved raw memory place: `{}`", place),
                        span,
                    )
                    .with_id(DiagnosticId::TypeRawMemoryOwnershipViolation),
                );
            }
            Some(RawPlaceState::Initialized) | None => {
                self.raw_place_states
                    .insert(place.to_string(), RawPlaceState::Moved);
            }
        }
    }

    fn check_raw_non_copy_store(&mut self, place: &str, span: Span) {
        match self.raw_place_states.get(place).copied() {
            Some(RawPlaceState::Initialized) | Some(RawPlaceState::PossiblyMoved) => {
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
            }
            Some(RawPlaceState::Moved) | None => {
                self.raw_place_states
                    .insert(place.to_string(), RawPlaceState::Initialized);
            }
        }
    }
}

fn type_storage_size_bytes_mapped(
    tctx: &crate::types::TypeCtx,
    ty: TypeId,
    mapping: &BTreeMap<TypeId, TypeId>,
) -> usize {
    let resolved = tctx.resolve_id(ty);
    if let Some(mapped) = mapping.get(&resolved) {
        return type_storage_size_bytes_mapped(tctx, *mapped, mapping);
    }
    match tctx.get_ref(resolved) {
        TypeKind::Unit | TypeKind::Never => 0,
        TypeKind::U8 | TypeKind::Bool => 1,
        TypeKind::I32
        | TypeKind::F32
        | TypeKind::Str
        | TypeKind::Named(_)
        | TypeKind::Box(_)
        | TypeKind::Reference(_, _)
        | TypeKind::Function { .. } => 4,
        TypeKind::Tuple { items } => items
            .iter()
            .map(|item| type_storage_size_bytes_mapped(tctx, *item, mapping))
            .sum(),
        TypeKind::Struct { fields, .. } => fields
            .iter()
            .map(|field| type_storage_size_bytes_mapped(tctx, *field, mapping))
            .sum(),
        TypeKind::Enum { variants, .. } => {
            4 + variants
                .iter()
                .filter_map(|variant| variant.payload)
                .map(|payload| type_storage_size_bytes_mapped(tctx, payload, mapping))
                .max()
                .unwrap_or(0)
        }
        TypeKind::Apply { base, args } => {
            let mut nested = mapping.clone();
            match tctx.get_ref(tctx.resolve_id(*base)) {
                TypeKind::Struct { type_params, .. }
                | TypeKind::Enum { type_params, .. }
                | TypeKind::Function { type_params, .. } => {
                    for (param, arg) in type_params.iter().zip(args.iter()) {
                        nested.insert(tctx.resolve_id(*param), tctx.resolve_id(*arg));
                    }
                }
                _ => {}
            }
            type_storage_size_bytes_mapped(tctx, *base, &nested)
        }
        TypeKind::Var(var) => var
            .binding
            .map(|binding| type_storage_size_bytes_mapped(tctx, binding, mapping))
            .unwrap_or(4),
    }
}

fn mapped_type_id(
    tctx: &crate::types::TypeCtx,
    ty: TypeId,
    mapping: &BTreeMap<TypeId, TypeId>,
) -> TypeId {
    let resolved = tctx.resolve_id(ty);
    mapping.get(&resolved).copied().unwrap_or(resolved)
}

fn aggregate_fields_with_offsets(tctx: &crate::types::TypeCtx, ty: TypeId) -> Vec<(usize, TypeId)> {
    fn inner(
        tctx: &crate::types::TypeCtx,
        ty: TypeId,
        mapping: &BTreeMap<TypeId, TypeId>,
    ) -> Vec<(usize, TypeId)> {
        let resolved = tctx.resolve_id(ty);
        if let Some(mapped) = mapping.get(&resolved) {
            return inner(tctx, *mapped, mapping);
        }
        match tctx.get_ref(resolved) {
            TypeKind::Struct { fields, .. } => {
                let mut offset = 0;
                let mut out = Vec::with_capacity(fields.len());
                for field in fields {
                    let field_ty = mapped_type_id(tctx, *field, mapping);
                    out.push((offset, field_ty));
                    offset += type_storage_size_bytes_mapped(tctx, *field, mapping);
                }
                out
            }
            TypeKind::Tuple { items } => {
                let mut offset = 0;
                let mut out = Vec::with_capacity(items.len());
                for item in items {
                    let item_ty = mapped_type_id(tctx, *item, mapping);
                    out.push((offset, item_ty));
                    offset += type_storage_size_bytes_mapped(tctx, *item, mapping);
                }
                out
            }
            TypeKind::Apply { base, args } => {
                let mut nested = mapping.clone();
                if let TypeKind::Struct { type_params, .. } = tctx.get_ref(tctx.resolve_id(*base)) {
                    for (param, arg) in type_params.iter().zip(args.iter()) {
                        nested.insert(tctx.resolve_id(*param), tctx.resolve_id(*arg));
                    }
                }
                inner(tctx, *base, &nested)
            }
            _ => Vec::new(),
        }
    }

    inner(tctx, ty, &BTreeMap::new())
}

fn field_move_path_from_addr(
    addr: &HirExpr,
    field_ty: TypeId,
    tctx: &crate::types::TypeCtx,
) -> Option<FieldMovePath> {
    fn base_owner(expr: &HirExpr) -> Option<(&str, TypeId, usize)> {
        match &expr.kind {
            HirExprKind::Var(name) => Some((name.as_str(), expr.ty, 0)),
            HirExprKind::Intrinsic { name, args, .. } if name == "add" && args.len() >= 2 => {
                let (owner, owner_ty, base_offset) = base_owner(&args[0])?;
                let offset = match &args[1].kind {
                    HirExprKind::LiteralI32(value) if *value >= 0 => *value as usize,
                    _ => return None,
                };
                Some((owner, owner_ty, base_offset + offset))
            }
            _ => None,
        }
    }

    let (owner, owner_ty, offset) = base_owner(addr)?;
    let field_ty = tctx.resolve_id(field_ty);
    let is_declared_field = aggregate_fields_with_offsets(tctx, owner_ty)
        .into_iter()
        .any(|(field_offset, declared_ty)| {
            field_offset == offset && tctx.resolve_id(declared_ty) == field_ty
        });
    if is_declared_field {
        Some(FieldMovePath {
            owner: owner.to_string(),
            offset,
            field_ty,
        })
    } else {
        None
    }
}

fn field_reference_path_from_addr(
    addr: &HirExpr,
    tctx: &crate::types::TypeCtx,
) -> Option<FieldMovePath> {
    let field_ty = match tctx.get_ref(tctx.resolve_id(addr.ty)) {
        TypeKind::Reference(inner, _) => *inner,
        _ => return None,
    };
    field_move_path_from_addr(addr, field_ty, tctx)
}

fn format_raw_memory_place_key(base: &str, offset: i64) -> String {
    if offset == 0 {
        base.to_string()
    } else {
        alloc::format!("{}+{}", base, offset)
    }
}

fn parse_raw_memory_place_key(key: &str) -> (String, i64) {
    let Some((base, offset)) = key.rsplit_once('+') else {
        return (key.to_string(), 0);
    };
    match offset.parse::<i64>() {
        Ok(offset) => (base.to_string(), offset),
        Err(_) => (key.to_string(), 0),
    }
}

fn raw_memory_place_key(addr: &HirExpr, ctx: &MoveCheckContext) -> Option<String> {
    fn inner(expr: &HirExpr, ctx: &MoveCheckContext) -> Option<(String, i64)> {
        match &expr.kind {
            HirExprKind::Var(name) => ctx
                .raw_addr_alias(name)
                .map(parse_raw_memory_place_key)
                .or_else(|| Some((name.clone(), 0))),
            HirExprKind::LiteralI32(value) => Some((String::from("$abs"), i64::from(*value))),
            HirExprKind::Intrinsic { name, args, .. } if name == "add" && args.len() >= 2 => {
                let (base, base_offset) = inner(&args[0], ctx)?;
                let offset = match &args[1].kind {
                    HirExprKind::LiteralI32(value) if *value >= 0 => i64::from(*value),
                    _ => return None,
                };
                Some((base, base_offset + offset))
            }
            _ => None,
        }
    }

    let (base, offset) = inner(addr, ctx)?;
    Some(format_raw_memory_place_key(base.as_str(), offset))
}

fn raw_addr_alias_from_value(
    value: &HirExpr,
    ctx: &MoveCheckContext,
    tctx: &crate::types::TypeCtx,
) -> Option<String> {
    if tctx.same_type(value.ty, tctx.i32()) {
        raw_memory_place_key(value, ctx)
    } else {
        None
    }
}

fn is_raw_memory_load_name(name: &str) -> bool {
    name == "load" || name.starts_with("load_")
}

fn is_raw_memory_store_name(name: &str) -> bool {
    name == "store" || name.starts_with("store_")
}

fn raw_memory_call_kind(
    callee: &FuncRef,
    args: &[HirExpr],
    result_ty: TypeId,
    tctx: &crate::types::TypeCtx,
) -> Option<RawMemoryCallKind> {
    let FuncRef::User(name, _, _) = callee else {
        return None;
    };
    if is_raw_memory_load_name(name)
        && args.len() == 1
        && tctx.same_type(args[0].ty, tctx.i32())
        && !tctx.is_copy(result_ty)
    {
        return Some(RawMemoryCallKind::Load);
    }
    if is_raw_memory_store_name(name)
        && args.len() >= 2
        && tctx.same_type(args[0].ty, tctx.i32())
        && !tctx.is_copy(args[1].ty)
    {
        return Some(RawMemoryCallKind::Store);
    }
    None
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
                    FuncRef::User(name, _, _) => ctx.function_params.get(name).map(Vec::as_slice),
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

struct BranchStateSnapshot {
    continues: bool,
    state: ResourceStateSnapshot,
}

fn snapshot_top_state(snapshot: &ResourceStateSnapshot, name: &str) -> Option<VarState> {
    snapshot
        .var_stacks
        .get(name)
        .and_then(|stack| stack.last().copied())
}

fn changed_state_names(
    start: &ResourceStateSnapshot,
    end: &ResourceStateSnapshot,
) -> BTreeSet<String> {
    let mut names = BTreeSet::new();
    for name in start.var_stacks.keys() {
        if snapshot_top_state(start, name) != snapshot_top_state(end, name) {
            names.insert(name.clone());
        }
    }
    for name in end.var_stacks.keys() {
        if snapshot_top_state(start, name) != snapshot_top_state(end, name) {
            names.insert(name.clone());
        }
    }
    for name in start.field_move_stacks.keys() {
        if start.field_move_stacks.get(name) != end.field_move_stacks.get(name) {
            names.insert(name.clone());
        }
    }
    for name in end.field_move_stacks.keys() {
        if start.field_move_stacks.get(name) != end.field_move_stacks.get(name) {
            names.insert(name.clone());
        }
    }
    names
}

fn push_unique_binding(out: &mut Vec<BorrowBinding>, binding: &BorrowBinding) {
    if !out.contains(binding) {
        out.push(binding.clone());
    }
}

fn merge_raw_place_state_pair(
    a: Option<RawPlaceState>,
    b: Option<RawPlaceState>,
) -> Option<RawPlaceState> {
    use RawPlaceState::*;
    match (a, b) {
        (None, None) => None,
        (Some(left), Some(right)) if left == right => Some(left),
        (Some(_), Some(_)) => Some(PossiblyMoved),
        (Some(Initialized), None) | (None, Some(Initialized)) => Some(PossiblyMoved),
        (Some(Moved), None) | (None, Some(Moved)) => Some(PossiblyMoved),
        (Some(PossiblyMoved), None) | (None, Some(PossiblyMoved)) => Some(PossiblyMoved),
    }
}

fn merge_raw_place_states(branches: &[&BranchStateSnapshot]) -> BTreeMap<String, RawPlaceState> {
    let mut names = BTreeSet::new();
    for branch in branches {
        for name in branch.state.raw_place_states.keys() {
            names.insert(name.clone());
        }
    }

    let mut merged = BTreeMap::new();
    for name in names {
        let mut state = None;
        for branch in branches {
            let branch_state = branch.state.raw_place_states.get(name.as_str()).copied();
            state = merge_raw_place_state_pair(state, branch_state);
        }
        if let Some(state) = state {
            merged.insert(name, state);
        }
    }
    merged
}

fn merge_raw_addr_alias_stacks(
    branches: &[&BranchStateSnapshot],
) -> BTreeMap<String, Vec<Option<String>>> {
    let mut names = BTreeSet::new();
    for branch in branches {
        for name in branch.state.raw_addr_alias_stacks.keys() {
            names.insert(name.clone());
        }
    }

    let mut merged = BTreeMap::new();
    for name in names {
        let max_len = branches
            .iter()
            .filter_map(|branch| branch.state.raw_addr_alias_stacks.get(name.as_str()))
            .map(Vec::len)
            .max()
            .unwrap_or(0);
        let mut stack = Vec::with_capacity(max_len);
        for index in 0..max_len {
            let mut branch_values = branches.iter().map(|branch| {
                branch
                    .state
                    .raw_addr_alias_stacks
                    .get(name.as_str())
                    .and_then(|stack| stack.get(index))
                    .cloned()
                    .unwrap_or(None)
            });
            let first = branch_values.next().unwrap_or(None);
            if branch_values.all(|alias| alias == first) {
                stack.push(first);
            } else {
                stack.push(None);
            }
        }
        if !stack.is_empty() {
            merged.insert(name, stack);
        }
    }
    merged
}

fn merged_branch_borrow_stack(
    name: &str,
    active_len: usize,
    saved: &ResourceStateSnapshot,
    branches: &[&BranchStateSnapshot],
) -> Vec<Vec<BorrowBinding>> {
    let saved_stack = saved.borrow_stacks.get(name);
    let mut merged = Vec::with_capacity(active_len);
    for index in 0..active_len {
        let mut bindings = Vec::new();
        for branch in branches {
            let branch_bindings = branch
                .state
                .borrow_stacks
                .get(name)
                .and_then(|stack| stack.get(index))
                .or_else(|| saved_stack.and_then(|stack| stack.get(index)));
            if let Some(branch_bindings) = branch_bindings {
                for binding in branch_bindings {
                    push_unique_binding(&mut bindings, binding);
                }
            }
        }
        merged.push(bindings);
    }
    merged
}

fn snapshot_top_field_moves(snapshot: &ResourceStateSnapshot, name: &str) -> BTreeSet<FieldMove> {
    snapshot
        .field_move_stacks
        .get(name)
        .and_then(|stack| stack.last())
        .cloned()
        .unwrap_or_default()
}

fn merge_continuing_branch_states(
    ctx: &mut MoveCheckContext,
    saved: &ResourceStateSnapshot,
    branches: &[BranchStateSnapshot],
) {
    let continuing: Vec<&BranchStateSnapshot> =
        branches.iter().filter(|branch| branch.continues).collect();
    if continuing.is_empty() {
        ctx.restore_resource_state(saved);
        return;
    }
    let merged_raw_place_states = merge_raw_place_states(&continuing);
    let merged_raw_addr_alias_stacks = merge_raw_addr_alias_stacks(&continuing);

    ctx.restore_resource_state(saved);

    let mut names = BTreeSet::new();
    for name in saved.var_stacks.keys() {
        names.insert(name.clone());
    }
    for branch in &continuing {
        for name in branch.state.var_stacks.keys() {
            names.insert(name.clone());
        }
    }

    for name in &names {
        let mut states = Vec::new();
        for branch in &continuing {
            let state = snapshot_top_state(&branch.state, name)
                .or_else(|| snapshot_top_state(saved, name))
                .unwrap_or(VarState::Valid);
            states.push(state);
        }
        if states.is_empty() {
            continue;
        }
        let merged = MoveCheckContext::merge_states(&states);
        ctx.set_state(name.as_str(), merged);

        let mut field_moves: Option<BTreeSet<FieldMove>> = None;
        let mut field_moves_match = true;
        for branch in &continuing {
            let branch_moves = snapshot_top_field_moves(&branch.state, name.as_str());
            match &field_moves {
                Some(existing) if *existing != branch_moves => {
                    field_moves_match = false;
                    break;
                }
                Some(_) => {}
                None => field_moves = Some(branch_moves),
            }
        }
        if field_moves_match {
            ctx.set_field_moves(name.as_str(), field_moves.unwrap_or_default());
        } else {
            ctx.clear_field_moves(name.as_str());
            ctx.set_state(name.as_str(), VarState::PossiblyMoved);
        }
    }

    let active_names: Vec<(String, usize)> = ctx
        .var_stacks
        .iter()
        .map(|(name, stack)| (name.clone(), stack.len()))
        .collect();
    ctx.borrow_stacks.clear();
    for (name, active_len) in active_names {
        let merged_stack =
            merged_branch_borrow_stack(name.as_str(), active_len, saved, &continuing);
        ctx.borrow_stacks.insert(name, merged_stack);
    }
    ctx.raw_addr_alias_stacks = merged_raw_addr_alias_stacks;
    ctx.raw_place_states = merged_raw_place_states;
    ctx.rebuild_borrow_counts_from_bindings();
    ctx.release_dead_borrows();
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
                } else if let Some(place) = raw_memory_place_key(addr, ctx) {
                    ctx.check_raw_non_copy_load(place.as_str(), expr.span);
                }
            }
        }
        RawMemoryCallKind::Store => {
            if let Some(addr) = args.get(0) {
                visit_expr(addr, ctx, tctx);
                if let Some(value) = args.get(1) {
                    visit_expr(value, ctx, tctx);
                }
                if let Some(place) = raw_memory_place_key(addr, ctx) {
                    ctx.check_raw_non_copy_store(place.as_str(), expr.span);
                }
            }
        }
    }
}

fn visit_expr_with_escape(
    expr: &HirExpr,
    ctx: &mut MoveCheckContext,
    tctx: &crate::types::TypeCtx,
    escape_depth: Option<usize>,
) -> Vec<ExprBorrow> {
    if escape_depth.is_none()
        && !type_contains_reference(tctx, expr.ty)
        && can_visit_expr_iteratively(expr, ctx, tctx)
    {
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
                visit_raw_memory_call(kind, expr, args, ctx, tctx);
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
                    ctx.declare_var_with_borrows(bind.clone(), retained_borrows);
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
            let value_borrows = visit_expr_with_escape(value, ctx, tctx, Some(target_depth));
            ctx.check_assign(name, expr.span);
            let retained_borrows = ctx.retain_expr_borrows(value_borrows);
            ctx.set_borrow_bindings(name, retained_borrows);
            ctx.set_raw_addr_alias(name, raw_addr_alias);
            Vec::new()
        }
        HirExprKind::Let { name, value, .. } => {
            let storage_depth = ctx.current_scope_depth();
            let raw_addr_alias = raw_addr_alias_from_value(value, ctx, tctx);
            let value_borrows = visit_expr_with_escape(value, ctx, tctx, Some(storage_depth));
            let retained_borrows = ctx.retain_expr_borrows(value_borrows);
            ctx.declare_var_with_borrows(name.clone(), retained_borrows);
            ctx.set_raw_addr_alias(name, raw_addr_alias);
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
                        if let Some(place) = raw_memory_place_key(addr, ctx) {
                            ctx.check_raw_non_copy_load(place.as_str(), expr.span);
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
                    if !tctx.is_copy(val.ty) {
                        if let Some(place) = raw_memory_place_key(addr, ctx) {
                            ctx.check_raw_non_copy_store(place.as_str(), expr.span);
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
    let mut diagnostics = Vec::new();

    for func in &module.functions {
        let mut f_ctx = MoveCheckContext::with_function_params(function_params.clone());
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
