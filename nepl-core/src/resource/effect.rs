extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;

use crate::ast::Effect;
use crate::span::Span;

use super::model::{
    EffectOp, Place, RawMemoryOp, ResourceBlock, ResourceFunction, ResourceModule, ResourceOp,
    ResourceTerminator,
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

    for function in &module.functions {
        let mut engine = ResourceEffectBoundaryEngine {
            function: function.name.as_str(),
            effect: function.effect,
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

struct ResourceEffectBoundaryEngine<'a> {
    function: &'a str,
    effect: Effect,
    diagnostics: Vec<ResourceEffectBoundaryDiagnostic>,
    counts: ResourceEffectCounts,
}

impl ResourceEffectBoundaryEngine<'_> {
    fn check_function(&mut self, function: &ResourceFunction) {
        let mut identities = RawIdentityTable::default();
        for block in &function.blocks {
            self.check_block(&mut identities, block);
        }
    }

    fn check_block(&mut self, identities: &mut RawIdentityTable, block: &ResourceBlock) {
        self.check_ops(identities, &block.ops);
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

    fn check_ops(&mut self, identities: &mut RawIdentityTable, ops: &[ResourceOp]) {
        for op in ops {
            self.check_op(identities, op);
        }
    }

    fn check_op(&mut self, identities: &mut RawIdentityTable, op: &ResourceOp) {
        match op {
            ResourceOp::CallEffect { effect, span } => self.check_effect(effect, *span),
            ResourceOp::RawMemory {
                operation, output, ..
            } => {
                if raw_memory_op_produces_identity(operation) {
                    identities.mark(output);
                }
            }
            ResourceOp::DeclareLocal {
                place, initializer, ..
            } => {
                if let Some(initializer) = initializer {
                    identities.copy_identity(initializer, place);
                }
            }
            ResourceOp::Read { source, output, .. } | ResourceOp::Move { source, output, .. } => {
                identities.copy_identity(source, output);
            }
            ResourceOp::Assign { target, value, .. } => {
                identities.copy_identity(value, target);
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
                self.check_ops(&mut then_identities, then_ops);
                self.check_ops(&mut else_identities, else_ops);
                then_identities.copy_identity(then_value, output);
                else_identities.copy_identity(else_value, output);
                *identities = RawIdentityTable::merge_paths(&[then_identities, else_identities]);
            }
            ResourceOp::Loop {
                condition_ops,
                body_ops,
                ..
            } => {
                let mut condition_identities = identities.clone();
                self.check_ops(&mut condition_identities, condition_ops);
                let mut body_identities = condition_identities.clone();
                self.check_ops(&mut body_identities, body_ops);
                *identities =
                    RawIdentityTable::merge_paths(&[condition_identities, body_identities]);
            }
            ResourceOp::Match { output, arms, .. } => {
                let mut arm_paths = Vec::new();
                for arm in arms {
                    let mut arm_identities = identities.clone();
                    self.check_ops(&mut arm_identities, &arm.ops);
                    arm_identities.copy_identity(&arm.value, output);
                    arm_paths.push(arm_identities);
                }
                if !arm_paths.is_empty() {
                    *identities = RawIdentityTable::merge_paths(&arm_paths);
                }
            }
            ResourceOp::Expr { .. }
            | ResourceOp::Borrow { .. }
            | ResourceOp::Drop { .. }
            | ResourceOp::FunctionValue { .. }
            | ResourceOp::Call { .. }
            | ResourceOp::IndirectCall { .. }
            | ResourceOp::Construct { .. } => {}
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
