extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;

use crate::diagnostic_codes::{
    DiagnosticCode, EffectDiagnosticCode, ResourceDiagnosticCode, ResourceLowerDiagnosticCode,
    ResourceRawDiagnosticCode,
};
use crate::span::Span;

use super::effect_check::ResourceEffectBoundaryEngine;
use super::effect_counts::ResourceEffectCounts;
use super::effect_summary::{
    compute_raw_identity_return_summaries, compute_raw_pointer_return_summaries,
    RawIdentityReturnSummaryIndex, RawPointerReturnSummaryIndex,
};
use super::model::{
    ExternalIoOp, NondetOp, Place, RawMemoryOp, ResourceModule, UnknownEffectReason,
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
    RawMemoryOutsideBoundary {
        function: String,
        operation: RawMemoryOp,
        span: Span,
    },
    RawAddressEscapeFromInternalAlloc {
        function: String,
        place: Place,
        span: Span,
    },
    UnknownEffect {
        function: String,
        reason: UnknownEffectReason,
        span: Span,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResourceEffectCallKind {
    Direct { name: String },
    ExternalIo { operation: ExternalIoOp },
    Nondet { operation: NondetOp },
    Indirect,
}

impl ResourceEffectBoundaryDiagnostic {
    pub fn diagnostic_code(&self) -> DiagnosticCode {
        match self {
            ResourceEffectBoundaryDiagnostic::ImpureCallInPureFunction { .. }
            | ResourceEffectBoundaryDiagnostic::UnsafeMemoryInPureFunction { .. } => {
                DiagnosticCode::Effect(EffectDiagnosticCode::PureCallsImpure)
            }
            ResourceEffectBoundaryDiagnostic::RawMemoryOutsideBoundary { .. } => {
                DiagnosticCode::Resource(ResourceDiagnosticCode::Raw(
                    ResourceRawDiagnosticCode::MemoryOutsideBoundary,
                ))
            }
            ResourceEffectBoundaryDiagnostic::RawAddressEscapeFromInternalAlloc { .. } => {
                DiagnosticCode::Resource(ResourceDiagnosticCode::Raw(
                    ResourceRawDiagnosticCode::IdentityEscape,
                ))
            }
            ResourceEffectBoundaryDiagnostic::UnknownEffect { .. } => DiagnosticCode::Resource(
                ResourceDiagnosticCode::Lower(ResourceLowerDiagnosticCode::Incomplete),
            ),
        }
    }
}

pub fn check_resource_effect_boundaries(module: &ResourceModule) -> ResourceEffectBoundaryReport {
    let mut functions = Vec::new();
    let mut diagnostics = Vec::new();
    let pointer_summaries = compute_raw_pointer_return_summaries(module);
    let pointer_summary_index = RawPointerReturnSummaryIndex::new(&pointer_summaries);
    let summaries = compute_raw_identity_return_summaries(module, &pointer_summaries);
    let summary_index = RawIdentityReturnSummaryIndex::new(&summaries);

    for function in &module.functions {
        let mut engine = ResourceEffectBoundaryEngine {
            function: function.name.as_str(),
            effect: function.effect,
            summaries: &summary_index,
            pointer_summaries: &pointer_summary_index,
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
