use alloc::string::String;

use crate::diagnostic::Diagnostic;
use crate::diagnostic_codes::{
    DiagnosticCode, EffectDiagnosticCode, ResolveDiagnosticCode, TypeDiagnosticCode,
};
use crate::span::Span;

pub(super) fn type_error(
    code: TypeDiagnosticCode,
    message: impl Into<String>,
    span: Span,
) -> Diagnostic {
    Diagnostic::error_with_code(DiagnosticCode::Type(code), message, span)
}

pub(super) fn effect_error(
    code: EffectDiagnosticCode,
    message: impl Into<String>,
    span: Span,
) -> Diagnostic {
    Diagnostic::error_with_code(DiagnosticCode::Effect(code), message, span)
}

pub(super) fn resolve_error(
    code: ResolveDiagnosticCode,
    message: impl Into<String>,
    span: Span,
) -> Diagnostic {
    Diagnostic::error_with_code(DiagnosticCode::Resolve(code), message, span)
}

pub(super) fn resolve_warning(
    code: ResolveDiagnosticCode,
    message: impl Into<String>,
    span: Span,
) -> Diagnostic {
    Diagnostic::warning_with_code(DiagnosticCode::Resolve(code), message, span)
}
