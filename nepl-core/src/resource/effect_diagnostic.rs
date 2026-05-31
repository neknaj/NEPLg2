extern crate alloc;

use alloc::string::String;

use crate::diagnostic_codes::{
    DiagnosticCode, EffectDiagnosticCode, ResourceDiagnosticCode, ResourceLowerDiagnosticCode,
    ResourceRawDiagnosticCode,
};
use crate::span::Span;

use super::model::{
    ExternalIoOp, NondetOp, Place, PrivateCacheOp, PrivateStateOp, RawAddressAliasKind,
    RawAddressViewKind, RawMemoryOp, UnknownEffectReason,
};

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
    PrivateStateInPureFunction {
        function: String,
        operation: PrivateStateOp,
        span: Span,
    },
    PrivateCacheInPureFunction {
        function: String,
        operation: PrivateCacheOp,
        span: Span,
    },
    PrivateCacheOutsideBoundary {
        function: String,
        operation: PrivateCacheOp,
        span: Span,
    },
    RawMemoryOutsideBoundary {
        function: String,
        operation: RawMemoryOp,
        span: Span,
    },
    RawAddressViewOutsideBoundary {
        function: String,
        kind: RawAddressViewKind,
        span: Span,
    },
    RawAddressAliasOutsideBoundary {
        function: String,
        kind: RawAddressAliasKind,
        span: Span,
    },
    CheckedMemPtrOutsideBoundary {
        function: String,
        operation: RawMemoryOp,
        place: Place,
        span: Span,
    },
    RawAddressEscapeFromInternalAlloc {
        function: String,
        operation: RawMemoryOp,
        place: Place,
        origin_span: Span,
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
            | ResourceEffectBoundaryDiagnostic::UnsafeMemoryInPureFunction { .. }
            | ResourceEffectBoundaryDiagnostic::PrivateStateInPureFunction { .. }
            | ResourceEffectBoundaryDiagnostic::PrivateCacheInPureFunction { .. } => {
                DiagnosticCode::Effect(EffectDiagnosticCode::PureCallsImpure)
            }
            ResourceEffectBoundaryDiagnostic::PrivateCacheOutsideBoundary { .. }
            | ResourceEffectBoundaryDiagnostic::RawMemoryOutsideBoundary { .. }
            | ResourceEffectBoundaryDiagnostic::RawAddressViewOutsideBoundary { .. }
            | ResourceEffectBoundaryDiagnostic::RawAddressAliasOutsideBoundary { .. }
            | ResourceEffectBoundaryDiagnostic::CheckedMemPtrOutsideBoundary { .. } => {
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
