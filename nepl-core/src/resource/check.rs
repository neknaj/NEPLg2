extern crate alloc;

use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

use crate::hir::HirModule;
use crate::span::Span;
use crate::types::{TypeCtx, TypeId};

use super::coverage::{compare_hir_resource_lowering, ResourceLoweringCoverage};
use super::effect::{check_resource_effect_boundaries, ResourceEffectBoundaryReport};
use super::lower::lower_hir_module_skeleton;
use super::model::{
    AggregateKind, BorrowKind, BorrowState, BorrowStateEntry, CellState, CellStateEntry,
    OwnerState, OwnerStateEntry, Place, PlaceProjection, PlaceRoot, RawMemoryOp, ResourceBlock,
    ResourceCallTarget, ResourceExprKind, ResourceFunction, ResourceModule, ResourceOp,
    ResourceTerminator, StorageId,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceSafetyShadowReport {
    pub lowering_coverage: ResourceLoweringCoverage,
    pub initialized_moves: ResourceCheckReport,
    pub owner_obligations: ResourceOwnerCheckReport,
    pub borrow_lifetimes: ResourceBorrowCheckReport,
    pub effect_boundaries: ResourceEffectBoundaryReport,
}

impl ResourceSafetyShadowReport {
    pub fn lowering_diagnostic_count(&self) -> usize {
        self.lowering_coverage.diagnostics.len()
    }

    pub fn resource_diagnostic_count(&self) -> usize {
        self.initialized_moves.diagnostics.len()
            + self.owner_obligations.diagnostics.len()
            + self.borrow_lifetimes.diagnostics.len()
            + self.effect_boundaries.diagnostics.len()
    }

    pub fn has_lowering_diagnostics(&self) -> bool {
        !self.lowering_coverage.diagnostics.is_empty()
    }

    pub fn has_resource_diagnostics(&self) -> bool {
        !self.initialized_moves.diagnostics.is_empty()
            || !self.owner_obligations.diagnostics.is_empty()
            || !self.borrow_lifetimes.diagnostics.is_empty()
            || !self.effect_boundaries.diagnostics.is_empty()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceCheckReport {
    pub functions: Vec<ResourceFunctionCheck>,
    pub diagnostics: Vec<ResourceCheckDiagnostic>,
    pub deferred: ResourceCheckDeferred,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceFunctionCheck {
    pub name: String,
    pub final_cells: Vec<CellStateEntry>,
    pub deferred: ResourceCheckDeferred,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ResourceCheckDeferred {
    pub branch_merges: usize,
    pub loop_merges: usize,
    pub match_merges: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResourceCheckDiagnostic {
    CellUnavailable {
        function: String,
        operation: ResourceCheckOperation,
        place: Place,
        state: CellState,
        span: Span,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResourceCheckOperation {
    Read,
    Move,
    Drop,
    Borrow,
    DeclareInitializer,
    AssignValue,
    CallArgument,
    ConstructInput,
    ReturnValue,
    BranchCondition,
    BranchValue,
    LoopCondition,
    MatchScrutinee,
    MatchValue,
    RawMemoryArgument,
    IndirectCallee,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceOwnerCheckReport {
    pub functions: Vec<ResourceOwnerFunctionCheck>,
    pub diagnostics: Vec<ResourceOwnerDiagnostic>,
    pub deferred: ResourceOwnerCheckDeferred,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceOwnerFunctionCheck {
    pub name: String,
    pub final_owners: Vec<OwnerStateEntry>,
    pub deferred: ResourceOwnerCheckDeferred,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ResourceOwnerCheckDeferred {
    pub branch_merges: usize,
    pub loop_merges: usize,
    pub match_merges: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResourceOwnerDiagnostic {
    OwnerUnavailable {
        function: String,
        operation: ResourceOwnerOperation,
        place: Place,
        state: OwnerState,
        span: Span,
    },
    OwnerLeaked {
        function: String,
        place: Place,
        storage: StorageId,
        span: Span,
    },
    OwnerMaybeLeaked {
        function: String,
        place: Place,
        span: Span,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResourceOwnerOperation {
    DeclareInitializer,
    Read,
    Move,
    AssignValue,
    ReturnValue,
    Dealloc,
    ReallocInput,
    BranchValue,
    MatchValue,
    ConstructInput,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceBorrowCheckReport {
    pub functions: Vec<ResourceBorrowFunctionCheck>,
    pub diagnostics: Vec<ResourceBorrowDiagnostic>,
    pub deferred: ResourceBorrowCheckDeferred,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceBorrowFunctionCheck {
    pub name: String,
    pub final_borrows: Vec<BorrowStateEntry>,
    pub deferred: ResourceBorrowCheckDeferred,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ResourceBorrowCheckDeferred {
    pub branch_merges: usize,
    pub loop_merges: usize,
    pub match_merges: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResourceBorrowDiagnostic {
    BorrowConflict {
        function: String,
        operation: ResourceBorrowOperation,
        place: Place,
        active: BorrowState,
        span: Span,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResourceBorrowOperation {
    SharedBorrow,
    UniqueBorrow,
    Read,
    Move,
    Assign,
    Drop,
    ReturnValue,
}

pub fn check_resource_initialized_moves(
    module: &ResourceModule,
    types: &TypeCtx,
) -> ResourceCheckReport {
    let mut functions = Vec::new();
    let mut diagnostics = Vec::new();
    let mut deferred = ResourceCheckDeferred::default();

    for function in &module.functions {
        let mut engine = ResourceCheckEngine {
            function: function.name.as_str(),
            types,
            diagnostics: Vec::new(),
            deferred: ResourceCheckDeferred::default(),
        };
        let final_cells = engine.check_function(function);
        merge_deferred(&mut deferred, engine.deferred);
        diagnostics.extend(engine.diagnostics);
        functions.push(ResourceFunctionCheck {
            name: function.name.clone(),
            final_cells,
            deferred: engine.deferred,
        });
    }

    ResourceCheckReport {
        functions,
        diagnostics,
        deferred,
    }
}

pub fn check_resource_owner_obligations(module: &ResourceModule) -> ResourceOwnerCheckReport {
    let mut functions = Vec::new();
    let mut diagnostics = Vec::new();
    let mut deferred = ResourceOwnerCheckDeferred::default();
    let summaries = compute_owner_return_summaries(module);

    for function in &module.functions {
        let mut engine = ResourceOwnerCheckEngine {
            function: function.name.as_str(),
            summaries: &summaries,
            diagnostics: Vec::new(),
            deferred: ResourceOwnerCheckDeferred::default(),
        };
        let final_owners = engine.check_function(function);
        merge_owner_deferred(&mut deferred, engine.deferred);
        diagnostics.extend(engine.diagnostics);
        functions.push(ResourceOwnerFunctionCheck {
            name: function.name.clone(),
            final_owners,
            deferred: engine.deferred,
        });
    }

    ResourceOwnerCheckReport {
        functions,
        diagnostics,
        deferred,
    }
}

pub fn check_resource_borrow_lifetimes(module: &ResourceModule) -> ResourceBorrowCheckReport {
    let mut functions = Vec::new();
    let mut diagnostics = Vec::new();
    let mut deferred = ResourceBorrowCheckDeferred::default();
    let summaries = compute_borrow_token_return_summaries(module);

    for function in &module.functions {
        let mut engine = ResourceBorrowCheckEngine {
            function: function.name.as_str(),
            summaries: &summaries,
            diagnostics: Vec::new(),
            deferred: ResourceBorrowCheckDeferred::default(),
        };
        let final_borrows = engine.check_function(function);
        merge_borrow_deferred(&mut deferred, engine.deferred);
        diagnostics.extend(engine.diagnostics);
        functions.push(ResourceBorrowFunctionCheck {
            name: function.name.clone(),
            final_borrows,
            deferred: engine.deferred,
        });
    }

    ResourceBorrowCheckReport {
        functions,
        diagnostics,
        deferred,
    }
}

pub fn check_hir_resource_safety_shadow(
    module: &HirModule,
    types: &TypeCtx,
) -> ResourceSafetyShadowReport {
    let resource = lower_hir_module_skeleton(module);
    ResourceSafetyShadowReport {
        lowering_coverage: compare_hir_resource_lowering(module, &resource),
        initialized_moves: check_resource_initialized_moves(&resource, types),
        owner_obligations: check_resource_owner_obligations(&resource),
        borrow_lifetimes: check_resource_borrow_lifetimes(&resource),
        effect_boundaries: check_resource_effect_boundaries(&resource),
    }
}

struct ResourceCheckEngine<'a> {
    function: &'a str,
    types: &'a TypeCtx,
    diagnostics: Vec<ResourceCheckDiagnostic>,
    deferred: ResourceCheckDeferred,
}

struct ResourceOwnerCheckEngine<'a> {
    function: &'a str,
    summaries: &'a [OwnerReturnSummary],
    diagnostics: Vec<ResourceOwnerDiagnostic>,
    deferred: ResourceOwnerCheckDeferred,
}

struct ResourceBorrowCheckEngine<'a> {
    function: &'a str,
    summaries: &'a [BorrowTokenReturnSummary],
    diagnostics: Vec<ResourceBorrowDiagnostic>,
    deferred: ResourceBorrowCheckDeferred,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct BorrowTokenReturnSummary {
    function: String,
    parameter_indices: Vec<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct OwnerReturnSummary {
    function: String,
    parameter_indices: Vec<usize>,
    returns_fresh_owner: bool,
    projection_returns: Vec<OwnerProjectionReturnSummary>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct OwnerProjectionReturnSummary {
    suffix: Vec<PlaceProjection>,
    ty: TypeId,
    parameter_indices: Vec<usize>,
    returns_fresh_owner: bool,
}

impl ResourceCheckEngine<'_> {
    fn check_function(&mut self, function: &ResourceFunction) -> Vec<CellStateEntry> {
        let mut cells = CellTable::default();
        for param in &function.params {
            cells.mark_initialized(&param.place);
        }
        for block in &function.blocks {
            self.check_block(&mut cells, block);
        }
        cells.into_entries()
    }

    fn check_block(&mut self, cells: &mut CellTable, block: &ResourceBlock) {
        self.check_ops(cells, &block.ops);
        match &block.terminator {
            ResourceTerminator::Return { value, span } => {
                if let Some(value) = value {
                    self.consume_by_value(cells, value, ResourceCheckOperation::ReturnValue, *span);
                }
            }
            ResourceTerminator::Unreachable { .. } | ResourceTerminator::RawBody { .. } => {}
        }
    }

    fn check_ops(&mut self, cells: &mut CellTable, ops: &[ResourceOp]) {
        for op in ops {
            self.check_op(cells, op);
        }
    }

    fn check_op(&mut self, cells: &mut CellTable, op: &ResourceOp) {
        match op {
            ResourceOp::Expr {
                kind,
                output,
                span: _,
                ..
            } => self.check_expr(cells, *kind, output),
            ResourceOp::DeclareLocal {
                place,
                initializer,
                span,
                ..
            } => {
                if let Some(initializer) = initializer {
                    if self.consume_by_value(
                        cells,
                        initializer,
                        ResourceCheckOperation::DeclareInitializer,
                        *span,
                    ) {
                        cells.mark_initialized(place);
                    } else {
                        cells.set_state(place, CellState::Uninit);
                    }
                } else {
                    cells.set_state(place, CellState::Uninit);
                }
            }
            ResourceOp::Read {
                source,
                output,
                span,
            } => {
                if self.consume_by_value(cells, source, ResourceCheckOperation::Read, *span) {
                    cells.mark_initialized(output);
                }
            }
            ResourceOp::Assign {
                target,
                value,
                span,
            } => {
                if self.consume_by_value(cells, value, ResourceCheckOperation::AssignValue, *span) {
                    cells.mark_initialized(target);
                }
            }
            ResourceOp::Borrow {
                source,
                output,
                span,
                ..
            } => {
                if self.ensure_available(cells, source, ResourceCheckOperation::Borrow, *span) {
                    cells.mark_initialized(output);
                }
            }
            ResourceOp::Move {
                source,
                output,
                span,
            } => {
                if self.ensure_available(cells, source, ResourceCheckOperation::Move, *span) {
                    cells.set_state(source, CellState::Moved);
                    cells.mark_initialized(output);
                }
            }
            ResourceOp::Drop { place, span } => {
                if self.ensure_available(cells, place, ResourceCheckOperation::Drop, *span) {
                    cells.set_state(place, CellState::Dropped);
                }
            }
            ResourceOp::CallEffect { .. } => {}
            ResourceOp::FunctionValue { output, .. } => {
                cells.mark_initialized(output);
            }
            ResourceOp::Call {
                output, args, span, ..
            } => {
                let args_available =
                    self.consume_args(cells, args, ResourceCheckOperation::CallArgument, *span);
                if args_available {
                    cells.mark_initialized(output);
                }
            }
            ResourceOp::IndirectCall {
                output,
                callee,
                args,
                span,
                ..
            } => {
                let callee_available = self.ensure_available(
                    cells,
                    callee,
                    ResourceCheckOperation::IndirectCallee,
                    *span,
                );
                let args_available =
                    self.consume_args(cells, args, ResourceCheckOperation::CallArgument, *span);
                if callee_available && args_available {
                    cells.mark_initialized(output);
                }
            }
            ResourceOp::RawMemory {
                output, args, span, ..
            } => {
                let args_available = self.ensure_args(
                    cells,
                    args,
                    ResourceCheckOperation::RawMemoryArgument,
                    *span,
                );
                if args_available {
                    cells.mark_initialized(output);
                }
            }
            ResourceOp::Construct {
                output,
                inputs,
                span,
                ..
            } => {
                let inputs_available =
                    self.consume_args(cells, inputs, ResourceCheckOperation::ConstructInput, *span);
                if inputs_available {
                    cells.mark_initialized(output);
                }
            }
            ResourceOp::Branch {
                output,
                condition,
                then_ops,
                then_value,
                else_ops,
                else_value,
                span,
            } => {
                let condition_available = self.consume_by_value(
                    cells,
                    condition,
                    ResourceCheckOperation::BranchCondition,
                    *span,
                );
                let mut then_cells = cells.clone();
                let mut else_cells = cells.clone();
                self.check_ops(&mut then_cells, then_ops);
                self.check_ops(&mut else_cells, else_ops);
                let then_available = self.consume_by_value(
                    &mut then_cells,
                    then_value,
                    ResourceCheckOperation::BranchValue,
                    *span,
                );
                let else_available = self.consume_by_value(
                    &mut else_cells,
                    else_value,
                    ResourceCheckOperation::BranchValue,
                    *span,
                );
                *cells = CellTable::merge_paths(&[then_cells, else_cells]);
                if condition_available && then_available && else_available {
                    cells.mark_initialized(output);
                }
            }
            ResourceOp::Loop {
                condition_ops,
                condition,
                body_ops,
                span,
            } => {
                let mut condition_cells = cells.clone();
                self.check_ops(&mut condition_cells, condition_ops);
                self.consume_by_value(
                    &mut condition_cells,
                    condition,
                    ResourceCheckOperation::LoopCondition,
                    *span,
                );
                let mut body_cells = condition_cells.clone();
                self.check_ops(&mut body_cells, body_ops);
                *cells = CellTable::merge_paths(&[condition_cells, body_cells]);
            }
            ResourceOp::Match {
                output,
                scrutinee,
                arms,
                span,
            } => {
                let scrutinee_available = self.consume_by_value(
                    cells,
                    scrutinee,
                    ResourceCheckOperation::MatchScrutinee,
                    *span,
                );
                let mut arms_available = true;
                let mut arm_paths = Vec::new();
                for arm in arms {
                    let mut arm_cells = cells.clone();
                    if let Some(bind_local) = &arm.bind_local {
                        arm_cells.mark_initialized(bind_local);
                    }
                    self.check_ops(&mut arm_cells, &arm.ops);
                    arms_available &= self.consume_by_value(
                        &mut arm_cells,
                        &arm.value,
                        ResourceCheckOperation::MatchValue,
                        arm.span,
                    );
                    arm_paths.push(arm_cells);
                }
                if !arm_paths.is_empty() {
                    *cells = CellTable::merge_paths(&arm_paths);
                }
                if scrutinee_available && arms_available {
                    cells.mark_initialized(output);
                }
            }
        }
    }

    fn check_expr(&mut self, cells: &mut CellTable, kind: ResourceExprKind, output: &Place) {
        match kind {
            ResourceExprKind::Literal
            | ResourceExprKind::Block
            | ResourceExprKind::Let
            | ResourceExprKind::Set
            | ResourceExprKind::Intrinsic
            | ResourceExprKind::Deref
            | ResourceExprKind::Drop => cells.mark_initialized(output),
            ResourceExprKind::LocalRead
            | ResourceExprKind::FunctionValue
            | ResourceExprKind::Call
            | ResourceExprKind::IndirectCall
            | ResourceExprKind::Branch
            | ResourceExprKind::Loop
            | ResourceExprKind::Match
            | ResourceExprKind::Construct
            | ResourceExprKind::Borrow => {}
        }
    }

    fn ensure_args(
        &mut self,
        cells: &mut CellTable,
        args: &[Place],
        operation: ResourceCheckOperation,
        span: Span,
    ) -> bool {
        let mut available = true;
        for arg in args {
            available &= self.ensure_available(cells, arg, operation, span);
        }
        available
    }

    fn consume_args(
        &mut self,
        cells: &mut CellTable,
        args: &[Place],
        operation: ResourceCheckOperation,
        span: Span,
    ) -> bool {
        let mut available = true;
        for arg in args {
            available &= self.consume_by_value(cells, arg, operation, span);
        }
        available
    }

    fn consume_by_value(
        &mut self,
        cells: &mut CellTable,
        place: &Place,
        operation: ResourceCheckOperation,
        span: Span,
    ) -> bool {
        if !self.ensure_available(cells, place, operation, span) {
            return false;
        }
        if should_track(place) && !self.types.is_copy(place.ty) {
            cells.set_state(place, CellState::Moved);
        }
        true
    }

    fn ensure_available(
        &mut self,
        cells: &CellTable,
        place: &Place,
        operation: ResourceCheckOperation,
        span: Span,
    ) -> bool {
        if !should_track(place) {
            return true;
        }
        match cells.state(place) {
            Some(CellState::Initialized(_)) => true,
            Some(state) => {
                self.push_unavailable(operation, place, state, span);
                false
            }
            None => {
                self.push_unavailable(operation, place, CellState::Uninit, span);
                false
            }
        }
    }

    fn push_unavailable(
        &mut self,
        operation: ResourceCheckOperation,
        place: &Place,
        state: CellState,
        span: Span,
    ) {
        self.diagnostics
            .push(ResourceCheckDiagnostic::CellUnavailable {
                function: String::from(self.function),
                operation,
                place: place.clone(),
                state,
                span,
            });
    }
}

impl ResourceBorrowCheckEngine<'_> {
    fn check_function(&mut self, function: &ResourceFunction) -> Vec<BorrowStateEntry> {
        let mut borrows = BorrowTable::default();
        let mut function_aliases = FunctionAliasTable::default();
        for block in &function.blocks {
            self.check_block(&mut borrows, &mut function_aliases, block);
        }
        borrows.into_entries()
    }

    fn check_block(
        &mut self,
        borrows: &mut BorrowTable,
        function_aliases: &mut FunctionAliasTable,
        block: &ResourceBlock,
    ) {
        self.check_ops(borrows, function_aliases, &block.ops);
        match &block.terminator {
            ResourceTerminator::Return { value, span } => {
                if let Some(value) = value {
                    self.check_return_escape(borrows, value, *span);
                }
            }
            ResourceTerminator::Unreachable { .. } | ResourceTerminator::RawBody { .. } => {}
        }
    }

    fn check_ops(
        &mut self,
        borrows: &mut BorrowTable,
        function_aliases: &mut FunctionAliasTable,
        ops: &[ResourceOp],
    ) {
        for op in ops {
            self.check_op(borrows, function_aliases, op);
        }
    }

    fn check_op(
        &mut self,
        borrows: &mut BorrowTable,
        function_aliases: &mut FunctionAliasTable,
        op: &ResourceOp,
    ) {
        match op {
            ResourceOp::DeclareLocal {
                place, initializer, ..
            } => {
                if let Some(initializer) = initializer {
                    borrows.transfer_token(initializer, place);
                    function_aliases.copy_alias(initializer, place);
                }
            }
            ResourceOp::Read {
                source,
                output,
                span,
            } => {
                if !borrows.copy_or_move_token(source, output) {
                    self.check_source_read(borrows, source, *span);
                }
                function_aliases.copy_alias(source, output);
            }
            ResourceOp::Assign {
                target,
                value,
                span,
            } => {
                self.check_source_exclusive(
                    borrows,
                    target,
                    ResourceBorrowOperation::Assign,
                    *span,
                );
                borrows.transfer_token(value, target);
                function_aliases.copy_alias(value, target);
            }
            ResourceOp::Borrow {
                source,
                output,
                kind,
                span,
            } => self.start_borrow(borrows, source, output, *kind, *span),
            ResourceOp::Move {
                source,
                output,
                span,
            } => {
                if !borrows.transfer_token(source, output) {
                    self.check_source_exclusive(
                        borrows,
                        source,
                        ResourceBorrowOperation::Move,
                        *span,
                    );
                }
                function_aliases.copy_alias(source, output);
            }
            ResourceOp::Drop { place, span } => {
                if !borrows.release_token(place) {
                    self.check_source_exclusive(
                        borrows,
                        place,
                        ResourceBorrowOperation::Drop,
                        *span,
                    );
                }
            }
            ResourceOp::Branch {
                then_ops, else_ops, ..
            } => {
                let mut then_borrows = borrows.clone();
                let mut else_borrows = borrows.clone();
                let mut then_function_aliases = function_aliases.clone();
                let mut else_function_aliases = function_aliases.clone();
                self.check_ops(&mut then_borrows, &mut then_function_aliases, then_ops);
                self.check_ops(&mut else_borrows, &mut else_function_aliases, else_ops);
                *borrows = BorrowTable::merge_paths(&[then_borrows, else_borrows]);
                *function_aliases = FunctionAliasTable::merge_paths(&[
                    then_function_aliases,
                    else_function_aliases,
                ]);
            }
            ResourceOp::Loop {
                condition_ops,
                body_ops,
                ..
            } => {
                let mut condition_borrows = borrows.clone();
                let mut condition_function_aliases = function_aliases.clone();
                self.check_ops(
                    &mut condition_borrows,
                    &mut condition_function_aliases,
                    condition_ops,
                );
                let mut body_borrows = condition_borrows.clone();
                let mut body_function_aliases = condition_function_aliases.clone();
                self.check_ops(&mut body_borrows, &mut body_function_aliases, body_ops);
                *borrows = BorrowTable::merge_paths(&[condition_borrows, body_borrows]);
                *function_aliases = FunctionAliasTable::merge_paths(&[
                    condition_function_aliases,
                    body_function_aliases,
                ]);
            }
            ResourceOp::Match { arms, .. } => {
                let mut arm_paths = Vec::new();
                let mut function_alias_paths = Vec::new();
                for arm in arms {
                    let mut arm_borrows = borrows.clone();
                    let mut arm_function_aliases = function_aliases.clone();
                    self.check_ops(&mut arm_borrows, &mut arm_function_aliases, &arm.ops);
                    arm_paths.push(arm_borrows);
                    function_alias_paths.push(arm_function_aliases);
                }
                if !arm_paths.is_empty() {
                    *borrows = BorrowTable::merge_paths(&arm_paths);
                    *function_aliases = FunctionAliasTable::merge_paths(&function_alias_paths);
                }
            }
            ResourceOp::FunctionValue { output, name, .. } => {
                function_aliases.set_alias(output, name.clone());
            }
            ResourceOp::Call {
                output,
                target,
                args,
                ..
            } => self.propagate_call_return_token(borrows, output, target, args),
            ResourceOp::IndirectCall {
                output,
                callee,
                args,
                ..
            } => self.propagate_indirect_call_return_token(
                borrows,
                function_aliases,
                output,
                callee,
                args,
            ),
            ResourceOp::Expr { .. }
            | ResourceOp::CallEffect { .. }
            | ResourceOp::RawMemory { .. }
            | ResourceOp::Construct { .. } => {}
        }
    }

    fn start_borrow(
        &mut self,
        borrows: &mut BorrowTable,
        source: &Place,
        output: &Place,
        kind: BorrowKind,
        span: Span,
    ) {
        match (kind, borrows.state(source)) {
            (BorrowKind::Shared, BorrowState::Unique { .. }) => {
                self.push_conflict(
                    ResourceBorrowOperation::SharedBorrow,
                    source,
                    borrows.state(source),
                    span,
                );
            }
            (BorrowKind::Unique, BorrowState::Shared { .. } | BorrowState::Unique { .. }) => {
                self.push_conflict(
                    ResourceBorrowOperation::UniqueBorrow,
                    source,
                    borrows.state(source),
                    span,
                );
            }
            (BorrowKind::Shared, _) => borrows.add_shared(source, output),
            (BorrowKind::Unique, _) => borrows.add_unique(source, output),
        }
    }

    fn check_source_read(&mut self, borrows: &BorrowTable, place: &Place, span: Span) {
        if let BorrowState::Unique { .. } = borrows.state(place) {
            self.push_conflict(
                ResourceBorrowOperation::Read,
                place,
                borrows.state(place),
                span,
            );
        }
    }

    fn check_return_escape(&mut self, borrows: &BorrowTable, place: &Place, span: Span) {
        if let Some(binding) = borrows.binding(place) {
            let active = borrows.state(&binding.source);
            if matches!(
                active,
                BorrowState::Shared { .. } | BorrowState::Unique { .. }
            ) {
                self.push_conflict(ResourceBorrowOperation::ReturnValue, place, active, span);
            }
        }
    }

    fn propagate_call_return_token(
        &self,
        borrows: &mut BorrowTable,
        output: &Place,
        target: &ResourceCallTarget,
        args: &[Place],
    ) {
        let ResourceCallTarget::User { name, .. } = target else {
            return;
        };
        let Some(summary) = self
            .summaries
            .iter()
            .find(|summary| summary.function == name.as_str())
        else {
            return;
        };
        for arg in summary
            .parameter_indices
            .iter()
            .filter_map(|index| args.get(*index))
        {
            if borrows.copy_or_move_token(arg, output) {
                return;
            }
        }
    }

    fn propagate_indirect_call_return_token(
        &self,
        borrows: &mut BorrowTable,
        function_aliases: &FunctionAliasTable,
        output: &Place,
        callee: &Place,
        args: &[Place],
    ) {
        let functions = function_aliases.functions(callee);
        if functions.is_empty() {
            self.propagate_unknown_indirect_call_return_token(borrows, output, args);
            return;
        }
        for function in functions {
            if let Some(summary) = self
                .summaries
                .iter()
                .find(|summary| summary.function == function.as_str())
            {
                for arg in summary
                    .parameter_indices
                    .iter()
                    .filter_map(|index| args.get(*index))
                {
                    if borrows.copy_or_move_token(arg, output) {
                        return;
                    }
                }
            }
        }
    }

    fn propagate_unknown_indirect_call_return_token(
        &self,
        borrows: &mut BorrowTable,
        output: &Place,
        args: &[Place],
    ) {
        for arg in args.iter().filter(|arg| arg.ty == output.ty) {
            if borrows.copy_or_move_token(arg, output) {
                return;
            }
        }
    }

    fn check_source_exclusive(
        &mut self,
        borrows: &BorrowTable,
        place: &Place,
        operation: ResourceBorrowOperation,
        span: Span,
    ) {
        match borrows.state(place) {
            BorrowState::Shared { .. } | BorrowState::Unique { .. } => {
                self.push_conflict(operation, place, borrows.state(place), span);
            }
            BorrowState::Unborrowed | BorrowState::Released => {}
        }
    }

    fn push_conflict(
        &mut self,
        operation: ResourceBorrowOperation,
        place: &Place,
        active: BorrowState,
        span: Span,
    ) {
        self.diagnostics
            .push(ResourceBorrowDiagnostic::BorrowConflict {
                function: String::from(self.function),
                operation,
                place: place.clone(),
                active,
                span,
            });
    }
}

fn compute_borrow_token_return_summaries(module: &ResourceModule) -> Vec<BorrowTokenReturnSummary> {
    let mut summaries = Vec::new();
    for _ in 0..=module.functions.len() {
        let mut next = Vec::new();
        for function in &module.functions {
            let mut parameter_indices = Vec::new();
            for (index, param) in function.params.iter().enumerate() {
                if function_returns_borrow_token(function, &param.place, &summaries) {
                    parameter_indices.push(index);
                }
            }
            if !parameter_indices.is_empty() {
                next.push(BorrowTokenReturnSummary {
                    function: function.name.clone(),
                    parameter_indices,
                });
            }
        }
        if next == summaries {
            return summaries;
        }
        summaries = next;
    }
    summaries
}

fn function_returns_borrow_token(
    function: &ResourceFunction,
    parameter: &Place,
    summaries: &[BorrowTokenReturnSummary],
) -> bool {
    let mut engine = ResourceBorrowCheckEngine {
        function: function.name.as_str(),
        summaries,
        diagnostics: Vec::new(),
        deferred: ResourceBorrowCheckDeferred::default(),
    };
    let mut borrows = BorrowTable::default();
    let mut function_aliases = FunctionAliasTable::default();
    borrows.add_shared(parameter, parameter);
    for block in &function.blocks {
        engine.check_ops(&mut borrows, &mut function_aliases, &block.ops);
        if let ResourceTerminator::Return {
            value: Some(value), ..
        } = &block.terminator
        {
            if borrows
                .binding(value)
                .is_some_and(|binding| binding.source == *parameter)
            {
                return true;
            }
        }
    }
    false
}

fn compute_owner_return_summaries(module: &ResourceModule) -> Vec<OwnerReturnSummary> {
    let mut summaries = Vec::new();
    for _ in 0..=module.functions.len() {
        let mut next = Vec::new();
        for function in &module.functions {
            let summary = function_owner_return_summary(function, &summaries);
            if summary.returns_fresh_owner
                || !summary.parameter_indices.is_empty()
                || !summary.projection_returns.is_empty()
            {
                next.push(summary);
            }
        }
        if next == summaries {
            return summaries;
        }
        summaries = next;
    }
    summaries
}

fn function_owner_return_summary(
    function: &ResourceFunction,
    summaries: &[OwnerReturnSummary],
) -> OwnerReturnSummary {
    let mut engine = ResourceOwnerCheckEngine {
        function: function.name.as_str(),
        summaries,
        diagnostics: Vec::new(),
        deferred: ResourceOwnerCheckDeferred::default(),
    };
    let mut owners = OwnerTable::default();
    let mut parameter_storages = Vec::new();
    for param in &function.params {
        owners.allocate(&param.place);
        if let Some(OwnerState::Live { storage }) = owners.state(&param.place) {
            parameter_storages.push(storage);
        }
    }

    let mut parameter_indices = Vec::new();
    let mut returns_fresh_owner = false;
    let mut projection_returns = Vec::new();
    let mut function_aliases = FunctionAliasTable::default();
    for block in &function.blocks {
        engine.check_ops(&mut owners, &mut function_aliases, &block.ops);
        if let ResourceTerminator::Return {
            value: Some(value), ..
        } = &block.terminator
        {
            match owners.state(value) {
                Some(OwnerState::Live { storage }) => {
                    if let Some(index) = parameter_storages
                        .iter()
                        .position(|parameter_storage| *parameter_storage == storage)
                    {
                        push_unique_usize(&mut parameter_indices, index);
                    } else {
                        returns_fresh_owner = true;
                    }
                }
                Some(OwnerState::MaybeFreed) => {
                    returns_fresh_owner = true;
                }
                Some(OwnerState::NoFreeObligation | OwnerState::Moved | OwnerState::Freed)
                | None => {}
            }
            for entry in owners.descendant_entries(value) {
                if let OwnerState::Live { storage } = entry.state {
                    if let Some(suffix) = place_suffix_after_prefix(&entry.place, value) {
                        record_projection_owner_return(
                            &mut projection_returns,
                            suffix,
                            entry.place.ty,
                            storage,
                            &parameter_storages,
                        );
                    }
                }
            }
        }
    }

    OwnerReturnSummary {
        function: function.name.clone(),
        parameter_indices,
        returns_fresh_owner,
        projection_returns,
    }
}

fn record_projection_owner_return(
    projection_returns: &mut Vec<OwnerProjectionReturnSummary>,
    suffix: Vec<PlaceProjection>,
    ty: TypeId,
    storage: StorageId,
    parameter_storages: &[StorageId],
) {
    let entry_index = projection_returns
        .iter()
        .position(|entry| entry.suffix == suffix && entry.ty == ty)
        .unwrap_or_else(|| {
            projection_returns.push(OwnerProjectionReturnSummary {
                suffix: suffix.clone(),
                ty,
                parameter_indices: Vec::new(),
                returns_fresh_owner: false,
            });
            projection_returns.len() - 1
        });
    if let Some(parameter_index) = parameter_storages
        .iter()
        .position(|parameter_storage| *parameter_storage == storage)
    {
        push_unique_usize(
            &mut projection_returns[entry_index].parameter_indices,
            parameter_index,
        );
    } else {
        projection_returns[entry_index].returns_fresh_owner = true;
    }
}

impl ResourceOwnerCheckEngine<'_> {
    fn check_function(&mut self, function: &ResourceFunction) -> Vec<OwnerStateEntry> {
        let mut owners = OwnerTable::default();
        let mut function_aliases = FunctionAliasTable::default();
        for block in &function.blocks {
            self.check_block(&mut owners, &mut function_aliases, block);
        }
        self.push_live_owner_diagnostics(&owners, function.span);
        owners.into_entries()
    }

    fn check_block(
        &mut self,
        owners: &mut OwnerTable,
        function_aliases: &mut FunctionAliasTable,
        block: &ResourceBlock,
    ) {
        self.check_ops(owners, function_aliases, &block.ops);
        match &block.terminator {
            ResourceTerminator::Return { value, span } => {
                if let Some(value) = value {
                    self.move_owner_out(owners, value, ResourceOwnerOperation::ReturnValue, *span);
                }
            }
            ResourceTerminator::Unreachable { .. } | ResourceTerminator::RawBody { .. } => {}
        }
    }

    fn check_ops(
        &mut self,
        owners: &mut OwnerTable,
        function_aliases: &mut FunctionAliasTable,
        ops: &[ResourceOp],
    ) {
        for op in ops {
            self.check_op(owners, function_aliases, op);
        }
    }

    fn check_op(
        &mut self,
        owners: &mut OwnerTable,
        function_aliases: &mut FunctionAliasTable,
        op: &ResourceOp,
    ) {
        match op {
            ResourceOp::DeclareLocal {
                place,
                initializer,
                span,
                ..
            } => {
                if let Some(initializer) = initializer {
                    self.transfer_owner(
                        owners,
                        initializer,
                        place,
                        ResourceOwnerOperation::DeclareInitializer,
                        *span,
                    );
                    function_aliases.copy_alias(initializer, place);
                }
            }
            ResourceOp::Read {
                source,
                output,
                span,
            } => {
                self.transfer_owner(owners, source, output, ResourceOwnerOperation::Read, *span);
                function_aliases.copy_alias(source, output);
            }
            ResourceOp::Assign {
                target,
                value,
                span,
            } => {
                self.transfer_owner(
                    owners,
                    value,
                    target,
                    ResourceOwnerOperation::AssignValue,
                    *span,
                );
                function_aliases.copy_alias(value, target);
            }
            ResourceOp::Move {
                source,
                output,
                span,
            } => {
                self.transfer_owner(owners, source, output, ResourceOwnerOperation::Move, *span);
                function_aliases.copy_alias(source, output);
            }
            ResourceOp::RawMemory {
                operation,
                output,
                args,
                span,
            } => match operation {
                RawMemoryOp::Alloc => {
                    owners.allocate(output);
                }
                RawMemoryOp::Dealloc => {
                    if let Some(ptr) = args.first() {
                        self.release_owner(owners, ptr, ResourceOwnerOperation::Dealloc, *span);
                    }
                }
                RawMemoryOp::Realloc => {
                    if let Some(ptr) = args.first() {
                        if self.release_owner(
                            owners,
                            ptr,
                            ResourceOwnerOperation::ReallocInput,
                            *span,
                        ) {
                            owners.allocate(output);
                        }
                    }
                }
                RawMemoryOp::Load
                | RawMemoryOp::Store
                | RawMemoryOp::BulkCopy
                | RawMemoryOp::BulkMove
                | RawMemoryOp::MemorySize
                | RawMemoryOp::MemoryGrow
                | RawMemoryOp::Fill
                | RawMemoryOp::Other { .. } => {}
            },
            ResourceOp::Branch {
                output,
                then_ops,
                then_value,
                else_ops,
                else_value,
                span,
                ..
            } => {
                let mut then_owners = owners.clone();
                let mut else_owners = owners.clone();
                let mut then_function_aliases = function_aliases.clone();
                let mut else_function_aliases = function_aliases.clone();
                self.check_ops(&mut then_owners, &mut then_function_aliases, then_ops);
                self.check_ops(&mut else_owners, &mut else_function_aliases, else_ops);
                self.transfer_owner(
                    &mut then_owners,
                    then_value,
                    output,
                    ResourceOwnerOperation::BranchValue,
                    *span,
                );
                self.transfer_owner(
                    &mut else_owners,
                    else_value,
                    output,
                    ResourceOwnerOperation::BranchValue,
                    *span,
                );
                *owners = OwnerTable::merge_paths(&[then_owners, else_owners]);
                *function_aliases = FunctionAliasTable::merge_paths(&[
                    then_function_aliases,
                    else_function_aliases,
                ]);
            }
            ResourceOp::Loop {
                condition_ops,
                body_ops,
                ..
            } => {
                let mut condition_owners = owners.clone();
                let mut condition_function_aliases = function_aliases.clone();
                self.check_ops(
                    &mut condition_owners,
                    &mut condition_function_aliases,
                    condition_ops,
                );
                let mut body_owners = condition_owners.clone();
                let mut body_function_aliases = condition_function_aliases.clone();
                self.check_ops(&mut body_owners, &mut body_function_aliases, body_ops);
                *owners = OwnerTable::merge_paths(&[condition_owners, body_owners]);
                *function_aliases = FunctionAliasTable::merge_paths(&[
                    condition_function_aliases,
                    body_function_aliases,
                ]);
            }
            ResourceOp::Match {
                output, arms, span, ..
            } => {
                let mut arm_paths = Vec::new();
                let mut function_alias_paths = Vec::new();
                for arm in arms {
                    let mut arm_owners = owners.clone();
                    let mut arm_function_aliases = function_aliases.clone();
                    self.check_ops(&mut arm_owners, &mut arm_function_aliases, &arm.ops);
                    self.transfer_owner(
                        &mut arm_owners,
                        &arm.value,
                        output,
                        ResourceOwnerOperation::MatchValue,
                        *span,
                    );
                    arm_paths.push(arm_owners);
                    function_alias_paths.push(arm_function_aliases);
                }
                if !arm_paths.is_empty() {
                    *owners = OwnerTable::merge_paths(&arm_paths);
                    *function_aliases = FunctionAliasTable::merge_paths(&function_alias_paths);
                }
            }
            ResourceOp::FunctionValue { output, name, .. } => {
                function_aliases.set_alias(output, name.clone());
            }
            ResourceOp::Call {
                output,
                target,
                args,
                span,
                ..
            } => self.apply_call_return_owner(owners, output, target, args, *span),
            ResourceOp::IndirectCall {
                output,
                callee,
                args,
                span,
                ..
            } => self.apply_indirect_call_return_owner(
                owners,
                function_aliases,
                output,
                callee,
                args,
                *span,
            ),
            ResourceOp::Construct {
                output,
                kind,
                inputs,
                span,
            } => self.construct_owner_fields(owners, output, kind, inputs, *span),
            ResourceOp::Expr { .. }
            | ResourceOp::Borrow { .. }
            | ResourceOp::Drop { .. }
            | ResourceOp::CallEffect { .. } => {}
        }
    }

    fn construct_owner_fields(
        &mut self,
        owners: &mut OwnerTable,
        output: &Place,
        kind: &AggregateKind,
        inputs: &[Place],
        span: Span,
    ) {
        for (index, input) in inputs.iter().enumerate() {
            let field = construct_owner_field_place(output, kind, index, input);
            self.transfer_owner(
                owners,
                input,
                &field,
                ResourceOwnerOperation::ConstructInput,
                span,
            );
        }
    }

    fn apply_call_return_owner(
        &mut self,
        owners: &mut OwnerTable,
        output: &Place,
        target: &ResourceCallTarget,
        args: &[Place],
        span: Span,
    ) {
        let ResourceCallTarget::User { name, .. } = target else {
            return;
        };
        let Some(summary) = self
            .summaries
            .iter()
            .find(|summary| summary.function == name.as_str())
        else {
            return;
        };
        self.apply_owner_return_summary(owners, output, args, summary, span);
    }

    fn apply_indirect_call_return_owner(
        &mut self,
        owners: &mut OwnerTable,
        function_aliases: &FunctionAliasTable,
        output: &Place,
        callee: &Place,
        args: &[Place],
        span: Span,
    ) {
        let functions = function_aliases.functions(callee);
        if functions.is_empty() {
            self.apply_unknown_indirect_call_return_owner(owners, output, args, span);
            return;
        }
        for function in functions {
            if let Some(summary) = self
                .summaries
                .iter()
                .find(|summary| summary.function == function.as_str())
            {
                self.apply_owner_return_summary(owners, output, args, summary, span);
                if owners.has_transferable_owner(output) {
                    return;
                }
            }
        }
    }

    fn apply_unknown_indirect_call_return_owner(
        &mut self,
        owners: &mut OwnerTable,
        output: &Place,
        args: &[Place],
        span: Span,
    ) {
        for arg in args.iter().filter(|arg| arg.ty == output.ty) {
            if owners.has_transferable_owner(arg) {
                self.transfer_owner(
                    owners,
                    arg,
                    output,
                    ResourceOwnerOperation::ReturnValue,
                    span,
                );
                return;
            }
        }
    }

    fn apply_owner_return_summary(
        &mut self,
        owners: &mut OwnerTable,
        output: &Place,
        args: &[Place],
        summary: &OwnerReturnSummary,
        span: Span,
    ) {
        let mut transferred = false;
        for arg in summary
            .parameter_indices
            .iter()
            .filter_map(|index| args.get(*index))
        {
            if owners.has_transferable_owner(arg) {
                self.transfer_owner(
                    owners,
                    arg,
                    output,
                    ResourceOwnerOperation::ReturnValue,
                    span,
                );
                transferred = true;
                break;
            }
        }
        if summary.returns_fresh_owner && !transferred {
            owners.allocate(output);
        }
        for projection in &summary.projection_returns {
            let output_projection = place_with_suffix(output, &projection.suffix, projection.ty);
            self.apply_owner_projection_return_summary(
                owners,
                &output_projection,
                args,
                projection,
                span,
            );
        }
    }

    fn apply_owner_projection_return_summary(
        &mut self,
        owners: &mut OwnerTable,
        output: &Place,
        args: &[Place],
        summary: &OwnerProjectionReturnSummary,
        span: Span,
    ) {
        let mut transferred = false;
        for arg in summary
            .parameter_indices
            .iter()
            .filter_map(|index| args.get(*index))
        {
            if owners.has_transferable_owner(arg) {
                self.transfer_owner(
                    owners,
                    arg,
                    output,
                    ResourceOwnerOperation::ReturnValue,
                    span,
                );
                transferred = true;
                break;
            }
        }
        if summary.returns_fresh_owner && !transferred {
            owners.allocate(output);
        }
    }

    fn transfer_owner(
        &mut self,
        owners: &mut OwnerTable,
        source: &Place,
        target: &Place,
        operation: ResourceOwnerOperation,
        span: Span,
    ) {
        if source == target || !should_track(source) {
            return;
        }
        let descendants = owners.descendant_entries(source);
        match owners.state(source) {
            Some(OwnerState::Live { storage }) => {
                owners.set_state(source, OwnerState::Moved);
                if should_track(target) {
                    owners.set_state(target, OwnerState::Live { storage });
                }
            }
            Some(OwnerState::Moved | OwnerState::Freed | OwnerState::MaybeFreed) => {
                let state = owners.state(source).unwrap_or(OwnerState::NoFreeObligation);
                self.push_unavailable(operation, source, state, span);
            }
            Some(OwnerState::NoFreeObligation) | None => {}
        }
        for entry in descendants {
            let Some(target_place) = replace_place_prefix(&entry.place, source, target) else {
                continue;
            };
            match entry.state {
                OwnerState::Live { storage } => {
                    owners.set_state(&entry.place, OwnerState::Moved);
                    if should_track(&target_place) {
                        owners.set_state(&target_place, OwnerState::Live { storage });
                    }
                }
                OwnerState::Moved | OwnerState::Freed | OwnerState::MaybeFreed => {
                    self.push_unavailable(operation, &entry.place, entry.state, span);
                }
                OwnerState::NoFreeObligation => {}
            }
        }
    }

    fn move_owner_out(
        &mut self,
        owners: &mut OwnerTable,
        place: &Place,
        operation: ResourceOwnerOperation,
        span: Span,
    ) {
        if !should_track(place) {
            return;
        }
        let descendants = owners.descendant_entries(place);
        match owners.state(place) {
            Some(OwnerState::Live { .. }) => {
                owners.set_state(place, OwnerState::Moved);
            }
            Some(OwnerState::Moved | OwnerState::Freed | OwnerState::MaybeFreed) => {
                let state = owners.state(place).unwrap_or(OwnerState::NoFreeObligation);
                self.push_unavailable(operation, place, state, span);
            }
            Some(OwnerState::NoFreeObligation) | None => {}
        }
        for entry in descendants {
            match entry.state {
                OwnerState::Live { .. } => {
                    owners.set_state(&entry.place, OwnerState::Moved);
                }
                OwnerState::Moved | OwnerState::Freed | OwnerState::MaybeFreed => {
                    self.push_unavailable(operation, &entry.place, entry.state, span);
                }
                OwnerState::NoFreeObligation => {}
            }
        }
    }

    fn release_owner(
        &mut self,
        owners: &mut OwnerTable,
        place: &Place,
        operation: ResourceOwnerOperation,
        span: Span,
    ) -> bool {
        if !should_track(place) {
            return false;
        }
        match owners.state(place) {
            Some(OwnerState::Live { .. }) => {
                owners.set_state(place, OwnerState::Freed);
                true
            }
            Some(state) => {
                self.push_unavailable(operation, place, state, span);
                false
            }
            None => {
                self.push_unavailable(operation, place, OwnerState::NoFreeObligation, span);
                false
            }
        }
    }

    fn push_live_owner_diagnostics(&mut self, owners: &OwnerTable, span: Span) {
        for entry in owners.live_entries() {
            match entry.state {
                OwnerState::Live { storage } => {
                    self.diagnostics.push(ResourceOwnerDiagnostic::OwnerLeaked {
                        function: String::from(self.function),
                        place: entry.place,
                        storage,
                        span,
                    });
                }
                OwnerState::MaybeFreed => {
                    self.diagnostics
                        .push(ResourceOwnerDiagnostic::OwnerMaybeLeaked {
                            function: String::from(self.function),
                            place: entry.place,
                            span,
                        });
                }
                OwnerState::NoFreeObligation | OwnerState::Moved | OwnerState::Freed => {}
            }
        }
    }

    fn push_unavailable(
        &mut self,
        operation: ResourceOwnerOperation,
        place: &Place,
        state: OwnerState,
        span: Span,
    ) {
        self.diagnostics
            .push(ResourceOwnerDiagnostic::OwnerUnavailable {
                function: String::from(self.function),
                operation,
                place: place.clone(),
                state,
                span,
            });
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct BorrowBinding {
    token: Place,
    source: Place,
    kind: BorrowKind,
}

#[derive(Debug, Clone, Default)]
struct FunctionAliasTable {
    entries: Vec<FunctionAliasEntry>,
}

#[derive(Debug, Clone)]
struct FunctionAliasEntry {
    place: Place,
    functions: Vec<String>,
}

impl FunctionAliasTable {
    fn functions(&self, place: &Place) -> &[String] {
        self.entries
            .iter()
            .find(|entry| entry.place == *place)
            .map(|entry| entry.functions.as_slice())
            .unwrap_or(&[])
    }

    fn set_alias(&mut self, place: &Place, function: String) {
        self.set_functions(place, vec![function]);
    }

    fn copy_alias(&mut self, source: &Place, target: &Place) {
        let functions = self.functions(source).to_vec();
        if !functions.is_empty() {
            self.set_functions(target, functions);
        }
    }

    fn set_functions(&mut self, place: &Place, functions: Vec<String>) {
        if let Some(entry) = self.entries.iter_mut().find(|entry| entry.place == *place) {
            entry.functions = dedupe_functions(functions);
            return;
        }
        self.entries.push(FunctionAliasEntry {
            place: place.clone(),
            functions: dedupe_functions(functions),
        });
    }

    fn merge_paths(paths: &[FunctionAliasTable]) -> Self {
        let mut out = FunctionAliasTable::default();
        for path in paths {
            for entry in &path.entries {
                out.union_functions(&entry.place, entry.functions.iter().cloned());
            }
        }
        out
    }

    fn union_functions<I>(&mut self, place: &Place, functions: I)
    where
        I: IntoIterator<Item = String>,
    {
        let mut merged = self.functions(place).to_vec();
        for function in functions {
            if !merged.contains(&function) {
                merged.push(function);
            }
        }
        if !merged.is_empty() {
            self.set_functions(place, merged);
        }
    }
}

fn dedupe_functions(functions: Vec<String>) -> Vec<String> {
    let mut out = Vec::new();
    for function in functions {
        if !out.contains(&function) {
            out.push(function);
        }
    }
    out
}

#[derive(Debug, Clone, Default)]
struct BorrowTable {
    sources: Vec<BorrowStateEntry>,
    bindings: Vec<BorrowBinding>,
}

impl BorrowTable {
    fn into_entries(self) -> Vec<BorrowStateEntry> {
        self.sources
    }

    fn state(&self, place: &Place) -> BorrowState {
        self.sources
            .iter()
            .find(|entry| entry.place == *place)
            .map(|entry| entry.state.clone())
            .unwrap_or(BorrowState::Unborrowed)
    }

    fn add_shared(&mut self, source: &Place, token: &Place) {
        let next_count = match self.state(source) {
            BorrowState::Shared { count } => count + 1,
            _ => 1,
        };
        self.set_source(source, BorrowState::Shared { count: next_count });
        self.bindings.push(BorrowBinding {
            token: token.clone(),
            source: source.clone(),
            kind: BorrowKind::Shared,
        });
    }

    fn add_unique(&mut self, source: &Place, token: &Place) {
        self.set_source(
            source,
            BorrowState::Unique {
                source: Box::new(source.clone()),
            },
        );
        self.bindings.push(BorrowBinding {
            token: token.clone(),
            source: source.clone(),
            kind: BorrowKind::Unique,
        });
    }

    fn copy_or_move_token(&mut self, source: &Place, output: &Place) -> bool {
        let Some(index) = self.binding_index(source) else {
            return false;
        };
        let binding = self.bindings[index].clone();
        match binding.kind {
            BorrowKind::Shared => {
                self.add_shared(&binding.source, output);
            }
            BorrowKind::Unique => {
                self.bindings[index].token = output.clone();
            }
        }
        true
    }

    fn transfer_token(&mut self, source: &Place, target: &Place) -> bool {
        let Some(index) = self.binding_index(source) else {
            return false;
        };
        self.bindings[index].token = target.clone();
        true
    }

    fn release_token(&mut self, token: &Place) -> bool {
        let Some(index) = self.binding_index(token) else {
            return false;
        };
        let binding = self.bindings.remove(index);
        self.release_source(&binding.source, binding.kind);
        true
    }

    fn release_source(&mut self, source: &Place, kind: BorrowKind) {
        match (kind, self.state(source)) {
            (BorrowKind::Shared, BorrowState::Shared { count }) if count > 1 => {
                self.set_source(source, BorrowState::Shared { count: count - 1 });
            }
            (BorrowKind::Shared, BorrowState::Shared { .. })
            | (BorrowKind::Unique, BorrowState::Unique { .. }) => {
                self.set_source(source, BorrowState::Released);
            }
            _ => {}
        }
    }

    fn set_source(&mut self, place: &Place, state: BorrowState) {
        if !should_track(place) {
            return;
        }
        if let Some(entry) = self.sources.iter_mut().find(|entry| entry.place == *place) {
            entry.state = state;
        } else {
            self.sources.push(BorrowStateEntry {
                place: place.clone(),
                state,
            });
        }
    }

    fn binding_index(&self, token: &Place) -> Option<usize> {
        self.bindings
            .iter()
            .position(|binding| binding.token == *token)
    }

    fn binding(&self, token: &Place) -> Option<&BorrowBinding> {
        self.bindings.iter().find(|binding| binding.token == *token)
    }

    fn merge_paths(paths: &[BorrowTable]) -> Self {
        let mut out = BorrowTable::default();
        let mut places = Vec::new();
        for path in paths {
            for entry in &path.sources {
                push_unique_place(&mut places, &entry.place);
            }
        }
        for place in places {
            let mut merged = BorrowState::Unborrowed;
            for path in paths {
                merged = merge_borrow_states(merged, path.state(&place));
            }
            out.set_source(&place, merged);
        }
        for path in paths {
            for binding in &path.bindings {
                if out.binding_index(&binding.token).is_none() {
                    out.bindings.push(binding.clone());
                }
            }
        }
        let sources = out.sources.clone();
        out.bindings.retain(|binding| {
            let state = sources
                .iter()
                .find(|entry| entry.place == binding.source)
                .map(|entry| entry.state.clone())
                .unwrap_or(BorrowState::Unborrowed);
            matches!(
                state,
                BorrowState::Shared { .. } | BorrowState::Unique { .. }
            )
        });
        out
    }
}

#[derive(Debug, Clone, Default)]
struct CellTable {
    cells: Vec<CellStateEntry>,
}

impl CellTable {
    fn into_entries(self) -> Vec<CellStateEntry> {
        self.cells
    }

    fn state(&self, place: &Place) -> Option<CellState> {
        self.cells
            .iter()
            .find(|entry| entry.place == *place)
            .map(|entry| entry.state.clone())
    }

    fn mark_initialized(&mut self, place: &Place) {
        self.set_state(place, CellState::Initialized(place.ty));
    }

    fn set_state(&mut self, place: &Place, state: CellState) {
        if !should_track(place) {
            return;
        }
        if let Some(entry) = self.cells.iter_mut().find(|entry| entry.place == *place) {
            entry.state = state;
        } else {
            self.cells.push(CellStateEntry {
                place: place.clone(),
                state,
            });
        }
    }

    fn merge_paths(paths: &[CellTable]) -> Self {
        let mut out = CellTable::default();
        let mut places = Vec::new();
        for path in paths {
            for entry in &path.cells {
                push_unique_place(&mut places, &entry.place);
            }
        }
        for place in places {
            let mut merged = CellState::Uninit;
            for path in paths {
                let state = path.state(&place).unwrap_or(CellState::Uninit);
                merged = merge_cell_states(merged, state);
            }
            out.set_state(&place, merged);
        }
        out
    }
}

#[derive(Debug, Clone, Default)]
struct OwnerTable {
    owners: Vec<OwnerStateEntry>,
    next_storage: usize,
}

impl OwnerTable {
    fn into_entries(self) -> Vec<OwnerStateEntry> {
        self.owners
    }

    fn state(&self, place: &Place) -> Option<OwnerState> {
        self.owners
            .iter()
            .find(|entry| entry.place == *place)
            .map(|entry| entry.state.clone())
    }

    fn allocate(&mut self, place: &Place) {
        let storage = StorageId(self.next_storage);
        self.next_storage += 1;
        self.set_state(place, OwnerState::Live { storage });
    }

    fn set_state(&mut self, place: &Place, state: OwnerState) {
        if !should_track(place) {
            return;
        }
        if let Some(entry) = self.owners.iter_mut().find(|entry| entry.place == *place) {
            entry.state = state;
        } else {
            self.owners.push(OwnerStateEntry {
                place: place.clone(),
                state,
            });
        }
    }

    fn live_entries(&self) -> Vec<OwnerStateEntry> {
        self.owners
            .iter()
            .filter(|entry| {
                matches!(
                    entry.state,
                    OwnerState::Live { .. } | OwnerState::MaybeFreed
                )
            })
            .cloned()
            .collect()
    }

    fn descendant_entries(&self, prefix: &Place) -> Vec<OwnerStateEntry> {
        self.owners
            .iter()
            .filter(|entry| {
                entry.place != *prefix
                    && replace_place_prefix(&entry.place, prefix, prefix).is_some()
            })
            .cloned()
            .collect()
    }

    fn has_transferable_owner(&self, place: &Place) -> bool {
        self.state(place)
            .is_some_and(|state| matches!(state, OwnerState::Live { .. }))
            || self
                .descendant_entries(place)
                .iter()
                .any(|entry| matches!(entry.state, OwnerState::Live { .. }))
    }

    fn merge_paths(paths: &[OwnerTable]) -> Self {
        let mut out = OwnerTable::default();
        out.next_storage = paths
            .iter()
            .map(|path| path.next_storage)
            .max()
            .unwrap_or_default();
        let mut places = Vec::new();
        for path in paths {
            for entry in &path.owners {
                push_unique_place(&mut places, &entry.place);
            }
        }
        for place in places {
            let mut merged = OwnerState::NoFreeObligation;
            for path in paths {
                let state = path.state(&place).unwrap_or(OwnerState::NoFreeObligation);
                merged = merge_owner_states(merged, state);
            }
            out.set_state(&place, merged);
        }
        out
    }
}

fn should_track(place: &Place) -> bool {
    !matches!(place.root, PlaceRoot::Unknown)
}

fn construct_owner_field_place(
    output: &Place,
    kind: &AggregateKind,
    index: usize,
    input: &Place,
) -> Place {
    let mut place = output.clone();
    match kind {
        AggregateKind::Struct { .. } => {
            place.projections.push(PlaceProjection::Field {
                index,
                offset_bytes: 0,
            });
        }
        AggregateKind::Tuple => {
            place.projections.push(PlaceProjection::TupleField {
                index,
                offset_bytes: 0,
            });
        }
        AggregateKind::Enum { variant, .. } => {
            place.projections.push(PlaceProjection::EnumPayload {
                variant: variant.clone(),
            });
            if index > 0 {
                place.projections.push(PlaceProjection::TupleField {
                    index,
                    offset_bytes: 0,
                });
            }
        }
    }
    place.ty = input.ty;
    place
}

fn replace_place_prefix(place: &Place, prefix: &Place, replacement: &Place) -> Option<Place> {
    place_suffix_after_prefix(place, prefix)
        .map(|suffix| place_with_suffix(replacement, &suffix, place.ty))
}

fn place_suffix_after_prefix(place: &Place, prefix: &Place) -> Option<Vec<PlaceProjection>> {
    if place.root != prefix.root || place.projections.len() < prefix.projections.len() {
        return None;
    }
    if place.projections[..prefix.projections.len()] != prefix.projections[..] {
        return None;
    }
    Some(place.projections[prefix.projections.len()..].to_vec())
}

fn place_with_suffix(base: &Place, suffix: &[PlaceProjection], ty: TypeId) -> Place {
    let mut out = base.clone();
    out.projections.extend_from_slice(suffix);
    out.ty = ty;
    out
}

fn push_unique_place(places: &mut Vec<Place>, place: &Place) {
    if !places.iter().any(|existing| existing == place) {
        places.push(place.clone());
    }
}

fn push_unique_usize(values: &mut Vec<usize>, value: usize) {
    if !values.contains(&value) {
        values.push(value);
    }
}

fn merge_cell_states(left: CellState, right: CellState) -> CellState {
    if left == right {
        return left;
    }
    match (left, right) {
        (CellState::Initialized(left_ty), CellState::Initialized(right_ty))
            if left_ty == right_ty =>
        {
            CellState::Initialized(left_ty)
        }
        (CellState::Uninit, CellState::Uninit) => CellState::Uninit,
        (CellState::Moved, CellState::Moved) => CellState::Moved,
        (CellState::Dropped, CellState::Dropped) => CellState::Dropped,
        _ => CellState::MaybeMoved,
    }
}

fn merge_owner_states(left: OwnerState, right: OwnerState) -> OwnerState {
    if left == right {
        return left;
    }
    match (left, right) {
        (
            OwnerState::Live {
                storage: left_storage,
            },
            OwnerState::Live {
                storage: right_storage,
            },
        ) if left_storage == right_storage => OwnerState::Live {
            storage: left_storage,
        },
        (OwnerState::NoFreeObligation, OwnerState::Freed)
        | (OwnerState::Freed, OwnerState::NoFreeObligation) => OwnerState::NoFreeObligation,
        (OwnerState::NoFreeObligation, OwnerState::NoFreeObligation) => {
            OwnerState::NoFreeObligation
        }
        (OwnerState::Moved, OwnerState::Moved) => OwnerState::Moved,
        (OwnerState::Freed, OwnerState::Freed) => OwnerState::Freed,
        _ => OwnerState::MaybeFreed,
    }
}

fn merge_borrow_states(left: BorrowState, right: BorrowState) -> BorrowState {
    if left == right {
        return left;
    }
    match (left, right) {
        (BorrowState::Unique { source }, _) | (_, BorrowState::Unique { source }) => {
            BorrowState::Unique { source }
        }
        (BorrowState::Shared { count: left_count }, BorrowState::Shared { count: right_count }) => {
            BorrowState::Shared {
                count: left_count.max(right_count),
            }
        }
        (BorrowState::Shared { count }, BorrowState::Unborrowed)
        | (BorrowState::Unborrowed, BorrowState::Shared { count })
        | (BorrowState::Shared { count }, BorrowState::Released)
        | (BorrowState::Released, BorrowState::Shared { count }) => BorrowState::Shared { count },
        (BorrowState::Released, BorrowState::Unborrowed)
        | (BorrowState::Unborrowed, BorrowState::Released)
        | (BorrowState::Released, BorrowState::Released) => BorrowState::Released,
        (BorrowState::Unborrowed, BorrowState::Unborrowed) => BorrowState::Unborrowed,
    }
}

fn merge_deferred(target: &mut ResourceCheckDeferred, source: ResourceCheckDeferred) {
    target.branch_merges += source.branch_merges;
    target.loop_merges += source.loop_merges;
    target.match_merges += source.match_merges;
}

fn merge_owner_deferred(
    target: &mut ResourceOwnerCheckDeferred,
    source: ResourceOwnerCheckDeferred,
) {
    target.branch_merges += source.branch_merges;
    target.loop_merges += source.loop_merges;
    target.match_merges += source.match_merges;
}

fn merge_borrow_deferred(
    target: &mut ResourceBorrowCheckDeferred,
    source: ResourceBorrowCheckDeferred,
) {
    target.branch_merges += source.branch_merges;
    target.loop_merges += source.loop_merges;
    target.match_merges += source.match_merges;
}
