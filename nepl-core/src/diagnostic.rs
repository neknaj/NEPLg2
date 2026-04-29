//! Rich diagnostics for the NEPL compiler.
//!
//! This module defines diagnostic structures used to report errors
//! and warnings with precise source locations and optional notes.

use alloc::string::String;
use alloc::vec::Vec;

use crate::diagnostic_codes::DiagnosticCode;
use crate::span::Span;

/// Severity level of a diagnostic message.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
}

/// A labeled span used inside diagnostics.
///
/// Each label points to a specific span in the source code and
/// optionally carries a short message explaining the highlight.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Label {
    pub span: Span,
    pub message: Option<String>,
}

/// A single diagnostic message produced by the compiler.
///
/// A diagnostic has a main message, a primary label indicating the
/// main source location, and zero or more secondary labels for
/// related locations (for example, “defined here”, “required here”).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub severity: Severity,
    pub code: Option<DiagnosticCode>,
    pub message: String,
    pub primary: Label,
    pub secondary: Vec<Label>,
    pub notes: Vec<String>,
    pub helps: Vec<String>,
}

/// Compiler-owned diagnostic category and severity.
///
/// `DiagnosticSpec` keeps the stable enum code at the point where a diagnostic
/// is constructed.  Passes can still attach contextual text, but the diagnostic
/// category is no longer a post-construction string-like field.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DiagnosticSpec {
    pub severity: Severity,
    pub code: DiagnosticCode,
}

impl DiagnosticSpec {
    pub const fn error(code: DiagnosticCode) -> DiagnosticSpec {
        DiagnosticSpec {
            severity: Severity::Error,
            code,
        }
    }

    pub const fn warning(code: DiagnosticCode) -> DiagnosticSpec {
        DiagnosticSpec {
            severity: Severity::Warning,
            code,
        }
    }

    pub fn build(self, primary_span: Span) -> Diagnostic {
        self.build_with_message(self.code.message(), primary_span)
    }

    pub fn build_with_message(self, message: impl Into<String>, primary_span: Span) -> Diagnostic {
        Diagnostic::from_parts(self.severity, Some(self.code), message, primary_span)
    }
}

impl Diagnostic {
    fn from_parts(
        severity: Severity,
        code: Option<DiagnosticCode>,
        message: impl Into<String>,
        primary_span: Span,
    ) -> Diagnostic {
        Diagnostic {
            severity,
            code,
            message: message.into(),
            primary: Label {
                span: primary_span,
                message: None,
            },
            secondary: Vec::new(),
            notes: Vec::new(),
            helps: Vec::new(),
        }
    }

    /// Create a new error diagnostic with a primary span.
    pub fn error(message: impl Into<String>, primary_span: Span) -> Diagnostic {
        Diagnostic::from_parts(Severity::Error, None, message, primary_span)
    }

    /// Create a new error diagnostic from a compiler-owned diagnostic code.
    pub fn error_code(code: DiagnosticCode, primary_span: Span) -> Diagnostic {
        DiagnosticSpec::error(code).build(primary_span)
    }

    /// Create a new error diagnostic with a compiler-owned code and custom text.
    pub fn error_with_code(
        code: DiagnosticCode,
        message: impl Into<String>,
        primary_span: Span,
    ) -> Diagnostic {
        DiagnosticSpec::error(code).build_with_message(message, primary_span)
    }

    /// Create a new warning diagnostic with a primary span.
    pub fn warning(message: impl Into<String>, primary_span: Span) -> Diagnostic {
        Diagnostic::from_parts(Severity::Warning, None, message, primary_span)
    }

    /// Create a new warning diagnostic from a compiler-owned diagnostic code.
    pub fn warning_code(code: DiagnosticCode, primary_span: Span) -> Diagnostic {
        DiagnosticSpec::warning(code).build(primary_span)
    }

    /// Create a new warning diagnostic with a compiler-owned code and custom text.
    pub fn warning_with_code(
        code: DiagnosticCode,
        message: impl Into<String>,
        primary_span: Span,
    ) -> Diagnostic {
        DiagnosticSpec::warning(code).build_with_message(message, primary_span)
    }

    /// Attach a compiler-owned diagnostic code to this diagnostic.
    pub fn with_code(mut self, code: DiagnosticCode) -> Diagnostic {
        self.code = Some(code);
        self
    }

    /// Add a secondary label with its own span and optional message.
    pub fn with_secondary_label(
        mut self,
        span: Span,
        message: impl Into<Option<String>>,
    ) -> Diagnostic {
        self.secondary.push(Label {
            span,
            message: message.into(),
        });
        self
    }

    /// Add additional context that should be rendered below the primary error.
    pub fn with_note(mut self, note: impl Into<String>) -> Diagnostic {
        self.notes.push(note.into());
        self
    }

    /// Add actionable guidance that should be rendered below the primary error.
    pub fn with_help(mut self, help: impl Into<String>) -> Diagnostic {
        self.helps.push(help.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagnostic_codes::{
        DiagnosticCode, ResourceDiagnosticCode, ResourceLowerDiagnosticCode,
    };

    #[test]
    fn coded_error_uses_registry_message_and_code() {
        let code = DiagnosticCode::Resource(ResourceDiagnosticCode::Lower(
            ResourceLowerDiagnosticCode::Incomplete,
        ));
        let diagnostic = Diagnostic::error_code(code, Span::dummy());

        assert_eq!(diagnostic.severity, Severity::Error);
        assert_eq!(diagnostic.code, Some(code));
        assert_eq!(diagnostic.message, code.message());
    }

    #[test]
    fn diagnostic_notes_and_helps_are_structured_values() {
        let diagnostic = Diagnostic::warning("careful", Span::dummy())
            .with_note("context")
            .with_help("action");

        assert_eq!(diagnostic.notes, ["context"]);
        assert_eq!(diagnostic.helps, ["action"]);
    }
}
