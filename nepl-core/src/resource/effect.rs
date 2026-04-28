extern crate alloc;

use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

use crate::ast::Effect;
use crate::span::Span;

use super::model::{
    EffectOp, Place, RawMemoryOp, ResourceBlock, ResourceCallTarget, ResourceFunction,
    ResourceModule, ResourceOp, ResourceTerminator,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceEffectBoundaryReport {
    pub functions: Vec<ResourceEffectFunctionCheck>,
    pub diagnostics: Vec<ResourceEffectBoundaryDiagnostic>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceEffectFunctionCheck {
    pub name: String,
    pub counts: ResourceEffectCounts,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ResourceEffectCounts {
    pub internal_allocs: usize,
    pub unsafe_memory_ops: usize,
    pub external_io_ops: usize,
    pub nondet_ops: usize,
    pub unknown_ops: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResourceEffectBoundaryDiagnostic {
    UnsafeMemoryInPureFunction {
        function: String,
        operation: String,
        span: Span,
    },
    RawAddressEscapeFromInternalAlloc {
        function: String,
        place: Place,
        span: Span,
    },
}

pub fn check_resource_effect_boundaries(module: &ResourceModule) -> ResourceEffectBoundaryReport {
    let mut functions = Vec::new();
    let mut diagnostics = Vec::new();
    let pointer_summaries = compute_raw_pointer_return_summaries(module);
    let summaries = compute_raw_identity_return_summaries(module, &pointer_summaries);

    for function in &module.functions {
        let mut engine = ResourceEffectBoundaryEngine {
            function: function.name.as_str(),
            effect: function.effect,
            summaries: &summaries,
            pointer_summaries: &pointer_summaries,
            track_alloc_identities: true,
            diagnostics: Vec::new(),
            counts: ResourceEffectCounts::default(),
        };
        engine.check_function(function);
        diagnostics.extend(engine.diagnostics);
        functions.push(ResourceEffectFunctionCheck {
            name: function.name.clone(),
            counts: engine.counts,
        });
    }

    ResourceEffectBoundaryReport {
        functions,
        diagnostics,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RawIdentityReturnSummary {
    function: String,
    parameter_indices: Vec<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RawPointerReturnSummary {
    function: String,
    parameter_indices: Vec<usize>,
}

struct ResourceEffectBoundaryEngine<'a> {
    function: &'a str,
    effect: Effect,
    summaries: &'a [RawIdentityReturnSummary],
    pointer_summaries: &'a [RawPointerReturnSummary],
    track_alloc_identities: bool,
    diagnostics: Vec<ResourceEffectBoundaryDiagnostic>,
    counts: ResourceEffectCounts,
}

impl ResourceEffectBoundaryEngine<'_> {
    fn check_function(&mut self, function: &ResourceFunction) {
        let mut identities = RawIdentityTable::default();
        let mut pointer_aliases = RawPointerAliasTable::default();
        let mut function_aliases = FunctionAliasTable::default();
        let mut raw_memory_identities = RawMemoryIdentityTable::default();
        for block in &function.blocks {
            self.check_block(
                &mut identities,
                &mut pointer_aliases,
                &mut function_aliases,
                &mut raw_memory_identities,
                block,
            );
        }
    }

    fn check_block(
        &mut self,
        identities: &mut RawIdentityTable,
        pointer_aliases: &mut RawPointerAliasTable,
        function_aliases: &mut FunctionAliasTable,
        raw_memory_identities: &mut RawMemoryIdentityTable,
        block: &ResourceBlock,
    ) {
        self.check_ops(
            identities,
            pointer_aliases,
            function_aliases,
            raw_memory_identities,
            &block.ops,
        );
        match &block.terminator {
            ResourceTerminator::Return { value, span } => {
                if matches!(self.effect, Effect::Pure) {
                    if let Some(place) = value {
                        if identities.contains(place) {
                            self.diagnostics.push(
                                ResourceEffectBoundaryDiagnostic::RawAddressEscapeFromInternalAlloc {
                                    function: String::from(self.function),
                                    place: place.clone(),
                                    span: *span,
                                },
                            );
                        }
                    }
                }
            }
            ResourceTerminator::Unreachable { .. } | ResourceTerminator::RawBody { .. } => {}
        }
    }

    fn check_ops(
        &mut self,
        identities: &mut RawIdentityTable,
        pointer_aliases: &mut RawPointerAliasTable,
        function_aliases: &mut FunctionAliasTable,
        raw_memory_identities: &mut RawMemoryIdentityTable,
        ops: &[ResourceOp],
    ) {
        for op in ops {
            self.check_op(
                identities,
                pointer_aliases,
                function_aliases,
                raw_memory_identities,
                op,
            );
        }
    }

    fn check_op(
        &mut self,
        identities: &mut RawIdentityTable,
        pointer_aliases: &mut RawPointerAliasTable,
        function_aliases: &mut FunctionAliasTable,
        raw_memory_identities: &mut RawMemoryIdentityTable,
        op: &ResourceOp,
    ) {
        match op {
            ResourceOp::CallEffect { effect, span } => self.check_effect(effect, *span),
            ResourceOp::RawMemory {
                operation,
                output,
                args,
                ..
            } => {
                if self.track_alloc_identities && raw_memory_op_produces_identity(operation) {
                    identities.mark(output);
                }
                if raw_memory_op_produces_identity(operation) {
                    pointer_aliases.mark(output);
                }
                self.apply_raw_memory_identity_effect(
                    identities,
                    pointer_aliases,
                    raw_memory_identities,
                    operation,
                    output,
                    args,
                );
            }
            ResourceOp::DeclareLocal {
                place, initializer, ..
            } => {
                if let Some(initializer) = initializer {
                    identities.copy_identity(initializer, place);
                    copy_pointer_alias(pointer_aliases, raw_memory_identities, initializer, place);
                    function_aliases.copy_alias(initializer, place);
                }
            }
            ResourceOp::Read { source, output, .. } | ResourceOp::Move { source, output, .. } => {
                identities.copy_identity(source, output);
                copy_pointer_alias(pointer_aliases, raw_memory_identities, source, output);
                function_aliases.copy_alias(source, output);
            }
            ResourceOp::Assign { target, value, .. } => {
                identities.copy_identity(value, target);
                copy_pointer_alias(pointer_aliases, raw_memory_identities, value, target);
                function_aliases.copy_alias(value, target);
            }
            ResourceOp::Construct { output, inputs, .. } => {
                identities.clear(output);
                for input in inputs {
                    identities.merge_identity(input, output);
                }
            }
            ResourceOp::Call {
                output,
                target,
                args,
                ..
            } => {
                self.copy_call_return_identity(identities, output, target, args);
                self.copy_call_return_pointer_alias(
                    pointer_aliases,
                    raw_memory_identities,
                    output,
                    target,
                    args,
                );
            }
            ResourceOp::IndirectCall {
                output,
                callee,
                args,
                ..
            } => {
                self.copy_indirect_call_return_identity(
                    identities,
                    function_aliases,
                    output,
                    callee,
                    args,
                );
                self.copy_indirect_call_return_pointer_alias(
                    pointer_aliases,
                    raw_memory_identities,
                    output,
                    callee,
                    args,
                    function_aliases,
                );
            }
            ResourceOp::Branch {
                output,
                then_ops,
                then_value,
                else_ops,
                else_value,
                ..
            } => {
                let mut then_identities = identities.clone();
                let mut else_identities = identities.clone();
                let mut then_pointer_aliases = pointer_aliases.clone();
                let mut else_pointer_aliases = pointer_aliases.clone();
                let mut then_function_aliases = function_aliases.clone();
                let mut else_function_aliases = function_aliases.clone();
                let mut then_raw_memory_identities = raw_memory_identities.clone();
                let mut else_raw_memory_identities = raw_memory_identities.clone();
                self.check_ops(
                    &mut then_identities,
                    &mut then_pointer_aliases,
                    &mut then_function_aliases,
                    &mut then_raw_memory_identities,
                    then_ops,
                );
                self.check_ops(
                    &mut else_identities,
                    &mut else_pointer_aliases,
                    &mut else_function_aliases,
                    &mut else_raw_memory_identities,
                    else_ops,
                );
                then_identities.copy_identity(then_value, output);
                else_identities.copy_identity(else_value, output);
                copy_pointer_alias(
                    &mut then_pointer_aliases,
                    &mut then_raw_memory_identities,
                    then_value,
                    output,
                );
                copy_pointer_alias(
                    &mut else_pointer_aliases,
                    &mut else_raw_memory_identities,
                    else_value,
                    output,
                );
                then_function_aliases.copy_alias(then_value, output);
                else_function_aliases.copy_alias(else_value, output);
                *identities = RawIdentityTable::merge_paths(&[then_identities, else_identities]);
                *pointer_aliases = RawPointerAliasTable::merge_paths(&[
                    then_pointer_aliases,
                    else_pointer_aliases,
                ]);
                *function_aliases = FunctionAliasTable::merge_paths(&[
                    then_function_aliases,
                    else_function_aliases,
                ]);
                *raw_memory_identities = RawMemoryIdentityTable::merge_paths(&[
                    then_raw_memory_identities,
                    else_raw_memory_identities,
                ]);
            }
            ResourceOp::Loop {
                condition_ops,
                body_ops,
                ..
            } => {
                let mut condition_identities = identities.clone();
                let mut condition_pointer_aliases = pointer_aliases.clone();
                let mut condition_function_aliases = function_aliases.clone();
                let mut condition_raw_memory_identities = raw_memory_identities.clone();
                self.check_ops(
                    &mut condition_identities,
                    &mut condition_pointer_aliases,
                    &mut condition_function_aliases,
                    &mut condition_raw_memory_identities,
                    condition_ops,
                );
                let mut body_identities = condition_identities.clone();
                let mut body_pointer_aliases = condition_pointer_aliases.clone();
                let mut body_function_aliases = condition_function_aliases.clone();
                let mut body_raw_memory_identities = condition_raw_memory_identities.clone();
                self.check_ops(
                    &mut body_identities,
                    &mut body_pointer_aliases,
                    &mut body_function_aliases,
                    &mut body_raw_memory_identities,
                    body_ops,
                );
                *identities =
                    RawIdentityTable::merge_paths(&[condition_identities, body_identities]);
                *pointer_aliases = RawPointerAliasTable::merge_paths(&[
                    condition_pointer_aliases,
                    body_pointer_aliases,
                ]);
                *function_aliases = FunctionAliasTable::merge_paths(&[
                    condition_function_aliases,
                    body_function_aliases,
                ]);
                *raw_memory_identities = RawMemoryIdentityTable::merge_paths(&[
                    condition_raw_memory_identities,
                    body_raw_memory_identities,
                ]);
            }
            ResourceOp::Match { output, arms, .. } => {
                let mut arm_paths = Vec::new();
                let mut pointer_alias_paths = Vec::new();
                let mut function_alias_paths = Vec::new();
                let mut raw_memory_identity_paths = Vec::new();
                for arm in arms {
                    let mut arm_identities = identities.clone();
                    let mut arm_pointer_aliases = pointer_aliases.clone();
                    let mut arm_function_aliases = function_aliases.clone();
                    let mut arm_raw_memory_identities = raw_memory_identities.clone();
                    self.check_ops(
                        &mut arm_identities,
                        &mut arm_pointer_aliases,
                        &mut arm_function_aliases,
                        &mut arm_raw_memory_identities,
                        &arm.ops,
                    );
                    arm_identities.copy_identity(&arm.value, output);
                    copy_pointer_alias(
                        &mut arm_pointer_aliases,
                        &mut arm_raw_memory_identities,
                        &arm.value,
                        output,
                    );
                    arm_function_aliases.copy_alias(&arm.value, output);
                    arm_paths.push(arm_identities);
                    pointer_alias_paths.push(arm_pointer_aliases);
                    function_alias_paths.push(arm_function_aliases);
                    raw_memory_identity_paths.push(arm_raw_memory_identities);
                }
                if !arm_paths.is_empty() {
                    *identities = RawIdentityTable::merge_paths(&arm_paths);
                    *pointer_aliases = RawPointerAliasTable::merge_paths(&pointer_alias_paths);
                    *function_aliases = FunctionAliasTable::merge_paths(&function_alias_paths);
                    *raw_memory_identities =
                        RawMemoryIdentityTable::merge_paths(&raw_memory_identity_paths);
                }
            }
            ResourceOp::FunctionValue { output, name, .. } => {
                function_aliases.set_alias(output, name.clone());
            }
            ResourceOp::Expr { .. } | ResourceOp::Borrow { .. } | ResourceOp::Drop { .. } => {}
        }
    }

    fn copy_call_return_identity(
        &self,
        identities: &mut RawIdentityTable,
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
        if summary
            .parameter_indices
            .iter()
            .filter_map(|index| args.get(*index))
            .any(|arg| identities.contains(arg))
        {
            identities.mark(output);
        }
    }

    fn copy_indirect_call_return_identity(
        &self,
        identities: &mut RawIdentityTable,
        function_aliases: &FunctionAliasTable,
        output: &Place,
        callee: &Place,
        args: &[Place],
    ) {
        let functions = function_aliases.functions(callee);
        if functions.is_empty() {
            if args.iter().any(|arg| identities.contains(arg)) {
                identities.mark(output);
            }
            return;
        }
        for function in functions {
            if self
                .summaries
                .iter()
                .find(|summary| summary.function == function.as_str())
                .is_some_and(|summary| {
                    summary
                        .parameter_indices
                        .iter()
                        .filter_map(|index| args.get(*index))
                        .any(|arg| identities.contains(arg))
                })
            {
                identities.mark(output);
                return;
            }
        }
    }

    fn copy_call_return_pointer_alias(
        &self,
        pointer_aliases: &mut RawPointerAliasTable,
        raw_memory_identities: &mut RawMemoryIdentityTable,
        output: &Place,
        target: &ResourceCallTarget,
        args: &[Place],
    ) {
        let ResourceCallTarget::User { name, .. } = target else {
            return;
        };
        let Some(summary) = self
            .pointer_summaries
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
            copy_pointer_alias(pointer_aliases, raw_memory_identities, arg, output);
        }
    }

    fn copy_indirect_call_return_pointer_alias(
        &self,
        pointer_aliases: &mut RawPointerAliasTable,
        raw_memory_identities: &mut RawMemoryIdentityTable,
        output: &Place,
        callee: &Place,
        args: &[Place],
        function_aliases: &FunctionAliasTable,
    ) {
        let functions = function_aliases.functions(callee);
        if functions.is_empty() {
            for arg in args {
                copy_pointer_alias(pointer_aliases, raw_memory_identities, arg, output);
            }
            return;
        }
        for function in functions {
            if let Some(summary) = self
                .pointer_summaries
                .iter()
                .find(|summary| summary.function == function.as_str())
            {
                for arg in summary
                    .parameter_indices
                    .iter()
                    .filter_map(|index| args.get(*index))
                {
                    copy_pointer_alias(pointer_aliases, raw_memory_identities, arg, output);
                }
            }
        }
    }

    fn apply_raw_memory_identity_effect(
        &self,
        identities: &mut RawIdentityTable,
        pointer_aliases: &RawPointerAliasTable,
        raw_memory_identities: &mut RawMemoryIdentityTable,
        operation: &RawMemoryOp,
        output: &Place,
        args: &[Place],
    ) {
        match operation {
            RawMemoryOp::Load => {
                if args
                    .first()
                    .is_some_and(|ptr| raw_memory_identities.contains(pointer_aliases, ptr))
                {
                    identities.mark(output);
                }
            }
            RawMemoryOp::Store => {
                if let Some(ptr) = args.first() {
                    if args.get(1).is_some_and(|value| identities.contains(value)) {
                        raw_memory_identities.mark(pointer_aliases, ptr);
                    } else {
                        raw_memory_identities.clear(pointer_aliases, ptr);
                    }
                }
            }
            RawMemoryOp::Realloc => {
                let carries_payload = args
                    .first()
                    .is_some_and(|ptr| raw_memory_identities.contains(pointer_aliases, ptr));
                if let Some(ptr) = args.first() {
                    raw_memory_identities.clear(pointer_aliases, ptr);
                }
                if carries_payload {
                    raw_memory_identities.mark(pointer_aliases, output);
                }
            }
            RawMemoryOp::Dealloc => {
                if let Some(ptr) = args.first() {
                    raw_memory_identities.clear(pointer_aliases, ptr);
                }
            }
            RawMemoryOp::BulkCopy | RawMemoryOp::BulkMove => {
                if let (Some(dst), Some(src)) = (args.first(), args.get(1)) {
                    if raw_memory_identities.contains(pointer_aliases, src) {
                        raw_memory_identities.mark(pointer_aliases, dst);
                    }
                }
            }
            RawMemoryOp::Alloc
            | RawMemoryOp::MemorySize
            | RawMemoryOp::MemoryGrow
            | RawMemoryOp::Fill
            | RawMemoryOp::Other { .. } => {}
        }
    }

    fn check_effect(&mut self, effect: &EffectOp, span: Span) {
        match effect {
            EffectOp::InternalAlloc => {
                self.counts.internal_allocs += 1;
            }
            EffectOp::UnsafeMemory { operation } => {
                self.counts.unsafe_memory_ops += 1;
                if matches!(self.effect, Effect::Pure) {
                    self.diagnostics.push(
                        ResourceEffectBoundaryDiagnostic::UnsafeMemoryInPureFunction {
                            function: String::from(self.function),
                            operation: operation.clone(),
                            span,
                        },
                    );
                }
            }
            EffectOp::ExternalIo { .. } => {
                self.counts.external_io_ops += 1;
            }
            EffectOp::Nondet { .. } => {
                self.counts.nondet_ops += 1;
            }
            EffectOp::Unknown { .. } => {
                self.counts.unknown_ops += 1;
            }
            EffectOp::Pure | EffectOp::UserCall { .. } => {}
        }
    }
}

fn compute_raw_identity_return_summaries(
    module: &ResourceModule,
    pointer_summaries: &[RawPointerReturnSummary],
) -> Vec<RawIdentityReturnSummary> {
    let mut summaries = Vec::new();
    for _ in 0..=module.functions.len() {
        let mut next = Vec::new();
        for function in &module.functions {
            let mut parameter_indices = Vec::new();
            for (index, param) in function.params.iter().enumerate() {
                let mut identities = RawIdentityTable::default();
                identities.mark(&param.place);
                if function_returns_marked_identity(
                    function,
                    identities,
                    &summaries,
                    pointer_summaries,
                ) {
                    parameter_indices.push(index);
                }
            }
            if !parameter_indices.is_empty() {
                next.push(RawIdentityReturnSummary {
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

fn function_returns_marked_identity(
    function: &ResourceFunction,
    mut identities: RawIdentityTable,
    summaries: &[RawIdentityReturnSummary],
    pointer_summaries: &[RawPointerReturnSummary],
) -> bool {
    let mut engine = ResourceEffectBoundaryEngine {
        function: function.name.as_str(),
        effect: function.effect,
        summaries,
        pointer_summaries,
        track_alloc_identities: false,
        diagnostics: Vec::new(),
        counts: ResourceEffectCounts::default(),
    };
    let mut function_aliases = FunctionAliasTable::default();
    let mut pointer_aliases = RawPointerAliasTable::default();
    let mut raw_memory_identities = RawMemoryIdentityTable::default();
    for block in &function.blocks {
        engine.check_ops(
            &mut identities,
            &mut pointer_aliases,
            &mut function_aliases,
            &mut raw_memory_identities,
            &block.ops,
        );
        if let ResourceTerminator::Return {
            value: Some(place), ..
        } = &block.terminator
        {
            if identities.contains(place) {
                return true;
            }
        }
    }
    false
}

fn compute_raw_pointer_return_summaries(module: &ResourceModule) -> Vec<RawPointerReturnSummary> {
    let mut summaries = Vec::new();
    for _ in 0..=module.functions.len() {
        let mut next = Vec::new();
        for function in &module.functions {
            let mut parameter_indices = Vec::new();
            for (index, param) in function.params.iter().enumerate() {
                let mut pointer_aliases = RawPointerAliasTable::default();
                pointer_aliases.mark(&param.place);
                if function_returns_pointer_alias(
                    function,
                    &param.place,
                    pointer_aliases,
                    &summaries,
                ) {
                    parameter_indices.push(index);
                }
            }
            if !parameter_indices.is_empty() {
                next.push(RawPointerReturnSummary {
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

fn function_returns_pointer_alias(
    function: &ResourceFunction,
    parameter: &Place,
    mut pointer_aliases: RawPointerAliasTable,
    pointer_summaries: &[RawPointerReturnSummary],
) -> bool {
    let mut engine = ResourceEffectBoundaryEngine {
        function: function.name.as_str(),
        effect: function.effect,
        summaries: &[],
        pointer_summaries,
        track_alloc_identities: false,
        diagnostics: Vec::new(),
        counts: ResourceEffectCounts::default(),
    };
    let mut identities = RawIdentityTable::default();
    let mut function_aliases = FunctionAliasTable::default();
    let mut raw_memory_identities = RawMemoryIdentityTable::default();
    for block in &function.blocks {
        engine.check_ops(
            &mut identities,
            &mut pointer_aliases,
            &mut function_aliases,
            &mut raw_memory_identities,
            &block.ops,
        );
        if let ResourceTerminator::Return {
            value: Some(place), ..
        } = &block.terminator
        {
            if pointer_aliases.aliases(place, parameter) {
                return true;
            }
        }
    }
    false
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
        } else {
            self.clear_alias(target);
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

    fn clear_alias(&mut self, place: &Place) {
        self.entries.retain(|entry| entry.place != *place);
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

fn copy_pointer_alias(
    pointer_aliases: &mut RawPointerAliasTable,
    raw_memory_identities: &mut RawMemoryIdentityTable,
    source: &Place,
    target: &Place,
) {
    raw_memory_identities.remove_place(target);
    pointer_aliases.copy_alias(source, target);
}

#[derive(Debug, Clone, Default)]
struct RawMemoryIdentityTable {
    pointer_groups: Vec<Vec<Place>>,
}

impl RawMemoryIdentityTable {
    fn contains(&self, pointer_aliases: &RawPointerAliasTable, place: &Place) -> bool {
        let group = pointer_aliases.group_for_or_singleton(place);
        self.pointer_groups
            .iter()
            .any(|stored| groups_overlap(stored, &group))
    }

    fn mark(&mut self, pointer_aliases: &RawPointerAliasTable, place: &Place) {
        self.union_group(&pointer_aliases.group_for_or_singleton(place));
    }

    fn clear(&mut self, pointer_aliases: &RawPointerAliasTable, place: &Place) {
        let group = pointer_aliases.group_for_or_singleton(place);
        self.pointer_groups
            .retain(|stored| !groups_overlap(stored, &group));
    }

    fn remove_place(&mut self, place: &Place) {
        for group in &mut self.pointer_groups {
            group.retain(|existing| existing != place);
        }
        self.pointer_groups.retain(|group| !group.is_empty());
    }

    fn merge_paths(paths: &[RawMemoryIdentityTable]) -> Self {
        let mut out = RawMemoryIdentityTable::default();
        for path in paths {
            for group in &path.pointer_groups {
                out.union_group(group);
            }
        }
        out
    }

    fn union_group(&mut self, group: &[Place]) {
        let mut merged = group.to_vec();
        let mut retained = Vec::new();
        for existing in self.pointer_groups.drain(..) {
            if groups_overlap(&existing, &merged) {
                push_unique_places(&mut merged, &existing);
            } else {
                retained.push(existing);
            }
        }
        if !merged.is_empty() {
            retained.push(merged);
        }
        self.pointer_groups = retained;
    }
}

#[derive(Debug, Clone, Default)]
struct RawPointerAliasTable {
    groups: Vec<Vec<Place>>,
}

impl RawPointerAliasTable {
    fn mark(&mut self, place: &Place) {
        self.union_group(core::slice::from_ref(place));
    }

    fn copy_alias(&mut self, source: &Place, target: &Place) {
        if source == target {
            return;
        }
        self.remove_place(target);
        let mut merged = self.group_for_or_singleton(source);
        if !merged.contains(target) {
            merged.push(target.clone());
        }
        self.union_group(&merged);
    }

    fn merge_paths(paths: &[RawPointerAliasTable]) -> Self {
        let mut out = RawPointerAliasTable::default();
        for path in paths {
            for group in &path.groups {
                out.union_group(group);
            }
        }
        out
    }

    fn aliases(&self, left: &Place, right: &Place) -> bool {
        let left_group = self.group_for_or_singleton(left);
        let right_group = self.group_for_or_singleton(right);
        groups_overlap(&left_group, &right_group)
    }

    fn group_for_or_singleton(&self, place: &Place) -> Vec<Place> {
        self.groups
            .iter()
            .find(|group| group.iter().any(|existing| existing == place))
            .cloned()
            .unwrap_or_else(|| vec![place.clone()])
    }

    fn remove_place(&mut self, place: &Place) {
        for group in &mut self.groups {
            group.retain(|existing| existing != place);
        }
        self.groups.retain(|group| !group.is_empty());
    }

    fn union_group(&mut self, group: &[Place]) {
        let mut merged = group.to_vec();
        let mut retained = Vec::new();
        for existing in self.groups.drain(..) {
            if groups_overlap(&existing, &merged) {
                push_unique_places(&mut merged, &existing);
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

#[derive(Debug, Clone, Default)]
struct RawIdentityTable {
    groups: Vec<Vec<Place>>,
}

impl RawIdentityTable {
    fn contains(&self, place: &Place) -> bool {
        self.groups
            .iter()
            .any(|group| group.iter().any(|existing| existing == place))
    }

    fn mark(&mut self, place: &Place) {
        self.union_group(core::slice::from_ref(place));
    }

    fn copy_identity(&mut self, source: &Place, target: &Place) {
        if source == target {
            return;
        }
        self.clear(target);
        self.merge_identity(source, target);
    }

    fn merge_identity(&mut self, source: &Place, target: &Place) {
        if let Some(group) = self.group_for(source) {
            let mut merged = group.to_vec();
            if !merged.contains(target) {
                merged.push(target.clone());
            }
            self.union_group(&merged);
        }
    }

    fn clear(&mut self, place: &Place) {
        for group in &mut self.groups {
            group.retain(|existing| existing != place);
        }
        self.groups.retain(|group| !group.is_empty());
    }

    fn merge_paths(paths: &[RawIdentityTable]) -> Self {
        let mut out = RawIdentityTable::default();
        for path in paths {
            for group in &path.groups {
                out.union_group(group);
            }
        }
        out
    }

    fn group_for(&self, place: &Place) -> Option<&[Place]> {
        self.groups
            .iter()
            .find(|group| group.iter().any(|existing| existing == place))
            .map(Vec::as_slice)
    }

    fn union_group(&mut self, group: &[Place]) {
        let mut merged = group.to_vec();
        let mut retained = Vec::new();
        for existing in self.groups.drain(..) {
            if groups_overlap(&existing, &merged) {
                push_unique_places(&mut merged, &existing);
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

fn push_unique_places(target: &mut Vec<Place>, source: &[Place]) {
    for place in source {
        if !target.contains(place) {
            target.push(place.clone());
        }
    }
}

fn raw_memory_op_produces_identity(operation: &RawMemoryOp) -> bool {
    matches!(operation, RawMemoryOp::Alloc | RawMemoryOp::Realloc)
}
