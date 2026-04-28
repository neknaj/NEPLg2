extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;

use crate::span::Span;
use crate::types::TypeCtx;

use super::cell_state::CellTable;
use super::function_alias::{construct_function_alias_fields, FunctionAliasTable};
use super::model::{
    AggregateKind, CellState, CellStateEntry, EffectOp, Place, RawMemoryOp, ResourceBlock,
    ResourceCallTarget, ResourceExprKind, ResourceFunction, ResourceModule, ResourceOp,
    ResourceTerminator,
};
use super::place_utils::{
    construct_aggregate_field_place, place_suffix_after_prefix, place_with_suffix,
    push_unique_place, raw_memory_cell_place, replace_place_prefix, should_track,
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
    let raw_alias_summaries = compute_raw_cell_address_return_summaries(module);

    for function in &module.functions {
        let mut engine = ResourceCheckEngine {
            function: function.name.as_str(),
            types,
            raw_alias_summaries: &raw_alias_summaries,
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
    raw_alias_summaries: &'a [RawCellAddressReturnSummary],
    diagnostics: Vec<ResourceCheckDiagnostic>,
    deferred: ResourceCheckDeferred,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RawCellAddressReturnSummary {
    function: String,
    parameter_indices: Vec<usize>,
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
        let groups = self.groups_with_replaced_prefix_or_singleton(source, target);
        self.clear(target);
        for group in groups {
            self.union_group(&group);
        }
    }

    fn aliases(&self, left: &Place, right: &Place) -> bool {
        self.alias_groups_for(left)
            .iter()
            .any(|group| group.iter().any(|place| place == right))
            || self
                .alias_groups_for(right)
                .iter()
                .any(|group| group.iter().any(|place| place == left))
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

    fn groups_with_replaced_prefix_or_singleton(
        &self,
        source: &Place,
        target: &Place,
    ) -> Vec<Vec<Place>> {
        let mut out = Vec::new();
        for group in &self.groups {
            let mut mapped = Vec::new();
            let mut mapped_descendant = false;
            for place in group {
                if let Some(replacement) = replace_place_prefix(place, source, target) {
                    if place.projections.len() > source.projections.len() {
                        mapped_descendant = true;
                    }
                    push_unique_place(&mut mapped, &replacement);
                }
            }
            if mapped.is_empty() {
                continue;
            }

            let mut merged: Vec<Place> = group
                .iter()
                .filter(|place| place_suffix_after_prefix(place, target).is_none())
                .cloned()
                .collect();
            for place in &mapped {
                push_unique_place(&mut merged, place);
            }
            if mapped_descendant {
                push_unique_place(&mut merged, target);
            }
            out.push(merged);
        }

        if out.is_empty() {
            let mut group = Vec::new();
            push_unique_place(&mut group, source);
            push_unique_place(&mut group, target);
            out.push(group);
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

fn compute_raw_cell_address_return_summaries(
    module: &ResourceModule,
) -> Vec<RawCellAddressReturnSummary> {
    let mut summaries = Vec::new();
    for _ in 0..=module.functions.len() {
        let mut next = Vec::new();
        for function in &module.functions {
            let mut parameter_indices = Vec::new();
            for (index, param) in function.params.iter().enumerate() {
                if function_returns_raw_cell_address_alias(function, &param.place, &summaries) {
                    parameter_indices.push(index);
                }
            }
            if !parameter_indices.is_empty() {
                next.push(RawCellAddressReturnSummary {
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

fn function_returns_raw_cell_address_alias(
    function: &ResourceFunction,
    parameter: &Place,
    summaries: &[RawCellAddressReturnSummary],
) -> bool {
    let mut raw_aliases = RawCellAddressAliases::default();
    let mut function_aliases = FunctionAliasTable::default();
    raw_aliases.mark(parameter);
    for block in &function.blocks {
        propagate_raw_address_alias_ops(
            &mut raw_aliases,
            &mut function_aliases,
            &block.ops,
            summaries,
        );
        if let ResourceTerminator::Return {
            value: Some(value), ..
        } = &block.terminator
        {
            if raw_aliases.aliases(value, parameter) {
                return true;
            }
        }
    }
    false
}

fn propagate_raw_address_alias_ops(
    raw_aliases: &mut RawCellAddressAliases,
    function_aliases: &mut FunctionAliasTable,
    ops: &[ResourceOp],
    summaries: &[RawCellAddressReturnSummary],
) {
    for op in ops {
        propagate_raw_address_alias_op(raw_aliases, function_aliases, op, summaries);
    }
}

fn propagate_raw_address_alias_op(
    raw_aliases: &mut RawCellAddressAliases,
    function_aliases: &mut FunctionAliasTable,
    op: &ResourceOp,
    summaries: &[RawCellAddressReturnSummary],
) {
    match op {
        ResourceOp::DeclareLocal {
            place, initializer, ..
        } => {
            if let Some(initializer) = initializer {
                raw_aliases.copy_alias_or_seed(initializer, place);
                function_aliases.copy_alias(initializer, place);
            } else {
                raw_aliases.clear(place);
            }
        }
        ResourceOp::Read { source, output, .. } | ResourceOp::Move { source, output, .. } => {
            raw_aliases.copy_alias_or_seed(source, output);
            function_aliases.copy_alias(source, output);
        }
        ResourceOp::Assign { target, value, .. } => {
            raw_aliases.copy_alias_or_seed(value, target);
            function_aliases.copy_alias(value, target);
        }
        ResourceOp::RawMemory {
            operation, output, ..
        } => match operation {
            RawMemoryOp::Alloc | RawMemoryOp::Realloc => raw_aliases.mark(output),
            RawMemoryOp::Load
            | RawMemoryOp::Store
            | RawMemoryOp::Dealloc
            | RawMemoryOp::BulkCopy
            | RawMemoryOp::BulkMove
            | RawMemoryOp::MemorySize
            | RawMemoryOp::MemoryGrow
            | RawMemoryOp::Fill
            | RawMemoryOp::Other { .. } => {}
        },
        ResourceOp::Construct {
            output,
            kind,
            inputs,
            ..
        } => {
            raw_aliases.clear(output);
            construct_raw_cell_address_alias_fields(raw_aliases, output, kind, inputs);
            construct_function_alias_fields(function_aliases, output, kind, inputs);
        }
        ResourceOp::FunctionValue { output, name, .. } => {
            function_aliases.set_alias(output, name.clone());
        }
        ResourceOp::Call {
            output,
            target,
            args,
            effect,
            ..
        } => {
            if !matches!(
                effect,
                EffectOp::InternalAlloc | EffectOp::UnsafeMemory { .. }
            ) && !apply_direct_call_raw_alias_summary(
                raw_aliases,
                output,
                target,
                args,
                summaries,
            ) {
                raw_aliases.clear(output);
            }
        }
        ResourceOp::IndirectCall {
            output,
            callee,
            args,
            ..
        } => {
            if !apply_indirect_call_raw_alias_summary(
                raw_aliases,
                function_aliases,
                output,
                callee,
                args,
                summaries,
            ) {
                raw_aliases.clear(output);
            }
        }
        ResourceOp::Branch {
            output,
            then_ops,
            then_value,
            else_ops,
            else_value,
            ..
        } => {
            let mut then_aliases = raw_aliases.clone();
            let mut else_aliases = raw_aliases.clone();
            let mut then_function_aliases = function_aliases.clone();
            let mut else_function_aliases = function_aliases.clone();
            propagate_raw_address_alias_ops(
                &mut then_aliases,
                &mut then_function_aliases,
                then_ops,
                summaries,
            );
            propagate_raw_address_alias_ops(
                &mut else_aliases,
                &mut else_function_aliases,
                else_ops,
                summaries,
            );
            then_aliases.copy_alias_or_seed(then_value, output);
            else_aliases.copy_alias_or_seed(else_value, output);
            then_function_aliases.copy_alias(then_value, output);
            else_function_aliases.copy_alias(else_value, output);
            *raw_aliases = RawCellAddressAliases::merge_paths(&[then_aliases, else_aliases]);
            *function_aliases =
                FunctionAliasTable::merge_paths(&[then_function_aliases, else_function_aliases]);
        }
        ResourceOp::Loop {
            condition_ops,
            body_ops,
            ..
        } => {
            let mut condition_aliases = raw_aliases.clone();
            let mut condition_function_aliases = function_aliases.clone();
            propagate_raw_address_alias_ops(
                &mut condition_aliases,
                &mut condition_function_aliases,
                condition_ops,
                summaries,
            );
            let mut body_aliases = condition_aliases.clone();
            let mut body_function_aliases = condition_function_aliases.clone();
            propagate_raw_address_alias_ops(
                &mut body_aliases,
                &mut body_function_aliases,
                body_ops,
                summaries,
            );
            *raw_aliases = RawCellAddressAliases::merge_paths(&[condition_aliases, body_aliases]);
            *function_aliases = FunctionAliasTable::merge_paths(&[
                condition_function_aliases,
                body_function_aliases,
            ]);
        }
        ResourceOp::Match { output, arms, .. } => {
            let mut alias_paths = Vec::new();
            let mut function_alias_paths = Vec::new();
            for arm in arms {
                let mut arm_aliases = raw_aliases.clone();
                let mut arm_function_aliases = function_aliases.clone();
                if let Some(bind_local) = &arm.bind_local {
                    arm_aliases.clear(bind_local);
                }
                propagate_raw_address_alias_ops(
                    &mut arm_aliases,
                    &mut arm_function_aliases,
                    &arm.ops,
                    summaries,
                );
                arm_aliases.copy_alias_or_seed(&arm.value, output);
                arm_function_aliases.copy_alias(&arm.value, output);
                alias_paths.push(arm_aliases);
                function_alias_paths.push(arm_function_aliases);
            }
            if !alias_paths.is_empty() {
                *raw_aliases = RawCellAddressAliases::merge_paths(&alias_paths);
                *function_aliases = FunctionAliasTable::merge_paths(&function_alias_paths);
            }
        }
        ResourceOp::Expr { output, kind, .. } => {
            if !matches!(kind, ResourceExprKind::LocalRead) {
                raw_aliases.clear(output);
            }
        }
        ResourceOp::Borrow { output, .. } => raw_aliases.clear(output),
        ResourceOp::Drop { place, .. } => raw_aliases.clear(place),
        ResourceOp::CallEffect { .. } => {}
    }
}

fn construct_raw_cell_address_alias_fields(
    raw_aliases: &mut RawCellAddressAliases,
    output: &Place,
    kind: &AggregateKind,
    inputs: &[Place],
) {
    for (index, input) in inputs.iter().enumerate() {
        let field = construct_aggregate_field_place(output, kind, index, input);
        raw_aliases.copy_alias_or_seed(input, &field);
    }
}

fn apply_direct_call_raw_alias_summary(
    raw_aliases: &mut RawCellAddressAliases,
    output: &Place,
    target: &ResourceCallTarget,
    args: &[Place],
    summaries: &[RawCellAddressReturnSummary],
) -> bool {
    let ResourceCallTarget::User { name, .. } = target else {
        return false;
    };
    let Some(summary) = summaries
        .iter()
        .find(|summary| summary.function == name.as_str())
    else {
        return false;
    };
    apply_raw_alias_summary(raw_aliases, output, args, summary)
}

fn apply_indirect_call_raw_alias_summary(
    raw_aliases: &mut RawCellAddressAliases,
    function_aliases: &FunctionAliasTable,
    output: &Place,
    callee: &Place,
    args: &[Place],
    summaries: &[RawCellAddressReturnSummary],
) -> bool {
    let functions = function_aliases.functions(callee);
    let mut applied = false;
    for function in functions {
        if let Some(summary) = summaries
            .iter()
            .find(|summary| summary.function == function.as_str())
        {
            applied |= apply_raw_alias_summary(raw_aliases, output, args, summary);
        }
    }
    applied
}

fn apply_raw_alias_summary(
    raw_aliases: &mut RawCellAddressAliases,
    output: &Place,
    args: &[Place],
    summary: &RawCellAddressReturnSummary,
) -> bool {
    let mut applied = false;
    for arg in summary
        .parameter_indices
        .iter()
        .filter_map(|index| args.get(*index))
    {
        raw_aliases.copy_alias_or_seed(arg, output);
        applied = true;
    }
    applied
}

impl ResourceCheckEngine<'_> {
    fn check_function(&mut self, function: &ResourceFunction) -> Vec<CellStateEntry> {
        let mut cells = CellTable::default();
        let mut raw_aliases = RawCellAddressAliases::default();
        let mut function_aliases = FunctionAliasTable::default();
        for param in &function.params {
            cells.mark_initialized(&param.place);
            raw_aliases.mark(&param.place);
        }
        for block in &function.blocks {
            self.check_block(&mut cells, &mut raw_aliases, &mut function_aliases, block);
        }
        cells.into_entries()
    }

    fn check_block(
        &mut self,
        cells: &mut CellTable,
        raw_aliases: &mut RawCellAddressAliases,
        function_aliases: &mut FunctionAliasTable,
        block: &ResourceBlock,
    ) {
        self.check_ops(cells, raw_aliases, function_aliases, &block.ops);
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
        function_aliases: &mut FunctionAliasTable,
        ops: &[ResourceOp],
    ) {
        for op in ops {
            self.check_op(cells, raw_aliases, function_aliases, op);
        }
    }

    fn check_op(
        &mut self,
        cells: &mut CellTable,
        raw_aliases: &mut RawCellAddressAliases,
        function_aliases: &mut FunctionAliasTable,
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
                        function_aliases.copy_alias(initializer, place);
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
                    function_aliases.copy_alias(source, output);
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
                    function_aliases.copy_alias(value, target);
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
                    function_aliases.copy_alias(source, output);
                }
            }
            ResourceOp::Drop { place, span } => {
                if self.ensure_available(cells, place, ResourceCheckOperation::Drop, *span) {
                    cells.set_state(place, CellState::Dropped);
                    raw_aliases.clear(place);
                }
            }
            ResourceOp::CallEffect { .. } => {}
            ResourceOp::FunctionValue { output, name, .. } => {
                cells.mark_initialized(output);
                raw_aliases.clear(output);
                function_aliases.set_alias(output, name.clone());
            }
            ResourceOp::Call {
                output,
                target,
                args,
                effect,
                span,
                ..
            } => {
                if matches!(
                    effect,
                    EffectOp::InternalAlloc | EffectOp::UnsafeMemory { .. }
                ) {
                    return;
                }
                let args_available =
                    self.consume_args(cells, args, ResourceCheckOperation::CallArgument, *span);
                if args_available {
                    cells.mark_initialized(output);
                    if !self.apply_call_return_raw_alias(raw_aliases, output, target, args) {
                        raw_aliases.clear(output);
                    }
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
                    if !self.apply_indirect_call_return_raw_alias(
                        raw_aliases,
                        function_aliases,
                        output,
                        callee,
                        args,
                    ) {
                        raw_aliases.clear(output);
                    }
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
                kind,
                inputs,
                span,
                ..
            } => {
                let inputs_available =
                    self.consume_args(cells, inputs, ResourceCheckOperation::ConstructInput, *span);
                if inputs_available {
                    cells.mark_initialized(output);
                    raw_aliases.clear(output);
                    construct_raw_cell_address_alias_fields(raw_aliases, output, kind, inputs);
                    construct_function_alias_fields(function_aliases, output, kind, inputs);
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
                let mut then_function_aliases = function_aliases.clone();
                let mut else_function_aliases = function_aliases.clone();
                self.check_ops(
                    &mut then_cells,
                    &mut then_aliases,
                    &mut then_function_aliases,
                    then_ops,
                );
                self.check_ops(
                    &mut else_cells,
                    &mut else_aliases,
                    &mut else_function_aliases,
                    else_ops,
                );
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
                if then_available {
                    then_aliases.copy_alias_or_seed(then_value, output);
                    then_function_aliases.copy_alias(then_value, output);
                }
                if else_available {
                    else_aliases.copy_alias_or_seed(else_value, output);
                    else_function_aliases.copy_alias(else_value, output);
                }
                *cells = CellTable::merge_paths(&[then_cells, else_cells]);
                *raw_aliases = RawCellAddressAliases::merge_paths(&[then_aliases, else_aliases]);
                *function_aliases = FunctionAliasTable::merge_paths(&[
                    then_function_aliases,
                    else_function_aliases,
                ]);
                if condition_available && then_available && else_available {
                    cells.mark_initialized(output);
                } else {
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
                let mut condition_function_aliases = function_aliases.clone();
                self.check_ops(
                    &mut condition_cells,
                    &mut condition_aliases,
                    &mut condition_function_aliases,
                    condition_ops,
                );
                self.consume_by_value(
                    &mut condition_cells,
                    condition,
                    ResourceCheckOperation::LoopCondition,
                    *span,
                );
                let mut body_cells = condition_cells.clone();
                let mut body_aliases = condition_aliases.clone();
                let mut body_function_aliases = condition_function_aliases.clone();
                self.check_ops(
                    &mut body_cells,
                    &mut body_aliases,
                    &mut body_function_aliases,
                    body_ops,
                );
                *cells = CellTable::merge_paths(&[condition_cells, body_cells]);
                *raw_aliases =
                    RawCellAddressAliases::merge_paths(&[condition_aliases, body_aliases]);
                *function_aliases = FunctionAliasTable::merge_paths(&[
                    condition_function_aliases,
                    body_function_aliases,
                ]);
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
                let mut function_alias_paths = Vec::new();
                for arm in arms {
                    let mut arm_cells = cells.clone();
                    let mut arm_aliases = raw_aliases.clone();
                    let mut arm_function_aliases = function_aliases.clone();
                    if let Some(bind_local) = &arm.bind_local {
                        arm_cells.mark_initialized(bind_local);
                        arm_aliases.clear(bind_local);
                    }
                    self.check_ops(
                        &mut arm_cells,
                        &mut arm_aliases,
                        &mut arm_function_aliases,
                        &arm.ops,
                    );
                    let arm_available = self.consume_by_value(
                        &mut arm_cells,
                        &arm.value,
                        ResourceCheckOperation::MatchValue,
                        arm.span,
                    );
                    arms_available &= arm_available;
                    if arm_available {
                        arm_aliases.copy_alias_or_seed(&arm.value, output);
                        arm_function_aliases.copy_alias(&arm.value, output);
                    }
                    arm_paths.push(arm_cells);
                    alias_paths.push(arm_aliases);
                    function_alias_paths.push(arm_function_aliases);
                }
                if !arm_paths.is_empty() {
                    *cells = CellTable::merge_paths(&arm_paths);
                    *raw_aliases = RawCellAddressAliases::merge_paths(&alias_paths);
                    *function_aliases = FunctionAliasTable::merge_paths(&function_alias_paths);
                }
                if scrutinee_available && arms_available {
                    cells.mark_initialized(output);
                } else {
                    raw_aliases.clear(output);
                }
            }
        }
    }

    fn apply_call_return_raw_alias(
        &self,
        raw_aliases: &mut RawCellAddressAliases,
        output: &Place,
        target: &ResourceCallTarget,
        args: &[Place],
    ) -> bool {
        apply_direct_call_raw_alias_summary(
            raw_aliases,
            output,
            target,
            args,
            self.raw_alias_summaries,
        )
    }

    fn apply_indirect_call_return_raw_alias(
        &self,
        raw_aliases: &mut RawCellAddressAliases,
        function_aliases: &FunctionAliasTable,
        output: &Place,
        callee: &Place,
        args: &[Place],
    ) -> bool {
        apply_indirect_call_raw_alias_summary(
            raw_aliases,
            function_aliases,
            output,
            callee,
            args,
            self.raw_alias_summaries,
        )
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
