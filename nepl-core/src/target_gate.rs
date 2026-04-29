//! target/profile 条件付きコンパイル gate の共通評価。
//!
//! `#if[target=...]` と `#if[profile=...]` は typecheck / raw body precheck /
//! codegen が同じ active statement 集合を見る必要があるため、この module に
//! 判定規則と invalid gate 診断を集約する。

extern crate alloc;

use alloc::collections::BTreeSet;
use alloc::vec::Vec;

use crate::ast::{
    Block, Directive, FnBody, FnDef, ImplDef, IntrinsicExpr, MatchExpr, Module, PrefixExpr,
    PrefixItem, Stmt, TraitDef,
};
use crate::compiler::{BuildProfile, CompileTarget};
use crate::diagnostic::Diagnostic;
use crate::diagnostic_codes::{DiagnosticCode, LoaderDiagnosticCode};
use crate::span::Span;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GateDecision {
    Active,
    Inactive,
    Invalid,
}

impl GateDecision {
    pub fn allows(self) -> bool {
        matches!(self, GateDecision::Active)
    }
}

pub fn target_gate_allows_expr(expr: &str, active: CompileTarget) -> bool {
    evaluate_target_gate_expr(expr, active).allows()
}

pub fn evaluate_target_gate_expr(expr: &str, active: CompileTarget) -> GateDecision {
    let mut parser = TargetGateParser::new(expr, active);
    let value = parser.parse_or();
    if !parser.ok {
        return GateDecision::Invalid;
    }
    parser.skip_ws();
    if parser.pos < parser.src.len() {
        return GateDecision::Invalid;
    }
    if value {
        GateDecision::Active
    } else {
        GateDecision::Inactive
    }
}

pub fn evaluate_profile_gate(profile: &str, active: BuildProfile) -> GateDecision {
    match profile {
        "debug" if matches!(active, BuildProfile::Debug) => GateDecision::Active,
        "debug" => GateDecision::Inactive,
        "release" if matches!(active, BuildProfile::Release) => GateDecision::Active,
        "release" => GateDecision::Inactive,
        _ => GateDecision::Invalid,
    }
}

pub fn directive_gate_decision(
    directive: &Directive,
    target: CompileTarget,
    profile: BuildProfile,
) -> Option<GateDecision> {
    match directive {
        Directive::IfTarget { target: gate, .. } => {
            Some(evaluate_target_gate_expr(gate.as_str(), target))
        }
        Directive::IfProfile { profile: gate, .. } => {
            Some(evaluate_profile_gate(gate.as_str(), profile))
        }
        _ => None,
    }
}

pub fn directive_gate_allows(
    directive: &Directive,
    target: CompileTarget,
    profile: BuildProfile,
) -> Option<bool> {
    directive_gate_decision(directive, target, profile).map(GateDecision::allows)
}

pub fn invalid_gate_diagnostic(directive: &Directive) -> Option<Diagnostic> {
    match directive {
        Directive::IfTarget { span, .. } | Directive::IfProfile { span, .. } => {
            Some(invalid_conditional_gate_diagnostic(*span))
        }
        _ => None,
    }
}

pub fn validate_module_gates(
    module: &Module,
    target: CompileTarget,
    profile: BuildProfile,
) -> Vec<Diagnostic> {
    let mut out = Vec::new();
    let mut seen = BTreeSet::new();
    for directive in &module.directives {
        push_invalid_gate_diagnostic(&mut out, &mut seen, directive, target, profile);
    }
    validate_block_gates(&module.root, target, profile, &mut out, &mut seen);
    out
}

fn validate_block_gates(
    block: &Block,
    target: CompileTarget,
    profile: BuildProfile,
    out: &mut Vec<Diagnostic>,
    seen: &mut BTreeSet<(u32, u32, u32)>,
) {
    for stmt in &block.items {
        match stmt {
            Stmt::Directive(directive) => {
                push_invalid_gate_diagnostic(out, seen, directive, target, profile);
            }
            Stmt::FnDef(function) => validate_function_gates(function, target, profile, out, seen),
            Stmt::Trait(trait_def) => validate_trait_gates(trait_def, target, profile, out, seen),
            Stmt::Impl(impl_def) => validate_impl_gates(impl_def, target, profile, out, seen),
            Stmt::Expr(expr) | Stmt::ExprSemi(expr, _) => {
                validate_prefix_gates(expr, target, profile, out, seen);
            }
            _ => {}
        }
    }
}

