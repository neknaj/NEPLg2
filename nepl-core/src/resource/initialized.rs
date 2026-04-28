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
use super::place_utils::{
    place_suffix_after_prefix, place_with_suffix, push_unique_place, raw_memory_cell_place,
    should_track,
};
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

#[derive(Debug, Clone, Default)]
struct RawCellAddressAliases {
    groups: Vec<Vec<Place>>,
}

impl RawCellAddressAliases {
    fn mark(&mut self, place: &Place) {
        self.clear(place);
        self.union_group(core::slice::from_ref(place));
    }

    fn copy_alias_or_seed(&mut self, source: &Place, target: &Place) {
        if source == target {
            return;
        }
        let groups = self.alias_groups_for(source);
        self.clear(target);
        if groups.is_empty() {
            let mut group = Vec::new();
            push_unique_place(&mut group, source);
            push_unique_place(&mut group, target);
            self.union_group(&group);
            return;
        }
        for mut group in groups {
            push_unique_place(&mut group, target);
            self.union_group(&group);
        }
    }

    fn canonicalize(&self, place: &Place) -> Place {
        for group in &self.groups {
            for alias in group {
                if let Some(suffix) = place_suffix_after_prefix(place, alias) {
                    return place_with_suffix(&group[0], &suffix, place.ty);
                }
            }
        }
        place.clone()
    }

    fn clear(&mut self, place: &Place) {
        for group in &mut self.groups {
            group.retain(|existing| place_suffix_after_prefix(existing, place).is_none());
        }
        self.groups.retain(|group| !group.is_empty());
    }

    fn merge_paths(paths: &[RawCellAddressAliases]) -> Self {
        let mut out = RawCellAddressAliases::default();
        for path in paths {
            for group in &path.groups {
                out.union_group(group);
            }
        }
        out
    }

    fn alias_groups_for(&self, place: &Place) -> Vec<Vec<Place>> {
        let mut out = Vec::new();
        for group in &self.groups {
            let mut mapped = Vec::new();
            for alias in group {
                if let Some(suffix) = place_suffix_after_prefix(place, alias) {
                    for group_alias in group {
                        push_unique_place(
                            &mut mapped,
                            &place_with_suffix(group_alias, &suffix, place.ty),
                        );
                    }
                    break;
                }
            }
            if !mapped.is_empty() {
                out.push(mapped);
            }
        }
        out
    }

    fn union_group(&mut self, group: &[Place]) {
        let mut merged = group.to_vec();
        let mut retained = Vec::new();
        for existing in self.groups.drain(..) {
            if groups_overlap(&existing, &merged) {
                for place in &existing {
                    push_unique_place(&mut merged, place);
                }
            } else {
                retained.push(existing);
            }
        }
        if !merged.is_empty() {
            retained.push(merged);
        }
        self.groups = retained;
    }
}

fn groups_overlap(left: &[Place], right: &[Place]) -> bool {
    left.iter().any(|place| right.contains(place))
}

