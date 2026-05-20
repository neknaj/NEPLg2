use alloc::boxed::Box;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

use crate::ast::{Effect, TypeExpr};
use crate::diagnostic::Diagnostic;
use crate::diagnostic_codes::{DiagnosticCode, ParserDiagnosticCode};
use crate::span::Span;

pub(super) fn parse_type_expr_str(
    s: &str,
    span: Span,
    diags: &mut Vec<Diagnostic>,
) -> Option<TypeExpr> {
    let trimmed = s.trim();
    if !trimmed.starts_with('<') || !trimmed.ends_with('>') {
        diags.push(signature_error("invalid type signature in #extern", span));
        return None;
    }
    let inner = &trimmed[1..trimmed.len() - 1];
    let effect = if let Some(idx) = inner.find("*>") {
        (Effect::Impure, idx)
    } else if let Some(idx) = inner.find("->") {
        (Effect::Pure, idx)
    } else {
        diags.push(signature_error("missing -> or *> in signature", span));
        return None;
    };
    let (eff, split_idx) = effect;
    let (params_part, ret_part) = inner.split_at(split_idx);
    let ret_part = &ret_part[2..];
    let params_clean = params_part
        .trim()
        .trim_start_matches('(')
        .trim_end_matches(')');
    let mut params = Vec::new();
    if !params_clean.is_empty() {
        for p in params_clean.split(',') {
            params.push(simple_type_atom(p.trim(), span, diags)?);
        }
    }
    let result = simple_type_atom(ret_part.trim(), span, diags)?;
    Some(
        TypeExpr::Function {
            params,
            result: Box::new(result),
            effect: eff,
        }
        .with_span(span),
    )
}

fn simple_type_atom(t: &str, span: Span, diags: &mut Vec<Diagnostic>) -> Option<TypeExpr> {
    let ty = match t {
        "i32" => TypeExpr::I32,
        "u8" => TypeExpr::U8,
        "f32" => TypeExpr::F32,
        "i64" => TypeExpr::Named("i64".to_string()),
        "f64" => TypeExpr::Named("f64".to_string()),
        "bool" => TypeExpr::Bool,
        "char" => TypeExpr::Char,
        "never" => TypeExpr::Never,
        "str" => TypeExpr::Str,
        "()" => TypeExpr::Unit,
        _ if t.starts_with('.') => TypeExpr::Label(Some(t.trim_start_matches('.').to_string())),
        _ if t.is_empty() => TypeExpr::Label(None),
        _ if t
            .chars()
            .next()
            .map(|c| c.is_ascii_alphabetic() || c == '_')
            .unwrap_or(false)
            && t.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') =>
        {
            TypeExpr::Named(t.to_string())
        }
        _ => {
            diags.push(signature_error("unknown type in signature", span));
            return None;
        }
    };
    Some(ty.with_span(span))
}

fn signature_error(message: impl Into<String>, span: Span) -> Diagnostic {
    Diagnostic::error_with_code(
        DiagnosticCode::Parser(ParserDiagnosticCode::ExternSignatureInvalid),
        message,
        span,
    )
}
