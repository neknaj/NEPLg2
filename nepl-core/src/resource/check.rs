extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;

use crate::span::Span;
use crate::types::TypeCtx;

use super::model::{
    CellState, CellStateEntry, Place, PlaceRoot, ResourceBlock, ResourceExprKind, ResourceFunction,
    ResourceModule, ResourceOp, ResourceTerminator,
};

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

struct ResourceCheckEngine<'a> {
    function: &'a str,
    types: &'a TypeCtx,
    diagnostics: Vec<ResourceCheckDiagnostic>,
    deferred: ResourceCheckDeferred,
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
                self.deferred.branch_merges += 1;
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
                self.deferred.loop_merges += 1;
                let mut condition_cells = cells.clone();
                self.check_ops(&mut condition_cells, condition_ops);
                self.consume_by_value(
                    &mut condition_cells,
                    condition,
                    ResourceCheckOperation::LoopCondition,
                    *span,
                );
                let mut body_cells = cells.clone();
                self.check_ops(&mut body_cells, body_ops);
            }
            ResourceOp::Match {
                output,
                scrutinee,
                arms,
                span,
            } => {
                self.deferred.match_merges += 1;
                let scrutinee_available = self.consume_by_value(
                    cells,
                    scrutinee,
                    ResourceCheckOperation::MatchScrutinee,
                    *span,
                );
                let mut arms_available = true;
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
}

fn should_track(place: &Place) -> bool {
    !matches!(place.root, PlaceRoot::Unknown)
}

fn merge_deferred(target: &mut ResourceCheckDeferred, source: ResourceCheckDeferred) {
    target.branch_merges += source.branch_merges;
    target.loop_merges += source.loop_merges;
    target.match_merges += source.match_merges;
}
