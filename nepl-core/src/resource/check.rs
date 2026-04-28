extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;

use crate::hir::HirModule;
use crate::span::Span;
use crate::types::TypeCtx;

use super::borrow_state::BorrowTable;
use super::cell_state::CellTable;
use super::coverage::compare_hir_resource_lowering;
use super::effect::check_resource_effect_boundaries;
use super::function_alias::{construct_function_alias_fields, FunctionAliasTable};
use super::lower::lower_hir_module_skeleton;
use super::model::{
    AggregateKind, BorrowKind, BorrowState, BorrowStateEntry, CellState, CellStateEntry,
    OwnerState, OwnerStateEntry, Place, RawMemoryOp, ResourceBlock, ResourceCallTarget,
    ResourceExprKind, ResourceFunction, ResourceModule, ResourceOp, ResourceTerminator,
};
use super::owner_state::OwnerTable;
use super::place_utils::{
    construct_aggregate_field_place, place_with_suffix, places_overlap, raw_memory_cell_place,
    replace_place_prefix, should_track,
};
use super::report::{
    ResourceBorrowCheckDeferred, ResourceBorrowCheckReport, ResourceBorrowDiagnostic,
    ResourceBorrowFunctionCheck, ResourceBorrowOperation, ResourceCheckDeferred,
    ResourceCheckDiagnostic, ResourceCheckOperation, ResourceCheckReport, ResourceFunctionCheck,
    ResourceOwnerCheckDeferred, ResourceOwnerCheckReport, ResourceOwnerDiagnostic,
    ResourceOwnerFunctionCheck, ResourceOwnerOperation, ResourceSafetyShadowReport,
};
use super::summary::{
    compute_borrow_token_return_summaries, compute_owner_return_summaries,
    BorrowTokenReturnSummary, OwnerProjectionReturnSummary, OwnerReturnSummary,
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

pub(super) struct ResourceOwnerCheckEngine<'a> {
    pub(super) function: &'a str,
    pub(super) summaries: &'a [OwnerReturnSummary],
    pub(super) diagnostics: Vec<ResourceOwnerDiagnostic>,
    pub(super) deferred: ResourceOwnerCheckDeferred,
}

pub(super) struct ResourceBorrowCheckEngine<'a> {
    pub(super) function: &'a str,
    pub(super) summaries: &'a [BorrowTokenReturnSummary],
    pub(super) diagnostics: Vec<ResourceBorrowDiagnostic>,
    pub(super) deferred: ResourceBorrowCheckDeferred,
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

    pub(super) fn check_ops(
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
            | ResourceOp::RawMemory { .. } => {}
            ResourceOp::Construct {
                output,
                kind,
                inputs,
                ..
            } => {
                construct_function_alias_fields(function_aliases, output, kind, inputs);
            }
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
        let active = borrows.active_state_overlapping(source);
        match (kind, active) {
            (BorrowKind::Shared, Some(active @ BorrowState::Unique { .. })) => {
                self.push_conflict(ResourceBorrowOperation::SharedBorrow, source, active, span);
            }
            (
                BorrowKind::Unique,
                Some(active @ (BorrowState::Shared { .. } | BorrowState::Unique { .. })),
            ) => {
                self.push_conflict(ResourceBorrowOperation::UniqueBorrow, source, active, span);
            }
            (BorrowKind::Shared, _) => borrows.add_shared(source, output),
            (BorrowKind::Unique, _) => borrows.add_unique(source, output),
        }
    }

    fn check_source_read(&mut self, borrows: &BorrowTable, place: &Place, span: Span) {
        if let Some(active @ BorrowState::Unique { .. }) = borrows.unique_state_overlapping(place) {
            self.push_conflict(ResourceBorrowOperation::Read, place, active, span);
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
        match borrows.active_state_overlapping(place) {
            Some(active @ (BorrowState::Shared { .. } | BorrowState::Unique { .. })) => {
                self.push_conflict(operation, place, active, span);
            }
            Some(BorrowState::Unborrowed | BorrowState::Released) | None => {}
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

    pub(super) fn check_ops(
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
                self.report_overwritten_owners(owners, target, value, *span);
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
            } => {
                self.construct_owner_fields(owners, output, kind, inputs, *span);
                construct_function_alias_fields(function_aliases, output, kind, inputs);
            }
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
            let field = construct_aggregate_field_place(output, kind, index, input);
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

    fn report_overwritten_owners(
        &mut self,
        owners: &mut OwnerTable,
        target: &Place,
        value: &Place,
        span: Span,
    ) {
        for entry in owners.live_entries_under(target) {
            if places_overlap(&entry.place, value) {
                continue;
            }
            match entry.state {
                OwnerState::Live { storage } => {
                    self.diagnostics.push(ResourceOwnerDiagnostic::OwnerLeaked {
                        function: String::from(self.function),
                        place: entry.place.clone(),
                        storage,
                        span,
                    });
                }
                OwnerState::MaybeFreed => {
                    self.diagnostics
                        .push(ResourceOwnerDiagnostic::OwnerMaybeLeaked {
                            function: String::from(self.function),
                            place: entry.place.clone(),
                            span,
                        });
                }
                OwnerState::NoFreeObligation | OwnerState::Moved | OwnerState::Freed => {}
            }
            owners.set_state(&entry.place, OwnerState::Moved);
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
