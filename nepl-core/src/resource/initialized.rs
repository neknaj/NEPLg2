extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;

use crate::span::Span;
use crate::types::TypeCtx;

use super::cell_state::CellTable;
use super::model::{
    CellState, CellStateEntry, Place, RawMemoryOp, ResourceBlock, ResourceExprKind,
    ResourceFunction, ResourceModule, ResourceOp, ResourceTerminator,
};
use super::place_utils::{raw_memory_cell_place, should_track};
use super::report::{
    ResourceCheckDeferred, ResourceCheckDiagnostic, ResourceCheckOperation, ResourceCheckReport,
    ResourceFunctionCheck,
};

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
                operation,
                output,
                args,
                span,
            } => self.check_raw_memory(cells, operation, output, args, *span),
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

    fn check_raw_memory(
        &mut self,
        cells: &mut CellTable,
        operation: &RawMemoryOp,
        output: &Place,
        args: &[Place],
        span: Span,
    ) {
        match operation {
            RawMemoryOp::Load => {
                let Some(address) = args.first() else {
                    cells.mark_initialized(output);
                    return;
                };
                let address_available = self.ensure_available(
                    cells,
                    address,
                    ResourceCheckOperation::RawMemoryLoadAddress,
                    span,
                );
                let cell = raw_memory_cell_place(address, output.ty);
                let cell_available = self.ensure_available(
                    cells,
                    &cell,
                    ResourceCheckOperation::RawMemoryLoadCell,
                    span,
                );
                if address_available && cell_available {
                    if !self.types.is_copy(output.ty) {
                        cells.set_state(&cell, CellState::Moved);
                    }
                    cells.mark_initialized(output);
                }
            }
            RawMemoryOp::Store => {
                let Some(address) = args.first() else {
                    cells.mark_initialized(output);
                    return;
                };
                let address_available = self.ensure_available(
                    cells,
                    address,
                    ResourceCheckOperation::RawMemoryStoreAddress,
                    span,
                );
                let cell_available = self.ensure_no_live_non_copy_raw_cells(
                    cells,
                    address,
                    ResourceCheckOperation::RawMemoryStoreCell,
                    span,
                );
                let value_available = if address_available && cell_available {
                    args.get(1).is_none_or(|value| {
                        self.consume_by_value(
                            cells,
                            value,
                            ResourceCheckOperation::RawMemoryStoreValue,
                            span,
                        )
                    })
                } else {
                    false
                };
                if address_available && cell_available && value_available {
                    if let Some(value) = args.get(1) {
                        let cell = raw_memory_cell_place(address, value.ty);
                        cells.clear_raw_cells_under(address);
                        cells.mark_initialized(&cell);
                    }
                    cells.mark_initialized(output);
                }
            }
            RawMemoryOp::Dealloc => {
                let Some(address) = args.first() else {
                    cells.mark_initialized(output);
                    return;
                };
                let address_available = self.ensure_available(
                    cells,
                    address,
                    ResourceCheckOperation::RawMemoryDeallocAddress,
                    span,
                );
                let cells_released = self.ensure_no_live_non_copy_raw_cells(
                    cells,
                    address,
                    ResourceCheckOperation::RawMemoryDeallocCell,
                    span,
                );
                if address_available && cells_released {
                    cells.clear_raw_cells_under(address);
                    cells.mark_initialized(output);
                }
            }
            RawMemoryOp::Realloc => {
                let Some(address) = args.first() else {
                    cells.mark_initialized(output);
                    return;
                };
                let address_available = self.ensure_available(
                    cells,
                    address,
                    ResourceCheckOperation::RawMemoryReallocAddress,
                    span,
                );
                let cells_released = self.ensure_no_live_non_copy_raw_cells(
                    cells,
                    address,
                    ResourceCheckOperation::RawMemoryReallocCell,
                    span,
                );
                if address_available && cells_released {
                    let relocated =
                        cells.copy_initialized_copy_raw_cells(address, output, self.types);
                    cells.clear_raw_cells_under(address);
                    cells.mark_initialized(output);
                    cells.extend_entries(relocated);
                }
            }
            RawMemoryOp::Fill => {
                let Some(address) = args.first() else {
                    cells.mark_initialized(output);
                    return;
                };
                let address_available = self.ensure_available(
                    cells,
                    address,
                    ResourceCheckOperation::RawMemoryFillAddress,
                    span,
                );
                let cells_released = self.ensure_no_live_non_copy_raw_cells(
                    cells,
                    address,
                    ResourceCheckOperation::RawMemoryFillCell,
                    span,
                );
                if address_available && cells_released {
                    cells.clear_raw_cells_under(address);
                    cells.mark_initialized(output);
                }
            }
            RawMemoryOp::BulkCopy | RawMemoryOp::BulkMove => {
                let Some(destination) = args.first() else {
                    cells.mark_initialized(output);
                    return;
                };
                let Some(source) = args.get(1) else {
                    cells.mark_initialized(output);
                    return;
                };
                let destination_available = self.ensure_available(
                    cells,
                    destination,
                    ResourceCheckOperation::RawMemoryBulkDestinationAddress,
                    span,
                );
                let source_available = self.ensure_available(
                    cells,
                    source,
                    ResourceCheckOperation::RawMemoryBulkSourceAddress,
                    span,
                );
                let destination_cells_released = self.ensure_no_live_non_copy_raw_cells(
                    cells,
                    destination,
                    ResourceCheckOperation::RawMemoryBulkDestinationCell,
                    span,
                );
                let source_cells_copyable = self.ensure_no_live_non_copy_raw_cells(
                    cells,
                    source,
                    ResourceCheckOperation::RawMemoryBulkSourceCell,
                    span,
                );
                if destination_available
                    && source_available
                    && destination_cells_released
                    && source_cells_copyable
                {
                    let copied =
                        cells.copy_initialized_copy_raw_cells(source, destination, self.types);
                    cells.clear_raw_cells_under(destination);
                    cells.extend_entries(copied);
                    cells.mark_initialized(output);
                }
            }
            _ => {
                let args_available =
                    self.ensure_args(cells, args, ResourceCheckOperation::RawMemoryArgument, span);
                if args_available {
                    cells.mark_initialized(output);
                }
            }
        }
    }

    fn ensure_no_live_non_copy_raw_cells(
        &mut self,
        cells: &CellTable,
        address: &Place,
        operation: ResourceCheckOperation,
        span: Span,
    ) -> bool {
        let conflicts = cells.live_non_copy_raw_cells_under(address, self.types);
        for conflict in &conflicts {
            self.push_unavailable(operation, &conflict.place, conflict.state.clone(), span);
        }
        conflicts.is_empty()
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
        match cells.availability_state(place) {
            CellState::Initialized(_) => true,
            state => {
                self.push_unavailable(operation, place, state, span);
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

fn merge_deferred(target: &mut ResourceCheckDeferred, source: ResourceCheckDeferred) {
    target.branch_merges += source.branch_merges;
    target.loop_merges += source.loop_merges;
    target.match_merges += source.match_merges;
}
