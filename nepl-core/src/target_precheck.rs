//! target/profile 条件を反映した raw body 事前検証。
//!
//! codegen backend に入る前段で `#wasm` / `#llvmir` の有効性を共通検証し、
//! backend ごとの差分診断を減らすために利用する。

extern crate alloc;

use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

use crate::ast::{Block, Directive, FnBody, FnDef, LlvmIrBlock, Module, Stmt, WasmBlock};
use crate::compiler::{BuildProfile, CompileTarget};
use crate::diagnostic::Diagnostic;
use crate::diagnostic_codes::{DiagnosticCode, EffectDiagnosticCode, LoaderDiagnosticCode};
use crate::span::Span;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RawBodyKind {
    Wasm,
    LlvmIr,
}

#[derive(Debug, Clone, Copy)]
pub enum ActiveRawBody<'a> {
    Wasm(&'a WasmBlock),
    LlvmIr(&'a LlvmIrBlock),
}

impl<'a> ActiveRawBody<'a> {
    pub fn kind(&self) -> RawBodyKind {
        match self {
            ActiveRawBody::Wasm(_) => RawBodyKind::Wasm,
            ActiveRawBody::LlvmIr(_) => RawBodyKind::LlvmIr,
        }
    }

    pub fn span(&self) -> Span {
        match self {
            ActiveRawBody::Wasm(b) => b.span,
            ActiveRawBody::LlvmIr(b) => b.span,
        }
    }
}

pub fn gate_allows(
    directive: &Directive,
    target: CompileTarget,
    active_profile: BuildProfile,
) -> Option<bool> {
    crate::target_gate::directive_gate_allows(directive, target, active_profile)
}

pub fn active_stmt_indices(
    block: &Block,
    target: CompileTarget,
    profile: BuildProfile,
) -> Vec<usize> {
    let mut pending_if: Option<bool> = None;
    let mut out = Vec::new();
    for (idx, stmt) in block.items.iter().enumerate() {
        if let Stmt::Directive(d) = stmt {
            if let Some(allowed) = gate_allows(d, target, profile) {
                pending_if = Some(allowed);
                continue;
            }
        }
        let allowed = pending_if.unwrap_or(true);
        pending_if = None;
        if allowed {
            out.push(idx);
        }
    }
    out
}

pub fn select_active_raw_body<'a>(
    block: &'a Block,
    target: CompileTarget,
    profile: BuildProfile,
    owner_name: &str,
) -> Result<Option<ActiveRawBody<'a>>, Diagnostic> {
    let mut selected: Option<ActiveRawBody<'a>> = None;
    for idx in active_stmt_indices(block, target, profile) {
        match &block.items[idx] {
            Stmt::Wasm(w) => {
                if selected.is_some() {
                    return Err(multiple_active_raw_bodies_diagnostic(w.span, owner_name));
                }
                selected = Some(ActiveRawBody::Wasm(w));
            }
            Stmt::LlvmIr(l) => {
                if selected.is_some() {
                    return Err(multiple_active_raw_bodies_diagnostic(l.span, owner_name));
                }
                selected = Some(ActiveRawBody::LlvmIr(l));
            }
            Stmt::Directive(_) => {}
            _ => return Ok(None),
        }
    }
    Ok(selected)
}

pub fn is_raw_body_allowed_for_target(kind: RawBodyKind, target: CompileTarget) -> bool {
    match target {
        CompileTarget::Llvm => matches!(kind, RawBodyKind::LlvmIr),
        CompileTarget::Wasm | CompileTarget::Wasi | CompileTarget::Wasix => {
            matches!(kind, RawBodyKind::Wasm)
        }
    }
}

fn target_name(target: CompileTarget) -> &'static str {
    match target {
        CompileTarget::Wasm => "wasm",
        CompileTarget::Wasi => "wasi",
        CompileTarget::Wasix => "wasix",
        CompileTarget::Llvm => "llvm",
    }
}

fn raw_name(kind: RawBodyKind) -> &'static str {
    match kind {
        RawBodyKind::Wasm => "wasm",
        RawBodyKind::LlvmIr => "llvmir",
    }
}

fn effect_error(code: EffectDiagnosticCode, message: impl Into<String>, span: Span) -> Diagnostic {
    Diagnostic::error_with_code(DiagnosticCode::Effect(code), message, span)
}

fn loader_error(code: LoaderDiagnosticCode, message: impl Into<String>, span: Span) -> Diagnostic {
    Diagnostic::error_with_code(DiagnosticCode::Loader(code), message, span)
}

