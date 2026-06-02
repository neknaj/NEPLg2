use alloc::boxed::Box;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

use crate::ast::{Effect, TypeExpr};
use crate::diagnostic::Diagnostic;
use crate::diagnostic_codes::{DiagnosticCode, ParserDiagnosticCode};
use crate::span::Span;

pub(in crate::parser) fn parse_type_expr_str(
    s: &str,
    span: Span,
    diags: &mut Vec<Diagnostic>,
) -> Option<TypeExpr> {
    let trimmed = s.trim();
    if let Some(rest) = trimmed.strip_prefix('%') {
        return parse_neplg21_type_expr_str(rest, span, diags);
    }

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

fn parse_neplg21_type_expr_str(
    s: &str,
    span: Span,
    diags: &mut Vec<Diagnostic>,
) -> Option<TypeExpr> {
    let tokens: Vec<&str> = s.split_whitespace().collect();
    let mut parser = PrefixTypeParser {
        tokens: &tokens,
        pos: 0,
        span,
        diags,
    };
    let ty = parser.parse_type()?;
    if parser.pos != tokens.len() {
        parser.diags.push(signature_error(
            "unexpected token in #extern signature",
            span,
        ));
        return None;
    }
    Some(ty)
}

struct PrefixTypeParser<'a, 'b> {
    tokens: &'a [&'a str],
    pos: usize,
    span: Span,
    diags: &'b mut Vec<Diagnostic>,
}

impl PrefixTypeParser<'_, '_> {
    fn parse_type(&mut self) -> Option<TypeExpr> {
        if self.pos >= self.tokens.len() {
            self.diags.push(signature_error(
                "missing type in #extern signature",
                self.span,
            ));
            return None;
        }

        if self.tokens[self.pos] == "impure" {
            self.pos += 1;
            if self.pos >= self.tokens.len() || self.tokens[self.pos] != "fn" {
                self.diags.push(signature_error(
                    "expected fn after impure in #extern signature",
                    self.span,
                ));
                return None;
            }
            self.pos += 1;
            return self.parse_function_type(Effect::Impure);
        }

        if self.tokens[self.pos] == "fn" {
            self.pos += 1;
            return self.parse_function_type(Effect::Pure);
        }

        let token = self.tokens[self.pos];
        self.pos += 1;
        simple_type_atom(token, self.span, self.diags)
    }

    fn parse_function_type(&mut self, effect: Effect) -> Option<TypeExpr> {
        if self.pos < self.tokens.len() && self.tokens[self.pos] == "void" {
            self.pos += 1;
            let result = self.parse_type()?;
            return Some(
                TypeExpr::Function {
                    params: Vec::new(),
                    result: Box::new(result),
                    effect,
                }
                .with_span(self.span),
            );
        }

        let first_param = self.parse_type()?;
        let result = self.parse_type()?;
        let mut params = Vec::new();
        params.push(first_param);
        let result = match result.into_unspanned() {
            TypeExpr::Function {
                params: nested_params,
                result,
                effect: nested_effect,
            } if nested_effect == effect && !nested_params.is_empty() => {
                params.extend(nested_params);
                result
            }
            other => Box::new(other.with_span(self.span)),
        };
        Some(
            TypeExpr::Function {
                params,
                result,
                effect,
            }
            .with_span(self.span),
        )
    }
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
        "unit" => TypeExpr::Unit,
        "()" => TypeExpr::Unit,
        "void" => {
            diags.push(signature_error(
                "void is only allowed as a zero-argument function marker",
                span,
            ));
            return None;
        }
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