impl ResourceCheckEngine<'_> {
    fn check_function(&mut self, function: &ResourceFunction) -> Vec<CellStateEntry> {
        let mut cells = CellTable::default();
        let mut raw_aliases = RawCellAddressAliases::default();
        for param in &function.params {
            cells.mark_initialized(&param.place);
            raw_aliases.mark(&param.place);
        }
        for block in &function.blocks {
            self.check_block(&mut cells, &mut raw_aliases, block);
        }
        cells.into_entries()
    }

    fn check_block(
        &mut self,
        cells: &mut CellTable,
        raw_aliases: &mut RawCellAddressAliases,
        block: &ResourceBlock,
    ) {
        self.check_ops(cells, raw_aliases, &block.ops);
        match &block.terminator {
            ResourceTerminator::Return { value, span } => {
                if let Some(value) = value {
                    self.consume_by_value(cells, value, ResourceCheckOperation::ReturnValue, *span);
                }
            }
            ResourceTerminator::Unreachable { .. } | ResourceTerminator::RawBody { .. } => {}
        }
    }

    fn check_ops(
        &mut self,
        cells: &mut CellTable,
        raw_aliases: &mut RawCellAddressAliases,
        ops: &[ResourceOp],
    ) {
        for op in ops {
            self.check_op(cells, raw_aliases, op);
        }
    }

    fn check_op(
        &mut self,
        cells: &mut CellTable,
        raw_aliases: &mut RawCellAddressAliases,
        op: &ResourceOp,
    ) {
        match op {
            ResourceOp::Expr {
                kind,
                output,
                span: _,
                ..
            } => self.check_expr(cells, raw_aliases, *kind, output),
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
                        raw_aliases.copy_alias_or_seed(initializer, place);
                    } else {
                        cells.set_state(place, CellState::Uninit);
                        raw_aliases.clear(place);
                    }
                } else {
                    cells.set_state(place, CellState::Uninit);
                    raw_aliases.clear(place);
                }
            }
            ResourceOp::Read {
                source,
                output,
                span,
            } => {
                if self.consume_by_value(cells, source, ResourceCheckOperation::Read, *span) {
                    cells.mark_initialized(output);
                    raw_aliases.copy_alias_or_seed(source, output);
                }
            }
            ResourceOp::Assign {
                target,
                value,
                span,
            } => {
                if self.consume_by_value(cells, value, ResourceCheckOperation::AssignValue, *span) {
                    cells.mark_initialized(target);
                    raw_aliases.copy_alias_or_seed(value, target);
                } else {
                    raw_aliases.clear(target);
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
                    raw_aliases.clear(output);
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
                    raw_aliases.copy_alias_or_seed(source, output);
                }
            }
            ResourceOp::Drop { place, span } => {
                if self.ensure_available(cells, place, ResourceCheckOperation::Drop, *span) {
                    cells.set_state(place, CellState::Dropped);
                    raw_aliases.clear(place);
                }
            }
            ResourceOp::CallEffect { .. } => {}
            ResourceOp::FunctionValue { output, .. } => {
                cells.mark_initialized(output);
                raw_aliases.clear(output);
            }
            ResourceOp::Call {
                output, args, span, ..
            } => {
                let args_available =
                    self.consume_args(cells, args, ResourceCheckOperation::CallArgument, *span);
                if args_available {
                    cells.mark_initialized(output);
                    raw_aliases.clear(output);
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
                    raw_aliases.clear(output);
                }
            }
            ResourceOp::RawMemory {
                operation,
                output,
                args,
                span,
            } => self.check_raw_memory(cells, raw_aliases, operation, output, args, *span),
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
                    raw_aliases.clear(output);
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
                let mut then_aliases = raw_aliases.clone();
                let mut else_aliases = raw_aliases.clone();
                self.check_ops(&mut then_cells, &mut then_aliases, then_ops);
                self.check_ops(&mut else_cells, &mut else_aliases, else_ops);
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
                *raw_aliases = RawCellAddressAliases::merge_paths(&[then_aliases, else_aliases]);
                if condition_available && then_available && else_available {
                    cells.mark_initialized(output);
                    raw_aliases.clear(output);
                }
            }
            ResourceOp::Loop {
                condition_ops,
                condition,
                body_ops,
                span,
            } => {
                let mut condition_cells = cells.clone();
                let mut condition_aliases = raw_aliases.clone();
                self.check_ops(&mut condition_cells, &mut condition_aliases, condition_ops);
                self.consume_by_value(
                    &mut condition_cells,
                    condition,
                    ResourceCheckOperation::LoopCondition,
                    *span,
                );
                let mut body_cells = condition_cells.clone();
                let mut body_aliases = condition_aliases.clone();
                self.check_ops(&mut body_cells, &mut body_aliases, body_ops);
                *cells = CellTable::merge_paths(&[condition_cells, body_cells]);
                *raw_aliases =
                    RawCellAddressAliases::merge_paths(&[condition_aliases, body_aliases]);
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
                let mut alias_paths = Vec::new();
                for arm in arms {
                    let mut arm_cells = cells.clone();
                    let mut arm_aliases = raw_aliases.clone();
                    if let Some(bind_local) = &arm.bind_local {
                        arm_cells.mark_initialized(bind_local);
                        arm_aliases.clear(bind_local);
                    }
                    self.check_ops(&mut arm_cells, &mut arm_aliases, &arm.ops);
                    arms_available &= self.consume_by_value(
                        &mut arm_cells,
                        &arm.value,
                        ResourceCheckOperation::MatchValue,
                        arm.span,
                    );
                    arm_paths.push(arm_cells);
                    alias_paths.push(arm_aliases);
                }
                if !arm_paths.is_empty() {
                    *cells = CellTable::merge_paths(&arm_paths);
                    *raw_aliases = RawCellAddressAliases::merge_paths(&alias_paths);
                }
                if scrutinee_available && arms_available {
                    cells.mark_initialized(output);
                    raw_aliases.clear(output);
                }
            }
        }
    }

    fn check_expr(
        &mut self,
        cells: &mut CellTable,
        raw_aliases: &mut RawCellAddressAliases,
        kind: ResourceExprKind,
        output: &Place,
    ) {
        match kind {
            ResourceExprKind::Literal
            | ResourceExprKind::Block
            | ResourceExprKind::Let
            | ResourceExprKind::Set
            | ResourceExprKind::Intrinsic
            | ResourceExprKind::Deref
            | ResourceExprKind::Drop
            | ResourceExprKind::Loop => cells.mark_initialized(output),
            ResourceExprKind::LocalRead
            | ResourceExprKind::FunctionValue
            | ResourceExprKind::Call
            | ResourceExprKind::IndirectCall
            | ResourceExprKind::Branch
            | ResourceExprKind::Match
            | ResourceExprKind::Construct
            | ResourceExprKind::Borrow => {}
        }
        if !matches!(kind, ResourceExprKind::LocalRead) {
            raw_aliases.clear(output);
        }
    }

    fn check_raw_memory(
        &mut self,
        cells: &mut CellTable,
        raw_aliases: &mut RawCellAddressAliases,
        operation: &RawMemoryOp,
        output: &Place,
        args: &[Place],
        span: Span,
    ) {
        match operation {
            RawMemoryOp::Alloc => {
                let args_available =
                    self.ensure_args(cells, args, ResourceCheckOperation::RawMemoryArgument, span);
                if args_available {
                    cells.mark_initialized(output);
                    raw_aliases.mark(output);
                }
            }
            RawMemoryOp::Load => {
                let Some(address) = args.first() else {
                    cells.mark_initialized(output);
                    raw_aliases.clear(output);
                    return;
                };
                let address = raw_aliases.canonicalize(address);
                let address_available = self.ensure_available(
                    cells,
                    &address,
                    ResourceCheckOperation::RawMemoryLoadAddress,
                    span,
                );
                let cell = raw_memory_cell_place(&address, output.ty);
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
                    raw_aliases.clear(output);
                }
            }
            RawMemoryOp::Store => {
                let Some(address) = args.first() else {
                    cells.mark_initialized(output);
                    raw_aliases.clear(output);
                    return;
                };
                let address = raw_aliases.canonicalize(address);
                let address_available = self.ensure_available(
                    cells,
                    &address,
                    ResourceCheckOperation::RawMemoryStoreAddress,
                    span,
                );
                let cell_available = self.ensure_no_live_non_copy_raw_cells(
                    cells,
                    &address,
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
                        let cell = raw_memory_cell_place(&address, value.ty);
                        cells.clear_raw_cells_under(&address);
                        cells.mark_initialized(&cell);
                    }
                    cells.mark_initialized(output);
                    raw_aliases.clear(output);
                }
            }
            RawMemoryOp::Dealloc => {
                let Some(address) = args.first() else {
                    cells.mark_initialized(output);
                    raw_aliases.clear(output);
                    return;
                };
                let address = raw_aliases.canonicalize(address);
                let address_available = self.ensure_available(
                    cells,
                    &address,
                    ResourceCheckOperation::RawMemoryDeallocAddress,
                    span,
                );
                let cells_released = self.ensure_no_live_non_copy_raw_cells(
                    cells,
                    &address,
                    ResourceCheckOperation::RawMemoryDeallocCell,
                    span,
                );
                if address_available && cells_released {
                    cells.clear_raw_cells_under(&address);
                    cells.mark_initialized(output);
                    raw_aliases.clear(output);
                }
            }
            RawMemoryOp::Realloc => {
                let Some(address) = args.first() else {
                    cells.mark_initialized(output);
                    raw_aliases.clear(output);
                    return;
                };
                let address = raw_aliases.canonicalize(address);
                let address_available = self.ensure_available(
                    cells,
                    &address,
                    ResourceCheckOperation::RawMemoryReallocAddress,
                    span,
                );
                let cells_released = self.ensure_no_live_non_copy_raw_cells(
                    cells,
                    &address,
                    ResourceCheckOperation::RawMemoryReallocCell,
                    span,
                );
                if address_available && cells_released {
                    let relocated =
                        cells.copy_initialized_copy_raw_cells(&address, output, self.types);
                    cells.clear_raw_cells_under(&address);
                    cells.mark_initialized(output);
                    cells.extend_entries(relocated);
                    raw_aliases.clear(&address);
                    raw_aliases.mark(output);
                }
            }
            RawMemoryOp::Fill => {
                let Some(address) = args.first() else {
                    cells.mark_initialized(output);
                    raw_aliases.clear(output);
                    return;
                };
                let address = raw_aliases.canonicalize(address);
                let address_available = self.ensure_available(
                    cells,
                    &address,
                    ResourceCheckOperation::RawMemoryFillAddress,
                    span,
                );
                let cells_released = self.ensure_no_live_non_copy_raw_cells(
                    cells,
                    &address,
                    ResourceCheckOperation::RawMemoryFillCell,
                    span,
                );
                if address_available && cells_released {
                    cells.clear_raw_cells_under(&address);
                    cells.mark_initialized(output);
                    raw_aliases.clear(output);
                }
            }
            RawMemoryOp::BulkCopy | RawMemoryOp::BulkMove => {
                let Some(destination) = args.first() else {
                    cells.mark_initialized(output);
                    raw_aliases.clear(output);
                    return;
                };
                let Some(source) = args.get(1) else {
                    cells.mark_initialized(output);
                    raw_aliases.clear(output);
                    return;
                };
                let destination = raw_aliases.canonicalize(destination);
                let source = raw_aliases.canonicalize(source);
                let destination_available = self.ensure_available(
                    cells,
                    &destination,
                    ResourceCheckOperation::RawMemoryBulkDestinationAddress,
                    span,
                );
                let source_available = self.ensure_available(
                    cells,
                    &source,
                    ResourceCheckOperation::RawMemoryBulkSourceAddress,
                    span,
                );
                let destination_cells_released = self.ensure_no_live_non_copy_raw_cells(
                    cells,
                    &destination,
                    ResourceCheckOperation::RawMemoryBulkDestinationCell,
                    span,
                );
                let source_cells_copyable = self.ensure_no_live_non_copy_raw_cells(
                    cells,
                    &source,
                    ResourceCheckOperation::RawMemoryBulkSourceCell,
                    span,
                );
                if destination_available
                    && source_available
                    && destination_cells_released
                    && source_cells_copyable
                {
                    let copied =
                        cells.copy_initialized_copy_raw_cells(&source, &destination, self.types);
                    cells.clear_raw_cells_under(&destination);
                    cells.extend_entries(copied);
                    cells.mark_initialized(output);
                    raw_aliases.clear(output);
                }
            }
            _ => {
                let args_available =
                    self.ensure_args(cells, args, ResourceCheckOperation::RawMemoryArgument, span);
                if args_available {
                    cells.mark_initialized(output);
                    raw_aliases.clear(output);
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
