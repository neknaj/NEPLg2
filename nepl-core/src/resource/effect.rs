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
    let summaries = compute_raw_identity_return_summaries(module);

    for function in &module.functions {
        let mut engine = ResourceEffectBoundaryEngine {
            function: function.name.as_str(),
            effect: function.effect,
            summaries: &summaries,
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

struct ResourceEffectBoundaryEngine<'a> {
    function: &'a str,
    effect: Effect,
    summaries: &'a [RawIdentityReturnSummary],
    track_alloc_identities: bool,
    diagnostics: Vec<ResourceEffectBoundaryDiagnostic>,
    counts: ResourceEffectCounts,
}

impl ResourceEffectBoundaryEngine<'_> {
    fn check_function(&mut self, function: &ResourceFunction) {
        let mut identities = RawIdentityTable::default();
        let mut function_aliases = FunctionAliasTable::default();
        for block in &function.blocks {
            self.check_block(&mut identities, &mut function_aliases, block);
        }
    }

    fn check_block(
        &mut self,
        identities: &mut RawIdentityTable,
        function_aliases: &mut FunctionAliasTable,
        block: &ResourceBlock,
    ) {
        self.check_ops(identities, function_aliases, &block.ops);
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
        function_aliases: &mut FunctionAliasTable,
        ops: &[ResourceOp],
    ) {
        for op in ops {
            self.check_op(identities, function_aliases, op);
        }
    }

    fn check_op(
        &mut self,
        identities: &mut RawIdentityTable,
        function_aliases: &mut FunctionAliasTable,
        op: &ResourceOp,
    ) {
        match op {
            ResourceOp::CallEffect { effect, span } => self.check_effect(effect, *span),
            ResourceOp::RawMemory {
                operation, output, ..
            } => {
                if self.track_alloc_identities && raw_memory_op_produces_identity(operation) {
                    identities.mark(output);
                }
            }
            ResourceOp::DeclareLocal {
                place, initializer, ..
            } => {
                if let Some(initializer) = initializer {
                    identities.copy_identity(initializer, place);
                    function_aliases.copy_alias(initializer, place);
                }
            }
            ResourceOp::Read { source, output, .. } | ResourceOp::Move { source, output, .. } => {
                identities.copy_identity(source, output);
                function_aliases.copy_alias(source, output);
            }
            ResourceOp::Assign { target, value, .. } => {
                identities.copy_identity(value, target);
                function_aliases.copy_alias(value, target);
            }
            ResourceOp::Construct { output, inputs, .. } => {
                for input in inputs {
                    identities.copy_identity(input, output);
                }
            }
            ResourceOp::Call {
                output,
                target,
                args,
                ..
            } => {
                self.copy_call_return_identity(identities, output, target, args);
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
                let mut then_function_aliases = function_aliases.clone();
                let mut else_function_aliases = function_aliases.clone();
                self.check_ops(&mut then_identities, &mut then_function_aliases, then_ops);
                self.check_ops(&mut else_identities, &mut else_function_aliases, else_ops);
                then_identities.copy_identity(then_value, output);
                else_identities.copy_identity(else_value, output);
                then_function_aliases.copy_alias(then_value, output);
                else_function_aliases.copy_alias(else_value, output);
                *identities = RawIdentityTable::merge_paths(&[then_identities, else_identities]);
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
                let mut condition_identities = identities.clone();
                let mut condition_function_aliases = function_aliases.clone();
                self.check_ops(
                    &mut condition_identities,
                    &mut condition_function_aliases,
                    condition_ops,
                );
                let mut body_identities = condition_identities.clone();
                let mut body_function_aliases = condition_function_aliases.clone();
                self.check_ops(&mut body_identities, &mut body_function_aliases, body_ops);
                *identities =
                    RawIdentityTable::merge_paths(&[condition_identities, body_identities]);
                *function_aliases = FunctionAliasTable::merge_paths(&[
                    condition_function_aliases,
                    body_function_aliases,
                ]);
            }
            ResourceOp::Match { output, arms, .. } => {
                let mut arm_paths = Vec::new();
                let mut function_alias_paths = Vec::new();
                for arm in arms {
                    let mut arm_identities = identities.clone();
                    let mut arm_function_aliases = function_aliases.clone();
                    self.check_ops(&mut arm_identities, &mut arm_function_aliases, &arm.ops);
                    arm_identities.copy_identity(&arm.value, output);
                    arm_function_aliases.copy_alias(&arm.value, output);
                    arm_paths.push(arm_identities);
                    function_alias_paths.push(arm_function_aliases);
                }
                if !arm_paths.is_empty() {
                    *identities = RawIdentityTable::merge_paths(&arm_paths);
                    *function_aliases = FunctionAliasTable::merge_paths(&function_alias_paths);
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

fn compute_raw_identity_return_summaries(module: &ResourceModule) -> Vec<RawIdentityReturnSummary> {
    let mut summaries = Vec::new();
    for _ in 0..=module.functions.len() {
        let mut next = Vec::new();
        for function in &module.functions {
            let mut parameter_indices = Vec::new();
            for (index, param) in function.params.iter().enumerate() {
                let mut identities = RawIdentityTable::default();
                identities.mark(&param.place);
                if function_returns_marked_identity(function, identities, &summaries) {
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
) -> bool {
    let mut engine = ResourceEffectBoundaryEngine {
        function: function.name.as_str(),
        effect: function.effect,
        summaries,
        track_alloc_identities: false,
        diagnostics: Vec::new(),
        counts: ResourceEffectCounts::default(),
    };
    let mut function_aliases = FunctionAliasTable::default();
    for block in &function.blocks {
        engine.check_ops(&mut identities, &mut function_aliases, &block.ops);
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
struct RawIdentityTable {
    places: Vec<Place>,
}

impl RawIdentityTable {
    fn contains(&self, place: &Place) -> bool {
        self.places.iter().any(|existing| existing == place)
    }

    fn mark(&mut self, place: &Place) {
        if !self.contains(place) {
            self.places.push(place.clone());
        }
    }

    fn copy_identity(&mut self, source: &Place, target: &Place) {
        if self.contains(source) {
            self.mark(target);
        }
    }

    fn merge_paths(paths: &[RawIdentityTable]) -> Self {
        let mut out = RawIdentityTable::default();
        for path in paths {
            for place in &path.places {
                out.mark(place);
            }
        }
        out
    }
}

fn raw_memory_op_produces_identity(operation: &RawMemoryOp) -> bool {
    matches!(operation, RawMemoryOp::Alloc | RawMemoryOp::Realloc)
}
