extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;

use crate::span::Span;

use super::effect_check::ResourceEffectBoundaryEngine;
use super::effect_summary::{
    compute_raw_identity_return_summaries, compute_raw_pointer_return_summaries,
};
use super::model::{Place, RawMemoryOp, ResourceModule};

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
    pub internal_memory_ops: RawMemoryEffectCounts,
    pub unsafe_memory_ops: RawMemoryEffectCounts,
    pub external_io_ops: usize,
    pub nondet_ops: usize,
    pub unknown_ops: usize,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct RawMemoryEffectCounts {
    pub alloc: usize,
    pub dealloc: usize,
    pub realloc: usize,
    pub load: usize,
    pub store: usize,
    pub bulk_copy: usize,
    pub bulk_move: usize,
    pub memory_size: usize,
    pub memory_grow: usize,
    pub fill: usize,
}

impl RawMemoryEffectCounts {
    pub fn record(&mut self, operation: RawMemoryOp) {
        match operation {
            RawMemoryOp::Alloc => self.alloc += 1,
            RawMemoryOp::Dealloc => self.dealloc += 1,
            RawMemoryOp::Realloc => self.realloc += 1,
            RawMemoryOp::Load => self.load += 1,
            RawMemoryOp::Store => self.store += 1,
            RawMemoryOp::BulkCopy => self.bulk_copy += 1,
            RawMemoryOp::BulkMove => self.bulk_move += 1,
            RawMemoryOp::MemorySize => self.memory_size += 1,
            RawMemoryOp::MemoryGrow => self.memory_grow += 1,
            RawMemoryOp::Fill => self.fill += 1,
        }
    }

    pub fn total(self) -> usize {
        self.alloc
            + self.dealloc
            + self.realloc
            + self.load
            + self.store
            + self.bulk_copy
            + self.bulk_move
            + self.memory_size
            + self.memory_grow
            + self.fill
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResourceEffectBoundaryDiagnostic {
    ImpureCallInPureFunction {
        function: String,
        call: ResourceEffectCallKind,
        span: Span,
    },
    UnsafeMemoryInPureFunction {
        function: String,
        operation: RawMemoryOp,
        span: Span,
    },
    RawAddressEscapeFromInternalAlloc {
        function: String,
        place: Place,
        span: Span,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResourceEffectCallKind {
    Direct { name: String },
    Indirect,
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