fn validate_trait_gates(
    trait_def: &TraitDef,
    target: CompileTarget,
    profile: BuildProfile,
    out: &mut Vec<Diagnostic>,
    seen: &mut BTreeSet<(u32, u32, u32)>,
) {
    for method in &trait_def.methods {
        validate_function_gates(method, target, profile, out, seen);
    }
}

fn validate_impl_gates(
    impl_def: &ImplDef,
    target: CompileTarget,
    profile: BuildProfile,
    out: &mut Vec<Diagnostic>,
    seen: &mut BTreeSet<(u32, u32, u32)>,
) {
    for method in &impl_def.methods {
        validate_function_gates(method, target, profile, out, seen);
    }
}

fn validate_function_gates(
    function: &FnDef,
    target: CompileTarget,
    profile: BuildProfile,
    out: &mut Vec<Diagnostic>,
    seen: &mut BTreeSet<(u32, u32, u32)>,
) {
    if let FnBody::Parsed(block) = &function.body {
        validate_block_gates(block, target, profile, out, seen);
    }
}

fn validate_prefix_gates(
    expr: &PrefixExpr,
    target: CompileTarget,
    profile: BuildProfile,
    out: &mut Vec<Diagnostic>,
    seen: &mut BTreeSet<(u32, u32, u32)>,
) {
    for item in &expr.items {
        match item {
            PrefixItem::Block(block, _) => validate_block_gates(block, target, profile, out, seen),
            PrefixItem::Match(match_expr, _) => {
                validate_match_gates(match_expr, target, profile, out, seen);
            }
            PrefixItem::Tuple(items, _) => {
                for item in items {
                    validate_prefix_gates(item, target, profile, out, seen);
                }
            }
            PrefixItem::Group(inner, _) => {
                validate_prefix_gates(inner, target, profile, out, seen);
            }
            PrefixItem::Intrinsic(intrinsic, _) => {
                validate_intrinsic_gates(intrinsic, target, profile, out, seen);
            }
            _ => {}
        }
    }
}

fn validate_match_gates(
    match_expr: &MatchExpr,
    target: CompileTarget,
    profile: BuildProfile,
    out: &mut Vec<Diagnostic>,
    seen: &mut BTreeSet<(u32, u32, u32)>,
) {
    validate_prefix_gates(&match_expr.scrutinee, target, profile, out, seen);
    for arm in &match_expr.arms {
        validate_block_gates(&arm.body, target, profile, out, seen);
    }
}

fn validate_intrinsic_gates(
    intrinsic: &IntrinsicExpr,
    target: CompileTarget,
    profile: BuildProfile,
    out: &mut Vec<Diagnostic>,
    seen: &mut BTreeSet<(u32, u32, u32)>,
) {
    for arg in &intrinsic.args {
        validate_prefix_gates(arg, target, profile, out, seen);
    }
}

fn push_invalid_gate_diagnostic(
    out: &mut Vec<Diagnostic>,
    seen: &mut BTreeSet<(u32, u32, u32)>,
    directive: &Directive,
    target: CompileTarget,
    profile: BuildProfile,
) {
    if !matches!(
        directive_gate_decision(directive, target, profile),
        Some(GateDecision::Invalid)
    ) {
        return;
    }
    let Some(diagnostic) = invalid_gate_diagnostic(directive) else {
        return;
    };
    let span = diagnostic.primary.span;
    if seen.insert((span.file_id.0, span.start, span.end)) {
        out.push(diagnostic);
    }
}

fn invalid_conditional_gate_diagnostic(span: Span) -> Diagnostic {
    Diagnostic::error_with_code(
        DiagnosticCode::Loader(LoaderDiagnosticCode::ConditionalGateInvalid),
        "invalid conditional compilation gate",
        span,
    )
}

struct TargetGateParser<'a> {
    src: &'a [u8],
    pos: usize,
    ok: bool,
    active: CompileTarget,
}