pub fn multiple_active_raw_bodies_diagnostic(span: Span, owner_name: &str) -> Diagnostic {
    effect_error(
        EffectDiagnosticCode::RawBodyMultipleActive,
        format!(
            "multiple active raw bodies are not allowed in function '{}'",
            owner_name
        ),
        span,
    )
}

pub fn raw_body_target_mismatch_diagnostic(
    span: Span,
    owner_name: &str,
    target: CompileTarget,
    raw_kind: RawBodyKind,
) -> Diagnostic {
    effect_error(
        EffectDiagnosticCode::RawBodyTargetMismatch,
        format!(
            "function '{}' uses #{} body, but #target {} does not allow it",
            owner_name,
            raw_name(raw_kind),
            target_name(target)
        ),
        span,
    )
}

pub fn precheck_function_raw_body_target(
    function: &FnDef,
    target: CompileTarget,
    profile: BuildProfile,
) -> Vec<Diagnostic> {
    let mut out = Vec::new();
    match &function.body {
        FnBody::Parsed(block) => {
            match select_active_raw_body(block, target, profile, function.name.name.as_str()) {
                Ok(Some(raw)) => {
                    if !is_raw_body_allowed_for_target(raw.kind(), target) {
                        out.push(raw_body_target_mismatch_diagnostic(
                            raw.span(),
                            function.name.name.as_str(),
                            target,
                            raw.kind(),
                        ));
                    }
                }
                Ok(None) => {}
                Err(diag) => out.push(diag),
            }
        }
        FnBody::Wasm(w) => {
            if !is_raw_body_allowed_for_target(RawBodyKind::Wasm, target) {
                out.push(raw_body_target_mismatch_diagnostic(
                    w.span,
                    function.name.name.as_str(),
                    target,
                    RawBodyKind::Wasm,
                ));
            }
        }
        FnBody::LlvmIr(l) => {
            if !is_raw_body_allowed_for_target(RawBodyKind::LlvmIr, target) {
                out.push(raw_body_target_mismatch_diagnostic(
                    l.span,
                    function.name.name.as_str(),
                    target,
                    RawBodyKind::LlvmIr,
                ));
            }
        }
    }
    out
}

pub fn precheck_module_raw_bodies(
    module: &Module,
    target: CompileTarget,
    profile: BuildProfile,
) -> Vec<Diagnostic> {
    let mut out = Vec::new();
    for idx in active_stmt_indices(&module.root, target, profile) {
        if let Stmt::FnDef(function) = &module.root.items[idx] {
            out.extend(precheck_function_raw_body_target(function, target, profile));
        }
    }
    out
}

fn is_known_target_name(name: &str) -> bool {
    matches!(name, "wasm" | "core" | "wasi" | "std" | "wasix" | "llvm")
}

pub fn precheck_module_target_directives(module: &Module) -> Vec<Diagnostic> {
    let mut out = Vec::new();
    let mut found = false;
    let mut saw_target_directive = false;
    for d in &module.directives {
        if let Directive::Target { target, span } = d {
            saw_target_directive = true;
            if !is_known_target_name(target.as_str()) {
                out.push(loader_error(
                    LoaderDiagnosticCode::TargetUnknown,
                    "unknown target in #target",
                    *span,
                ));
            } else if found {
                out.push(loader_error(
                    LoaderDiagnosticCode::TargetMultipleDirective,
                    "multiple #target directives are not allowed",
                    *span,
                ));
            } else {
                found = true;
            }
        }
    }

    if !saw_target_directive {
        for stmt in &module.root.items {
            if let Stmt::Directive(Directive::Target { target, span }) = stmt {
                if !is_known_target_name(target.as_str()) {
                    out.push(loader_error(
                        LoaderDiagnosticCode::TargetUnknown,
                        "unknown target in #target",
                        *span,
                    ));
                } else if found {
                    out.push(loader_error(
                        LoaderDiagnosticCode::TargetMultipleDirective,
                        "multiple #target directives are not allowed",
                        *span,
                    ));
                } else {
                    found = true;
                }
            }
        }
    }
    out
}

pub fn precheck_module_before_codegen(
    module: &Module,
    target: CompileTarget,
    profile: BuildProfile,
) -> Vec<Diagnostic> {
    let mut out = precheck_module_target_directives(module);
    out.extend(crate::target_gate::validate_module_gates(
        module, target, profile,
    ));
    out.extend(precheck_module_raw_bodies(module, target, profile));
    out
}