impl<'a> TargetGateParser<'a> {
    fn new(src: &'a str, active: CompileTarget) -> Self {
        Self {
            src: src.as_bytes(),
            pos: 0,
            ok: true,
            active,
        }
    }

    fn skip_ws(&mut self) {
        while self.pos < self.src.len() && self.src[self.pos].is_ascii_whitespace() {
            self.pos += 1;
        }
    }

    fn parse_or(&mut self) -> bool {
        let mut value = self.parse_and();
        loop {
            self.skip_ws();
            if self.pos < self.src.len() && self.src[self.pos] == b'|' {
                self.pos += 1;
                let rhs = self.parse_and();
                value = value || rhs;
            } else {
                break;
            }
        }
        value
    }

    fn parse_and(&mut self) -> bool {
        let mut value = self.parse_primary();
        loop {
            self.skip_ws();
            if self.pos < self.src.len() && self.src[self.pos] == b'&' {
                self.pos += 1;
                let rhs = self.parse_primary();
                value = value && rhs;
            } else {
                break;
            }
        }
        value
    }

    fn parse_primary(&mut self) -> bool {
        self.skip_ws();
        if self.pos >= self.src.len() {
            self.ok = false;
            return false;
        }
        match self.src[self.pos] {
            b'(' => {
                self.pos += 1;
                let value = self.parse_or();
                self.skip_ws();
                if self.pos >= self.src.len() || self.src[self.pos] != b')' {
                    self.ok = false;
                    return false;
                }
                self.pos += 1;
                value
            }
            b'a'..=b'z' | b'A'..=b'Z' | b'_' => {
                let start = self.pos;
                self.pos += 1;
                while self.pos < self.src.len() {
                    let c = self.src[self.pos];
                    if c.is_ascii_alphanumeric() || c == b'_' {
                        self.pos += 1;
                    } else {
                        break;
                    }
                }
                let name = core::str::from_utf8(&self.src[start..self.pos]).unwrap_or("");
                match target_gate_atom_allows(name, self.active) {
                    Some(value) => value,
                    None => {
                        self.ok = false;
                        false
                    }
                }
            }
            _ => {
                self.ok = false;
                false
            }
        }
    }
}

fn target_gate_atom_allows(gate: &str, active: CompileTarget) -> Option<bool> {
    let allowed = match gate {
        "wasm" => matches!(
            active,
            CompileTarget::Wasm | CompileTarget::Wasi | CompileTarget::Wasix
        ),
        "bare_wasm" => matches!(active, CompileTarget::Wasm),
        "wasi" => matches!(active, CompileTarget::Wasi | CompileTarget::Wasix),
        "wasix" => matches!(active, CompileTarget::Wasix),
        "llvm" => matches!(active, CompileTarget::Llvm),
        "core" => matches!(
            active,
            CompileTarget::Wasm | CompileTarget::Wasi | CompileTarget::Llvm
        ),
        "std" => matches!(
            active,
            CompileTarget::Wasi | CompileTarget::Wasix | CompileTarget::Llvm
        ),
        "linux" => matches!(active, CompileTarget::Llvm),
        "win" | "windows" => false,
        "mac" | "darwin" | "macos" => false,
        _ => return None,
    };
    Some(allowed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn target_gate_supports_boolean_expressions() {
        assert_eq!(
            evaluate_target_gate_expr("wasm & (wasi | bare_wasm)", CompileTarget::Wasi),
            GateDecision::Active
        );
        assert_eq!(
            evaluate_target_gate_expr("llvm | bare_wasm", CompileTarget::Wasi),
            GateDecision::Inactive
        );
    }

    #[test]
    fn invalid_target_gate_is_not_silently_inactive() {
        assert_eq!(
            evaluate_target_gate_expr("unknown_target", CompileTarget::Wasm),
            GateDecision::Invalid
        );
        assert_eq!(
            evaluate_target_gate_expr("wasm &", CompileTarget::Wasm),
            GateDecision::Invalid
        );
    }

    #[test]
    fn invalid_profile_gate_is_not_silently_inactive() {
        assert_eq!(
            evaluate_profile_gate("staging", BuildProfile::Debug),
            GateDecision::Invalid
        );
    }
}
