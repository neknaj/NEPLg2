//! LLVM IR 生成（core 側）
//!
//! このモジュールは AST から LLVM IR テキストを生成する責務のみを持つ。
//! clang 実行などのホスト依存処理は `nepl-cli` 側で扱う。

extern crate alloc;

use alloc::collections::{BTreeMap, BTreeSet};
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

use crate::ast::Directive;
use crate::ast::{Block, FnBody, Ident, Literal, Module, PrefixExpr, PrefixItem, Stmt, TypeExpr};
use crate::backend_scalar_type::BackendScalarType;
use crate::compiler::{self, BuildProfile, CompileTarget, PreparedLlvmProgram};
use crate::diagnostic::Diagnostic;
use crate::diagnostic_codes::DiagnosticCode;
use crate::hir::{
    FuncRef, HirBlock, HirBody, HirExpr, HirExprKind, HirFunction, HirMatchBindMode,
    HirMatchPattern, HirModule,
};
use crate::intrinsic_kinds::ScalarIntrinsicKind;
use crate::layout::{
    enum_payload_offset_bytes, intrinsic_storage_type, is_aggregate_storage_type,
    storage_align_bytes, storage_size_bytes,
};
use crate::llvm_ir::{
    collect_defined_functions_from_llvmir_block, parse_declared_or_defined_function_name,
};
use crate::resource_primitives::{CollectionSlotBorrowPrimitive, CollectionSlotLifecyclePrimitive};
use crate::runtime_helpers::{helper_base_name, helper_candidates, RuntimeHelperKind};
use crate::scalar_primitives::I32ArithmeticPrimitive;
use crate::source_map::SourceMap;
use crate::span::Span;
use crate::target_precheck::{self, ActiveRawBody};
use crate::types::{TypeCtx, TypeId, TypeKind};

mod aggregate;
mod scalar_intrinsic;
mod type_map;

use aggregate::aggregate_field_layout;
use type_map::llty_for_type;

/// LLVM IR 生成時のエラー。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LlvmCodegenError {
    MissingLlvmIrBlock,
    TypecheckFailed { reason: String },
    MissingEntryFunction { function: String },
    CodegenDiagnostic { diagnostic: Diagnostic },
}

impl core::fmt::Display for LlvmCodegenError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            LlvmCodegenError::MissingLlvmIrBlock => {
                write!(
                    f,
                    "llvm target requires at least one #llvmir block in module/function body"
                )
            }
            LlvmCodegenError::TypecheckFailed { reason } => {
                write!(
                    f,
                    "failed to typecheck module for llvm lowering: {}",
                    reason
                )
            }
            LlvmCodegenError::MissingEntryFunction { function } => write!(
                f,
                "entry function '{}' was not found in lowered module",
                function
            ),
            LlvmCodegenError::CodegenDiagnostic { diagnostic } => write!(
                f,
                "llvm codegen failed: {}",
                summarize_diagnostics_for_message(core::slice::from_ref(diagnostic))
            ),
        }
    }
}

fn llvm_codegen_error(
    message: impl Into<String>,
    span: Span,
    code: DiagnosticCode,
) -> LlvmCodegenError {
    LlvmCodegenError::CodegenDiagnostic {
        diagnostic: Diagnostic::error_with_code(code, message, span),
    }
}

macro_rules! llvm_codegen_bail {
    ($($arg:tt)*) => {{
        return Err(llvm_codegen_error(
            format!($($arg)*),
            Span::dummy(),
            DiagnosticCode::Backend(crate::diagnostic_codes::BackendDiagnosticCode::Llvm(crate::diagnostic_codes::LlvmDiagnosticCode::HirUnsupported)),
        ));
    }};
}

/// `#llvmir` ブロックを連結して LLVM IR テキストを生成する。
///
/// 現段階では手書き `#llvmir` を主経路とし、Parsed 関数は最小 subset のみ lower する。
pub fn emit_ll_from_module(module: &Module) -> Result<String, LlvmCodegenError> {
    emit_ll_from_module_for_target(module, CompileTarget::Llvm, BuildProfile::Debug, false)
}

/// `target/profile` 条件を評価しながら LLVM IR を生成する。
pub fn emit_ll_from_module_for_target(
    module: &Module,
    target: CompileTarget,
    profile: BuildProfile,
    minify: bool,
) -> Result<String, LlvmCodegenError> {
    emit_ll_from_module_for_target_with_source_map(module, target, profile, minify, None)
}

pub fn emit_ll_from_module_for_target_with_source_map(
    module: &Module,
    target: CompileTarget,
    profile: BuildProfile,
    minify: bool,
    source_map: Option<&SourceMap>,
) -> Result<String, LlvmCodegenError> {
    let mut out = String::new();
    let entry_names = collect_active_entry_names(module, target, profile);
    let prepared = compiler::prepare_module_for_llvm_codegen_with_source_map(
        module,
        target,
        profile,
        &entry_names,
        source_map,
    )
    .map_err(map_core_error_to_llvm_codegen_error)?;
    let reachable_hint = (!prepared.reachable_set.is_empty()).then_some(&prepared.reachable_set);
    let raw_call_requirements = collect_required_raw_calls_fixed_point(module, target, profile);
    let raw_name_counts = collect_active_ast_raw_name_counts(module, target, profile);
    let mut raw_canonical_taken: BTreeSet<String> = BTreeSet::new();
    let mut selected_raw_ll_sigs: BTreeSet<String> = BTreeSet::new();
    let mut emitted_functions: Vec<String> = Vec::new();
    for idx in target_precheck::active_stmt_indices(&module.root, target, profile) {
        let stmt = &module.root.items[idx];

        match stmt {
            Stmt::LlvmIr(block) => {
                collect_defined_functions_from_llvmir_block(block, &mut emitted_functions);
                append_llvmir_block(&mut out, block);
            }
            Stmt::FnDef(def) => match &def.body {
                FnBody::LlvmIr(block) => {
                    if !should_emit_ast_llvmir_fn(
                        def,
                        reachable_hint,
                        raw_call_requirements.as_slice(),
                    ) {
                        continue;
                    }
                    if !raw_call_requirements.is_empty() {
                        if let Some((ps, ret)) = ast_fn_signature_llty(&def.signature) {
                            let key =
                                raw_abi_signature_key(def.name.name.as_str(), ps.as_slice(), ret);
                            if selected_raw_ll_sigs.contains(key.as_str()) {
                                continue;
                            }
                            selected_raw_ll_sigs.insert(key);
                        }
                    }
                    let normalized = normalize_ast_raw_llvmir_block(
                        def,
                        block,
                        &raw_name_counts,
                        &mut raw_canonical_taken,
                    );
                    collect_defined_functions_from_llvmir_block(
                        &normalized,
                        &mut emitted_functions,
                    );
                    append_llvmir_block(&mut out, &normalized);
                }
                FnBody::Parsed(block) => {
                    if !is_ast_fn_reachable(def.name.name.as_str(), reachable_hint) {
                        continue;
                    }
                    match target_precheck::select_active_raw_body(
                        block,
                        target,
                        profile,
                        def.name.name.as_str(),
                    ) {
                        Ok(Some(ActiveRawBody::LlvmIr(raw))) => {
                            let normalized = normalize_ast_raw_llvmir_block(
                                def,
                                raw,
                                &raw_name_counts,
                                &mut raw_canonical_taken,
                            );
                            collect_defined_functions_from_llvmir_block(
                                &normalized,
                                &mut emitted_functions,
                            );
                            append_llvmir_block(&mut out, &normalized);
                        }
                        Ok(Some(ActiveRawBody::Wasm(_))) => {
                            return Err(llvm_codegen_error(
                                format!(
                                    "wasm raw body reached llvm codegen in function '{}'",
                                    def.name.name
                                ),
                                def.name.span,
                                DiagnosticCode::Backend(crate::diagnostic_codes::BackendDiagnosticCode::Llvm(crate::diagnostic_codes::LlvmDiagnosticCode::RawBodyMismatch)),
                            ));
                        }
                        Ok(None) => {
                            if let Some(lowered) = lower_parsed_fn_with_gates(
                                def.name.name.as_str(),
                                &def.signature,
                                &def.params,
                                block,
                                target,
                                profile,
                            ) {
                                emitted_functions.push(def.name.name.clone());
                                out.push_str(&lowered);
                                out.push('\n');
                            }
                        }
                        Err(diag) => {
                            return Err(LlvmCodegenError::CodegenDiagnostic { diagnostic: diag });
                        }
                    }
                }
                FnBody::Wasm(_) => {
                    if is_ast_fn_reachable(def.name.name.as_str(), reachable_hint) {
                        return Err(llvm_codegen_error(
                            format!(
                                "wasm function body reached llvm codegen in function '{}'",
                                def.name.name
                            ),
                            def.name.span,
                            DiagnosticCode::Backend(
                                crate::diagnostic_codes::BackendDiagnosticCode::Llvm(
                                    crate::diagnostic_codes::LlvmDiagnosticCode::RawBodyMismatch,
                                ),
                            ),
                        ));
                    }
                }
            },
            _ => {}
        }
    }

    if let Some(entry) = entry_names.last() {
        let mut resolved_entry = entry.clone();
        if !emitted_functions.iter().any(|n| n == entry) {
            resolved_entry = try_lower_entry_from_hir(
                &prepared,
                entry.as_str(),
                &mut out,
                &mut emitted_functions,
            )?;
        }
        if !emitted_functions.iter().any(|n| n == &resolved_entry) {
            return Err(LlvmCodegenError::MissingEntryFunction {
                function: resolved_entry.clone(),
            });
        }
        if emitted_functions.iter().any(|n| n == &resolved_entry)
            && resolved_entry != "main"
            && !emitted_functions.iter().any(|n| n == "main")
        {
            out.push_str(&format!(
                "define i32 @main() {{\nentry:\n  %0 = call i32 @{}()\n  ret i32 %0\n}}\n\n",
                resolved_entry
            ));
        }
    }

    let final_out = deduplicate_overloaded_llvm_symbols(out.as_str());
    if minify {
        Ok(minify_ll_text(&final_out))
    } else {
        Ok(final_out)
    }
}

fn map_core_error_to_llvm_codegen_error(err: crate::error::CoreError) -> LlvmCodegenError {
    match err {
        crate::error::CoreError::Diagnostics(diags) => LlvmCodegenError::TypecheckFailed {
            reason: summarize_diagnostics_for_message(diags.as_slice()),
        },
        other => LlvmCodegenError::TypecheckFailed {
            reason: other.to_string(),
        },
    }
}

fn is_ast_fn_reachable(name: &str, reachable_hint: Option<&BTreeSet<String>>) -> bool {
    match reachable_hint {
        None => true,
        Some(set) => set.contains(name),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RawCallRequirement {
    name: String,
    params: Vec<LlTy>,
    ret: LlTy,
}

fn should_emit_ast_llvmir_fn(
    def: &crate::ast::FnDef,
    reachable_hint: Option<&BTreeSet<String>>,
    raw_reqs: &[RawCallRequirement],
) -> bool {
    if !is_ast_fn_reachable(def.name.name.as_str(), reachable_hint) {
        return false;
    }
    if raw_reqs.is_empty() {
        return true;
    }
    let reqs_for_name = raw_reqs
        .iter()
        .filter(|r| r.name == def.name.name)
        .collect::<Vec<_>>();
    if reqs_for_name.is_empty() {
        return false;
    }
    let Some((params, ret)) = ast_fn_signature_llty(&def.signature) else {
        return false;
    };
    reqs_for_name
        .iter()
        .any(|r| r.ret == ret && r.params.as_slice() == params.as_slice())
}

fn ast_fn_signature_llty(sig: &TypeExpr) -> Option<(Vec<LlTy>, LlTy)> {
    let TypeExpr::Function { params, result, .. } = sig.as_unspanned() else {
        return None;
    };
    let mut ps = Vec::new();
    for p in params {
        ps.push(llty_for_type_expr(p)?);
    }
    let ret = llty_for_type_expr(result.as_ref())?;
    Some((ps, ret))
}

fn llty_for_type_expr(ty: &TypeExpr) -> Option<LlTy> {
    match ty.as_unspanned() {
        TypeExpr::Unit | TypeExpr::Never => Some(LlTy::Void),
        TypeExpr::I32 | TypeExpr::U8 | TypeExpr::Bool | TypeExpr::Char | TypeExpr::Str => {
            Some(LlTy::I32)
        }
        TypeExpr::F32 => Some(LlTy::F32),
        TypeExpr::Named(_) => match BackendScalarType::from_type_expr(ty) {
            Some(scalar) if scalar.is_wasm_i64() => Some(LlTy::I64),
            Some(scalar) if scalar.is_wasm_f64() => Some(LlTy::F64),
            Some(_) | None => Some(LlTy::I32),
        },
        TypeExpr::Reference(_, _)
        | TypeExpr::Boxed(_)
        | TypeExpr::Tuple(_)
        | TypeExpr::Apply(_, _)
        | TypeExpr::Label(_) => Some(LlTy::I32),
        TypeExpr::Function { .. } => Some(LlTy::I32),
        TypeExpr::Spanned(inner, _) => llty_for_type_expr(inner),
    }
}

fn collect_required_raw_calls_fixed_point(
    module: &Module,
    target: CompileTarget,
    profile: BuildProfile,
) -> Vec<RawCallRequirement> {
    #[derive(Debug, Clone)]
    struct Candidate<'a> {
        name: &'a str,
        params: Vec<LlTy>,
        ret: LlTy,
        block: &'a crate::ast::LlvmIrBlock,
    }
    let mut candidates: Vec<Candidate<'_>> = Vec::new();
    let mut reqs = Vec::new();
    for idx in target_precheck::active_stmt_indices(&module.root, target, profile) {
        let stmt = &module.root.items[idx];
        match stmt {
            Stmt::LlvmIr(block) => {
                collect_call_requirements_from_llvmir_block(block, &mut reqs);
            }
            Stmt::FnDef(def) => match &def.body {
                FnBody::LlvmIr(block) => {
                    if let Some((params, ret)) = ast_fn_signature_llty(&def.signature) {
                        candidates.push(Candidate {
                            name: def.name.name.as_str(),
                            params,
                            ret,
                            block,
                        });
                    }
                }
                FnBody::Parsed(block) => {
                    if let Ok(Some(ActiveRawBody::LlvmIr(raw))) =
                        target_precheck::select_active_raw_body(
                            block,
                            target,
                            profile,
                            def.name.name.as_str(),
                        )
                    {
                        if let Some((params, ret)) = ast_fn_signature_llty(&def.signature) {
                            candidates.push(Candidate {
                                name: def.name.name.as_str(),
                                params,
                                ret,
                                block: raw,
                            });
                        }
                    }
                }
                FnBody::Wasm(_) => {}
            },
            _ => {}
        }
    }
    loop {
        let mut changed = false;
        for c in &candidates {
            let selected = reqs.iter().any(|r| {
                r.name == c.name && r.ret == c.ret && r.params.as_slice() == c.params.as_slice()
            });
            if !selected {
                continue;
            }
            let before = reqs.len();
            collect_call_requirements_from_llvmir_block(c.block, &mut reqs);
            if reqs.len() != before {
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }
    reqs
}

fn collect_active_ast_raw_name_counts(
    module: &Module,
    target: CompileTarget,
    profile: BuildProfile,
) -> BTreeMap<String, usize> {
    let mut out = BTreeMap::new();
    for idx in target_precheck::active_stmt_indices(&module.root, target, profile) {
        let stmt = &module.root.items[idx];
        let Some(def) = (match stmt {
            Stmt::FnDef(def) => Some(def),
            _ => None,
        }) else {
            continue;
        };
        let has_raw = match &def.body {
            FnBody::LlvmIr(_) => true,
            FnBody::Parsed(block) => matches!(
                target_precheck::select_active_raw_body(
                    block,
                    target,
                    profile,
                    def.name.name.as_str()
                ),
                Ok(Some(ActiveRawBody::LlvmIr(_)))
            ),
            FnBody::Wasm(_) => false,
        };
        if !has_raw {
            continue;
        }
        let entry = out.entry(def.name.name.clone()).or_insert(0);
        *entry += 1;
    }
    out
}

fn normalize_ast_raw_llvmir_block(
    def: &crate::ast::FnDef,
    block: &crate::ast::LlvmIrBlock,
    raw_name_counts: &BTreeMap<String, usize>,
    raw_canonical_taken: &mut BTreeSet<String>,
) -> crate::ast::LlvmIrBlock {
    let name = def.name.name.as_str();
    let Some(count) = raw_name_counts.get(name) else {
        return block.clone();
    };
    if *count <= 1 {
        return block.clone();
    }
    if !raw_canonical_taken.contains(name) {
        raw_canonical_taken.insert(String::from(name));
        return block.clone();
    }
    let suffix = ast_signature_suffix(&def.signature);
    let target_symbol = format!("{}__raw_{}", name, suffix);
    rewrite_llvmir_symbol(block, name, target_symbol.as_str())
}

fn ast_signature_suffix(sig: &TypeExpr) -> String {
    let Some((params, ret)) = ast_fn_signature_llty(sig) else {
        return String::from("unknown");
    };
    let mut s = String::new();
    for (i, p) in params.iter().enumerate() {
        if i > 0 {
            s.push('_');
        }
        s.push_str(p.ir());
    }
    s.push_str("_to_");
    s.push_str(ret.ir());
    s
}

fn rewrite_llvmir_symbol(
    block: &crate::ast::LlvmIrBlock,
    from: &str,
    to: &str,
) -> crate::ast::LlvmIrBlock {
    let from_plain = format!("@{}(", from);
    let from_quoted = format!("@\"{}\"(", from);
    let to_quoted = format!("@\"{}\"(", to);
    let mut lines = Vec::new();
    for line in &block.lines {
        let mut replaced = String::from(line);
        replaced = replaced.replace(from_plain.as_str(), to_quoted.as_str());
        replaced = replaced.replace(from_quoted.as_str(), to_quoted.as_str());
        lines.push(replaced);
    }
    crate::ast::LlvmIrBlock {
        lines,
        span: block.span,
    }
}

fn collect_call_requirements_from_llvmir_block(
    block: &crate::ast::LlvmIrBlock,
    out: &mut Vec<RawCallRequirement>,
) {
    for line in &block.lines {
        if let Some(req) = parse_llvm_call_requirement(line) {
            if !out.iter().any(|e| e == &req) {
                out.push(req);
            }
        }
    }
}

fn parse_llvm_call_requirement(line: &str) -> Option<RawCallRequirement> {
    let trimmed = line.trim();
    let call_idx = trimmed.find("call ")?;
    let rest = &trimmed[(call_idx + 5)..];
    let at = rest.find('@')?;
    let ret_str = rest[..at].trim();
    let ret = parse_llty_token(ret_str)?;
    let after_at = &rest[(at + 1)..];
    let open = after_at.find('(')?;
    let close = after_at.rfind(')')?;
    if close < open {
        return None;
    }
    let mut name = after_at[..open].trim();
    if name.starts_with('"') && name.ends_with('"') && name.len() >= 2 {
        name = &name[1..name.len() - 1];
    }
    if name.is_empty() {
        return None;
    }
    let args = &after_at[(open + 1)..close];
    let mut params = Vec::new();
    for raw_arg in args.split(',') {
        let arg = raw_arg.trim();
        if arg.is_empty() {
            continue;
        }
        let ty_tok = arg.split_whitespace().next()?;
        params.push(parse_llty_token(ty_tok)?);
    }
    Some(RawCallRequirement {
        name: String::from(name),
        params,
        ret,
    })
}

fn parse_llty_token(tok: &str) -> Option<LlTy> {
    match tok.trim() {
        "void" => Some(LlTy::Void),
        "i32" | "i8" | "i1" => Some(LlTy::I32),
        "i64" => Some(LlTy::I64),
        "float" => Some(LlTy::F32),
        "double" => Some(LlTy::F64),
        _ => None,
    }
}

fn deduplicate_overloaded_llvm_symbols(src: &str) -> String {
    #[derive(Debug, Clone)]
    struct DefLine {
        idx: usize,
        name: String,
        ret: LlTy,
        params: Vec<LlTy>,
    }
    let mut lines = src.lines().map(String::from).collect::<Vec<_>>();
    let mut defs_by_name: BTreeMap<String, Vec<DefLine>> = BTreeMap::new();
    for (idx, line) in lines.iter().enumerate() {
        if let Some((name, ret, params)) = parse_define_signature(line.as_str()) {
            defs_by_name
                .entry(String::from(name))
                .or_default()
                .push(DefLine {
                    idx,
                    name: String::from(name),
                    ret,
                    params,
                });
        }
    }

    let mut rename_map: BTreeMap<String, String> = BTreeMap::new();
    for (name, defs) in defs_by_name {
        if defs.len() <= 1 {
            continue;
        }
        let canonical = 0usize;
        let mut serial = 0usize;
        for (i, d) in defs.iter().enumerate() {
            if i == canonical {
                continue;
            }
            let new_name = format!(
                "{}__ov{}_{}",
                name,
                serial,
                raw_abi_signature_key("", d.params.as_slice(), d.ret)
                    .trim_start_matches('|')
                    .replace("->", "_to_")
            );
            serial += 1;
            rename_map.insert(
                ir_sig_key(d.name.as_str(), d.params.as_slice(), d.ret),
                new_name.clone(),
            );
            if let Some(line) = lines.get_mut(d.idx) {
                *line = replace_signature_symbol_name(
                    line.as_str(),
                    d.name.as_str(),
                    new_name.as_str(),
                );
            }
        }
    }
    if rename_map.is_empty() {
        return src.to_string();
    }
    for line in &mut lines {
        if let Some(req) = parse_llvm_call_requirement(line.as_str()) {
            let k = ir_sig_key(req.name.as_str(), req.params.as_slice(), req.ret);
            if let Some(new_name) = rename_map.get(k.as_str()) {
                *line =
                    replace_call_symbol_name(line.as_str(), req.name.as_str(), new_name.as_str());
            }
        }
    }
    lines.join("\n")
}

fn ir_sig_key(name: &str, params: &[LlTy], ret: LlTy) -> String {
    raw_abi_signature_key(name, params, ret)
}

fn parse_define_signature(line: &str) -> Option<(&str, LlTy, Vec<LlTy>)> {
    let trimmed = line.trim_start();
    if !trimmed.starts_with("define ") {
        return None;
    }
    let at = trimmed.find('@')?;
    let ret_str = trimmed["define ".len()..at].trim();
    let ret = parse_llty_token(ret_str)?;
    let rest = &trimmed[(at + 1)..];
    let open = rest.find('(')?;
    let close = rest.rfind(')')?;
    if close < open {
        return None;
    }
    let mut name = rest[..open].trim();
    if name.starts_with('"') && name.ends_with('"') && name.len() >= 2 {
        name = &name[1..name.len() - 1];
    }
    if name.is_empty() {
        return None;
    }
    let args = &rest[(open + 1)..close];
    let mut params = Vec::new();
    for raw_arg in args.split(',') {
        let arg = raw_arg.trim();
        if arg.is_empty() {
            continue;
        }
        let ty_tok = arg.split_whitespace().next()?;
        params.push(parse_llty_token(ty_tok)?);
    }
    Some((name, ret, params))
}

fn replace_signature_symbol_name(line: &str, from: &str, to: &str) -> String {
    let from_plain = format!("@{}(", from);
    let from_quoted = format!("@\"{}\"(", from);
    let to_quoted = format!("@\"{}\"(", to);
    let mut out = String::from(line);
    out = out.replace(from_plain.as_str(), to_quoted.as_str());
    out.replace(from_quoted.as_str(), to_quoted.as_str())
}

fn replace_call_symbol_name(line: &str, from: &str, to: &str) -> String {
    replace_signature_symbol_name(line, from, to)
}

fn raw_abi_signature_key(name: &str, params: &[LlTy], ret: LlTy) -> String {
    let mut s = String::from(name);
    s.push('|');
    for p in params {
        s.push_str(p.ir());
        s.push(',');
    }
    s.push_str("->");
    s.push_str(ret.ir());
    s
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum LlTy {
    Void,
    I32,
    I64,
    F32,
    F64,
}

impl LlTy {
    fn ir(self) -> &'static str {
        match self {
            LlTy::Void => "void",
            LlTy::I32 => "i32",
            LlTy::I64 => "i64",
            LlTy::F32 => "float",
            LlTy::F64 => "double",
        }
    }
}

#[derive(Debug, Clone)]
struct FnSig {
    params: Vec<LlTy>,
    ret: LlTy,
}

#[derive(Debug, Clone)]
struct LocalBinding {
    ptr: String,
    ty: LlTy,
}

#[derive(Debug, Clone)]
struct LlValue {
    ty: LlTy,
    repr: String,
}

struct LowerCtx<'a> {
    function_name: &'a str,
    sigs: &'a BTreeMap<String, FnSig>,
    function_ids: BTreeMap<String, i32>,
    reachable: &'a BTreeSet<String>,
    strings: &'a [String],
    memory_global: &'a str,
    fallback_alloc_symbol: Option<&'a str>,
    out: String,
    tmp_seq: usize,
    label_seq: usize,
    scopes: Vec<BTreeMap<String, LocalBinding>>,
}

impl<'a> LowerCtx<'a> {
    fn new(
        function_name: &'a str,
        sigs: &'a BTreeMap<String, FnSig>,
        reachable: &'a BTreeSet<String>,
        strings: &'a [String],
        memory_global: &'a str,
        fallback_alloc_symbol: Option<&'a str>,
    ) -> Self {
        let mut function_ids = BTreeMap::new();
        let mut next_id = 1i32;
        for name in sigs.keys() {
            function_ids.insert(name.clone(), next_id);
            next_id += 1;
        }
        Self {
            function_name,
            sigs,
            function_ids,
            reachable,
            strings,
            memory_global,
            fallback_alloc_symbol,
            out: String::new(),
            tmp_seq: 0,
            label_seq: 0,
            scopes: Vec::new(),
        }
    }

    fn push_line(&mut self, line: &str) {
        self.out.push_str(line);
        self.out.push('\n');
    }

    fn next_tmp(&mut self) -> String {
        let name = format!("%t{}", self.tmp_seq);
        self.tmp_seq += 1;
        name
    }

    fn next_label(&mut self, prefix: &str) -> String {
        let label = format!("{}_{}", prefix, self.label_seq);
        self.label_seq += 1;
        label
    }

    fn begin_scope(&mut self) {
        self.scopes.push(BTreeMap::new());
    }

    fn end_scope(&mut self) {
        self.scopes.pop();
    }

    fn bind_local(&mut self, name: &str, ptr: String, ty: LlTy) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name.to_string(), LocalBinding { ptr, ty });
        }
    }

    fn bind_zero_sized_local(&mut self, name: &str) {
        self.bind_local(name, String::new(), LlTy::Void);
    }

    fn lookup_local_current(&self, name: &str) -> Option<&LocalBinding> {
        self.scopes.last().and_then(|scope| scope.get(name))
    }

    fn lookup_local(&self, name: &str) -> Option<&LocalBinding> {
        for scope in self.scopes.iter().rev() {
            if let Some(v) = scope.get(name) {
                return Some(v);
            }
        }
        None
    }

    fn lookup_local_fuzzy(&self, name: &str) -> Option<&LocalBinding> {
        if let Some(v) = self.lookup_local(name) {
            return Some(v);
        }
        if let Some((base, _)) = name.split_once("__") {
            return self.lookup_local(base);
        }
        None
    }

    fn function_id_of(&self, name: &str) -> Option<i32> {
        self.function_ids.get(name).copied()
    }

    fn linear_i8_ptr_from_i32(&mut self, offset_i32: &str) -> String {
        let idx_i64 = self.next_tmp();
        let ptr_i8 = self.next_tmp();
        self.push_line(&format!("  {} = zext i32 {} to i64", idx_i64, offset_i32));
        self.push_line(&format!(
            "  {} = getelementptr [67108864 x i8], [67108864 x i8]* {}, i64 0, i64 {}",
            ptr_i8, self.memory_global, idx_i64
        ));
        ptr_i8
    }

    fn linear_typed_ptr_from_i32(&mut self, offset_i32: &str, ty: LlTy) -> String {
        let base_i8 = self.linear_i8_ptr_from_i32(offset_i32);
        let typed_ptr = self.next_tmp();
        self.push_line(&format!(
            "  {} = bitcast i8* {} to {}*",
            typed_ptr,
            base_i8,
            ty.ir()
        ));
        typed_ptr
    }
}

fn llvm_i8_ptr_at(ctx: &mut LowerCtx<'_>, base_ptr: &str, offset: i64) -> String {
    if offset == 0 {
        return base_ptr.to_string();
    }
    let out = ctx.next_tmp();
    ctx.push_line(&format!(
        "  {} = getelementptr i8, i8* {}, i64 {}",
        out, base_ptr, offset
    ));
    out
}

fn emit_zero_linear_bytes_llvm(ctx: &mut LowerCtx<'_>, base_ptr: &str, size: i32) {
    for off in 0..size {
        let dst = llvm_i8_ptr_at(ctx, base_ptr, off as i64);
        ctx.push_line(&format!("  store i8 0, i8* {}, align 1", dst));
    }
}

fn emit_copy_linear_bytes_llvm(
    ctx: &mut LowerCtx<'_>,
    dst_base: &str,
    dst_offset: i64,
    src_base: &str,
    src_offset: i64,
    size: i64,
) {
    for off in 0..size {
        let src = llvm_i8_ptr_at(ctx, src_base, src_offset + off);
        let byte = ctx.next_tmp();
        let dst = llvm_i8_ptr_at(ctx, dst_base, dst_offset + off);
        ctx.push_line(&format!("  {} = load i8, i8* {}, align 1", byte, src));
        ctx.push_line(&format!("  store i8 {}, i8* {}, align 1", byte, dst));
    }
}

fn try_lower_entry_from_hir(
    prepared: &PreparedLlvmProgram,
    entry: &str,
    out: &mut String,
    emitted_functions: &mut Vec<String>,
) -> Result<String, LlvmCodegenError> {
    let types = &prepared.program.types;
    let hir = &prepared.program.hir_module;
    let mut function_map: BTreeMap<String, &HirFunction> = BTreeMap::new();
    for f in &hir.functions {
        if crate::wasm_shared::should_skip_wasm_codegen_for_generic(types, f) {
            continue;
        }
        function_map.insert(f.name.clone(), f);
    }
    let Some(resolved_entry) = prepared.resolved_entries.get(entry) else {
        return Err(LlvmCodegenError::MissingEntryFunction {
            function: String::from(entry),
        });
    };

    let mut sigs = collect_hir_signatures(types, hir);
    let alloc_helper_symbol = resolve_runtime_helper_symbol(
        &sigs,
        helper_candidates(RuntimeHelperKind::Alloc),
        &[LlTy::I32],
        LlTy::I32,
    )
    .map(String::from);
    let fallback_alloc_symbol = alloc_helper_symbol
        .is_none()
        .then_some("__nepl_fallback_alloc");
    let mut backend_reachable_set = prepared.reachable_set.clone();
    if let Some(helper_name) = alloc_helper_symbol.as_ref() {
        extend_hir_reachable_from(
            hir,
            core::iter::once(helper_name.clone()),
            &mut backend_reachable_set,
        );
    }
    let memory_global = if fallback_alloc_symbol.is_some() {
        emit_fallback_linear_memory_runtime(out);
        "@__nepl_fallback_mem"
    } else {
        "@__nepl_mem"
    };

    let mut declared_extern_symbols: BTreeSet<String> = BTreeSet::new();
    for ex in &hir.externs {
        let local_name_raw = ex.local_name.as_str();
        let base_alias =
            find_mangled_signature_separator(local_name_raw).map(|sep| &local_name_raw[..sep]);
        let needs_base = base_alias
            .map(|base| backend_reachable_set.contains(base))
            .unwrap_or(false);

        let local_name = ll_symbol(ex.local_name.as_str());
        let external_name = ll_symbol(ex.name.as_str());
        let params_ll = ex
            .params
            .iter()
            .map(|t| llty_for_type(&types, *t).ir())
            .collect::<Vec<_>>()
            .join(", ");
        let ret = llty_for_type(&types, ex.result).ir();

        if !backend_reachable_set.contains(ex.local_name.as_str()) && !needs_base {
            continue;
        }

        if declared_extern_symbols.insert(ex.name.clone()) {
            out.push_str(&format!(
                "declare {} {}({})\n",
                ret, external_name, params_ll
            ));
        }

        if ex.local_name != ex.name {
            let args = ex
                .params
                .iter()
                .enumerate()
                .map(|(i, t)| format!("{} %a{}", llty_for_type(&types, *t).ir(), i))
                .collect::<Vec<_>>()
                .join(", ");
            let call_args = ex
                .params
                .iter()
                .enumerate()
                .map(|(i, t)| format!("{} %a{}", llty_for_type(&types, *t).ir(), i))
                .collect::<Vec<_>>()
                .join(", ");
            out.push_str(&format!("define {} {}({}) {{\n", ret, local_name, args));
            out.push_str("entry:\n");
            if ret == "void" {
                out.push_str(&format!(
                    "  call {} {}({})\n",
                    ret, external_name, call_args
                ));
                out.push_str("  ret void\n");
            } else {
                out.push_str(&format!(
                    "  %ret = call {} {}({})\n",
                    ret, external_name, call_args
                ));
                out.push_str(&format!("  ret {} %ret\n", ret));
            }
            out.push_str("}\n");
        }

        if !emitted_functions.iter().any(|n| n == &ex.local_name) {
            emitted_functions.push(ex.local_name.clone());
        }

        if needs_base {
            if let Some(base) = base_alias {
                if base != ex.local_name
                    && !llvm_output_has_function(out, base)
                    && !emitted_functions.iter().any(|n| n == base)
                {
                    let args = ex
                        .params
                        .iter()
                        .enumerate()
                        .map(|(i, t)| format!("{} %a{}", llty_for_type(&types, *t).ir(), i))
                        .collect::<Vec<_>>()
                        .join(", ");
                    let call_args = ex
                        .params
                        .iter()
                        .enumerate()
                        .map(|(i, t)| format!("{} %a{}", llty_for_type(&types, *t).ir(), i))
                        .collect::<Vec<_>>()
                        .join(", ");
                    let base_sym = ll_symbol(base);
                    out.push_str(&format!("define {} {}({}) {{\n", ret, base_sym, args));
                    out.push_str("entry:\n");
                    if ret == "void" {
                        out.push_str(&format!("  call {} {}({})\n", ret, local_name, call_args));
                        out.push_str("  ret void\n");
                    } else {
                        out.push_str(&format!(
                            "  %ret = call {} {}({})\n",
                            ret, local_name, call_args
                        ));
                        out.push_str(&format!("  ret {} %ret\n", ret));
                    }
                    out.push_str("}\n");
                    emitted_functions.push(String::from(base));
                }
            }
        }
    }
    if !prepared.reachable_set.is_empty() {
        out.push('\n');
    }

    for name in &backend_reachable_set {
        if emitted_functions.iter().any(|n| n == name) {
            continue;
        }
        let Some(func) = function_map.get(name.as_str()) else {
            continue;
        };
        match &func.body {
            HirBody::LlvmIr(raw) => {
                out.push_str(&format!("; nepl: function {} (raw llvmir)\n", name));
                let mut defined = Vec::new();
                collect_defined_functions_from_llvmir_block(raw, &mut defined);
                let defines_current_name = defined.iter().any(|d| d == name);
                let already_defined = !defined.is_empty()
                    && defined.iter().all(|n| {
                        emitted_functions.iter().any(|e| e == n)
                            || llvm_output_has_function(out, n.as_str())
                    });
                if already_defined {
                    if defines_current_name && !emitted_functions.iter().any(|n| n == name) {
                        emitted_functions.push(name.clone());
                    }
                    continue;
                }
                append_llvmir_block(out, raw);
                for def in defined {
                    if !emitted_functions.iter().any(|n| n == &def) {
                        emitted_functions.push(def);
                    }
                }
                if !defines_current_name {
                    if let Some(sig) = sigs.get(name.as_str()) {
                        if let Some(sep) = find_mangled_signature_separator(name.as_str()) {
                            let base_name = &name[..sep];
                            let _ = emit_alias_to_symbol(
                                name.as_str(),
                                base_name,
                                sig,
                                out,
                                emitted_functions,
                            );
                        }
                    }
                }
                if defines_current_name && !emitted_functions.iter().any(|n| n == name) {
                    emitted_functions.push(name.clone());
                }
            }
            HirBody::Wasm(_) => {
                llvm_codegen_bail!(
                    "internal compiler error: llvm lowering reached wasm body after precheck in function '{}'",
                    func.name
                );
            }
            HirBody::Block(block) => {
                out.push_str(&format!("; nepl: function {} (lowered block)\n", name));
                let lowered = lower_hir_function(
                    &types,
                    &hir,
                    &sigs,
                    &backend_reachable_set,
                    memory_global,
                    fallback_alloc_symbol,
                    func,
                    block,
                )?;
                out.push_str(&lowered);
                out.push('\n');
                emitted_functions.push(name.clone());
            }
        }
    }

    let emitted_snapshot = emitted_functions.clone();
    for name in emitted_snapshot {
        let Some(sig) = sigs.get(name.as_str()) else {
            continue;
        };
        let _ = emit_base_alias_to_mangled(name.as_str(), sig, &sigs, out, emitted_functions);
    }

    for (name, sig) in sigs.iter() {
        if emitted_functions.iter().any(|n| n == name) {
            continue;
        }
        let _ = emit_base_alias_for_mangled(name.as_str(), sig, out, emitted_functions);
    }

    if resolved_entry == "main" && emitted_functions.iter().any(|n| n == "__nepl_entry_main") {
        out.push_str(
            "define i32 @main() {\nentry:\n  call void @__nepl_entry_main()\n  ret i32 0\n}\n\n",
        );
        emitted_functions.push(String::from("main"));
    }
    if llvm_output_mentions_symbol(out, "alloc") && !llvm_output_has_function(out, "alloc") {
        if !llvm_output_has_function(out, "__nepl_fallback_alloc") {
            emit_fallback_linear_memory_runtime(out);
        }
        out.push_str("define i32 @alloc(i32 %size) {\nentry:\n");
        out.push_str("  %0 = call i32 @__nepl_fallback_alloc(i32 %size)\n");
        out.push_str("  ret i32 %0\n}\n\n");
        if !emitted_functions.iter().any(|n| n == "alloc") {
            emitted_functions.push(String::from("alloc"));
        }
    }

    // suppress unused warning when future passes extend signature synthesis
    sigs.clear();
    Ok(resolved_entry.clone())
}

fn emit_base_alias_for_mangled(
    mangled: &str,
    sig: &FnSig,
    out: &mut String,
    emitted_functions: &mut Vec<String>,
) -> bool {
    let Some(sep) = find_mangled_signature_separator(mangled) else {
        return false;
    };
    let base = &mangled[..sep];
    if base.is_empty() || base == mangled {
        return false;
    }
    emit_alias_to_symbol(mangled, base, sig, out, emitted_functions)
}

fn emit_alias_to_symbol(
    mangled: &str,
    base: &str,
    sig: &FnSig,
    out: &mut String,
    emitted_functions: &mut Vec<String>,
) -> bool {
    let base_available =
        emitted_functions.iter().any(|n| n == base) || llvm_output_has_function(out, base);
    if !base_available {
        return false;
    }
    if emitted_functions.iter().any(|n| n == mangled) || llvm_output_has_function(out, mangled) {
        return false;
    }
    let params = sig
        .params
        .iter()
        .enumerate()
        .map(|(i, ty)| format!("{} %p{}", ty.ir(), i))
        .collect::<Vec<_>>();
    let call_args = sig
        .params
        .iter()
        .enumerate()
        .map(|(i, ty)| format!("{} %p{}", ty.ir(), i))
        .collect::<Vec<_>>()
        .join(", ");
    out.push_str(&format!(
        "define {} {}({}) {{\nentry:\n",
        sig.ret.ir(),
        ll_symbol(mangled),
        params.join(", ")
    ));
    if sig.ret == LlTy::Void {
        out.push_str(&format!("  call void {}({})\n", ll_symbol(base), call_args));
        out.push_str("  ret void\n");
    } else {
        out.push_str(&format!(
            "  %0 = call {} {}({})\n",
            sig.ret.ir(),
            ll_symbol(base),
            call_args
        ));
        out.push_str(&format!("  ret {} %0\n", sig.ret.ir()));
    }
    out.push_str("}\n\n");
    emitted_functions.push(mangled.to_string());
    true
}

fn emit_base_alias_to_mangled(
    mangled: &str,
    sig: &FnSig,
    sigs: &BTreeMap<String, FnSig>,
    out: &mut String,
    emitted_functions: &mut Vec<String>,
) -> bool {
    let Some(sep) = find_mangled_signature_separator(mangled) else {
        return false;
    };
    let base = &mangled[..sep];
    if base.is_empty() || base == mangled {
        return false;
    }
    if emitted_functions.iter().any(|n| n == base) || llvm_output_has_function(out, base) {
        return false;
    }
    let variants = sigs
        .keys()
        .filter(|n| *n == base || n.starts_with(&format!("{}__", base)))
        .collect::<Vec<_>>();
    if variants.len() != 1 || variants[0].as_str() != mangled {
        return false;
    }
    let params = sig
        .params
        .iter()
        .enumerate()
        .map(|(i, ty)| format!("{} %p{}", ty.ir(), i))
        .collect::<Vec<_>>();
    let call_args = sig
        .params
        .iter()
        .enumerate()
        .map(|(i, ty)| format!("{} %p{}", ty.ir(), i))
        .collect::<Vec<_>>()
        .join(", ");
    out.push_str(&format!(
        "define {} {}({}) {{\nentry:\n",
        sig.ret.ir(),
        ll_symbol(base),
        params.join(", ")
    ));
    if sig.ret == LlTy::Void {
        out.push_str(&format!(
            "  call void {}({})\n",
            ll_symbol(mangled),
            call_args
        ));
        out.push_str("  ret void\n");
    } else {
        out.push_str(&format!(
            "  %0 = call {} {}({})\n",
            sig.ret.ir(),
            ll_symbol(mangled),
            call_args
        ));
        out.push_str(&format!("  ret {} %0\n", sig.ret.ir()));
    }
    out.push_str("}\n\n");
    emitted_functions.push(String::from(base));
    true
}

fn emit_fallback_linear_memory_runtime(out: &mut String) {
    out.push_str(
        "@__nepl_fallback_mem = internal global [67108864 x i8] zeroinitializer, align 16\n",
    );
    out.push_str("@__nepl_fallback_heap = internal global i32 16, align 4\n");
    out.push_str("define internal i32 @__nepl_fallback_alloc(i32 %size) {\n");
    out.push_str("entry:\n");
    out.push_str("  %ok = icmp sgt i32 %size, 0\n");
    out.push_str("  br i1 %ok, label %alloc, label %ret_zero\n");
    out.push_str("alloc:\n");
    out.push_str("  %heap0 = load i32, i32* @__nepl_fallback_heap, align 4\n");
    out.push_str("  %add = add i32 %size, 7\n");
    out.push_str("  %q = sdiv i32 %add, 8\n");
    out.push_str("  %aligned = mul i32 %q, 8\n");
    out.push_str("  %next = add i32 %heap0, %aligned\n");
    out.push_str("  store i32 %next, i32* @__nepl_fallback_heap, align 4\n");
    out.push_str("  ret i32 %heap0\n");
    out.push_str("ret_zero:\n");
    out.push_str("  ret i32 0\n");
    out.push_str("}\n\n");
}

fn find_mangled_signature_separator(name: &str) -> Option<usize> {
    let bytes = name.as_bytes();
    if bytes.len() < 3 {
        return None;
    }
    for i in 1..(bytes.len() - 1) {
        if bytes[i] == b'_' && bytes[i + 1] == b'_' {
            return Some(i);
        }
    }
    None
}

fn llvm_output_mentions_symbol(out: &str, sym: &str) -> bool {
    let plain = format!("@{}(", sym);
    let quoted = format!("@\"{}\"(", sym);
    out.contains(plain.as_str()) || out.contains(quoted.as_str())
}

fn collect_hir_signatures(types: &TypeCtx, module: &HirModule) -> BTreeMap<String, FnSig> {
    let mut out = BTreeMap::new();
    for f in &module.functions {
        if crate::wasm_shared::should_skip_wasm_codegen_for_generic(types, f) {
            continue;
        }
        let params = f
            .params
            .iter()
            .map(|p| llty_for_type(types, p.ty))
            .collect::<Vec<_>>();
        let ret = llty_for_type(types, f.result);
        out.insert(f.name.clone(), FnSig { params, ret });
    }
    for ex in &module.externs {
        let params = ex
            .params
            .iter()
            .map(|p| llty_for_type(types, *p))
            .collect::<Vec<_>>();
        let ret = llty_for_type(types, ex.result);
        out.insert(ex.local_name.clone(), FnSig { params, ret });
    }
    out
}

fn insert_hir_reachable_name(out: &mut BTreeSet<String>, name: String) {
    if let Some(sep) = find_mangled_signature_separator(name.as_str()) {
        out.insert(String::from(&name[..sep]));
    }
    out.insert(name);
}

fn extend_hir_reachable_from<I>(module: &HirModule, roots: I, out: &mut BTreeSet<String>)
where
    I: IntoIterator<Item = String>,
{
    let mut function_map: BTreeMap<String, &HirFunction> = BTreeMap::new();
    for f in &module.functions {
        function_map.insert(f.name.clone(), f);
    }
    let mut stack = roots.into_iter().collect::<Vec<_>>();
    let mut visited = BTreeSet::new();
    while let Some(name) = stack.pop() {
        if !visited.insert(name.clone()) {
            continue;
        }
        insert_hir_reachable_name(out, name.clone());
        let Some(func) = function_map.get(name.as_str()) else {
            continue;
        };
        collect_hir_called_functions_from_body(&func.body, &mut stack);
    }
}

fn collect_hir_called_functions_from_body(body: &HirBody, stack: &mut Vec<String>) {
    match body {
        HirBody::Block(block) => collect_hir_called_functions_from_block(block, stack),
        HirBody::Wasm(_) => {}
        HirBody::LlvmIr(block) => {
            for line in &block.lines {
                collect_hir_called_functions_from_llvm_line(line.as_str(), stack);
            }
        }
    }
}

fn collect_hir_called_functions_from_llvm_line(line: &str, stack: &mut Vec<String>) {
    let mut rest = line;
    while let Some(at_idx) = rest.find('@') {
        let after_at = &rest[at_idx + 1..];
        let Some(open_idx) = after_at.find('(') else {
            break;
        };
        let mut name = after_at[..open_idx].trim();
        if name.starts_with('"') && name.ends_with('"') && name.len() >= 2 {
            name = &name[1..name.len() - 1];
        }
        if !name.is_empty() {
            stack.push(String::from(name));
        }
        rest = &after_at[open_idx + 1..];
    }
}

fn collect_hir_called_functions_from_block(block: &HirBlock, stack: &mut Vec<String>) {
    for line in &block.lines {
        collect_hir_called_functions_from_expr(&line.expr, stack);
    }
}

fn collect_hir_called_functions_from_expr(expr: &HirExpr, stack: &mut Vec<String>) {
    match &expr.kind {
        HirExprKind::Call { callee, args } => {
            if let FuncRef::User(name, _, _) = callee {
                stack.push(name.clone());
            }
            for arg in args {
                collect_hir_called_functions_from_expr(arg, stack);
            }
        }
        HirExprKind::CallIndirect { callee, args, .. } => {
            collect_hir_called_functions_from_expr(callee, stack);
            for arg in args {
                collect_hir_called_functions_from_expr(arg, stack);
            }
        }
        HirExprKind::If {
            cond,
            then_branch,
            else_branch,
        } => {
            collect_hir_called_functions_from_expr(cond, stack);
            collect_hir_called_functions_from_expr(then_branch, stack);
            collect_hir_called_functions_from_expr(else_branch, stack);
        }
        HirExprKind::While { cond, body } => {
            collect_hir_called_functions_from_expr(cond, stack);
            collect_hir_called_functions_from_expr(body, stack);
        }
        HirExprKind::Match { scrutinee, arms } => {
            collect_hir_called_functions_from_expr(scrutinee, stack);
            for arm in arms {
                collect_hir_called_functions_from_expr(&arm.body, stack);
            }
        }
        HirExprKind::Block(block) => collect_hir_called_functions_from_block(block, stack),
        HirExprKind::Let { value, .. }
        | HirExprKind::Set { value, .. }
        | HirExprKind::AddrOf(value)
        | HirExprKind::Deref(value) => collect_hir_called_functions_from_expr(value, stack),
        HirExprKind::EnumConstruct { payload, .. } => {
            if let Some(payload) = payload {
                collect_hir_called_functions_from_expr(payload, stack);
            }
        }
        HirExprKind::StructConstruct { fields, .. }
        | HirExprKind::TupleConstruct { items: fields }
        | HirExprKind::Intrinsic { args: fields, .. } => {
            for field in fields {
                collect_hir_called_functions_from_expr(field, stack);
            }
        }
        HirExprKind::LiteralI32(_)
        | HirExprKind::LiteralF32(_)
        | HirExprKind::LiteralBool(_)
        | HirExprKind::LiteralStr(_)
        | HirExprKind::Unit
        | HirExprKind::Var(_)
        | HirExprKind::FnValue(_)
        | HirExprKind::MemoizedFunctionValue(_)
        | HirExprKind::Drop { .. } => {}
    }
}

fn lower_hir_function(
    types: &TypeCtx,
    module: &HirModule,
    sigs: &BTreeMap<String, FnSig>,
    reachable: &BTreeSet<String>,
    memory_global: &str,
    fallback_alloc_symbol: Option<&str>,
    func: &HirFunction,
    block: &HirBlock,
) -> Result<String, LlvmCodegenError> {
    let mut exported_name = func.name.clone();
    let mut ret_ty = llty_for_type(types, func.result);
    if func.name == "main" && matches!(ret_ty, LlTy::Void) {
        exported_name = String::from("__nepl_entry_main");
        ret_ty = LlTy::Void;
    }

    let mut ctx = LowerCtx::new(
        func.name.as_str(),
        sigs,
        reachable,
        &module.string_literals,
        memory_global,
        fallback_alloc_symbol,
    );
    let mut params = Vec::new();
    for (idx, p) in func.params.iter().enumerate() {
        let pty = llty_for_type(types, p.ty);
        params.push(format!("{} %p{}", pty.ir(), idx));
    }
    ctx.push_line(&format!(
        "define {} {}({}) {{",
        ret_ty.ir(),
        ll_symbol(exported_name.as_str()),
        params.join(", ")
    ));
    ctx.push_line("entry:");

    ctx.begin_scope();
    for (idx, p) in func.params.iter().enumerate() {
        let pty = llty_for_type(types, p.ty);
        let ptr = ctx.next_tmp();
        ctx.push_line(&format!("  {} = alloca {}", ptr, pty.ir()));
        ctx.push_line(&format!(
            "  store {} %p{}, {}* {}",
            pty.ir(),
            idx,
            pty.ir(),
            ptr
        ));
        ctx.bind_local(p.name.as_str(), ptr, pty);
    }

    let ret_val = lower_hir_block(types, &mut ctx, block)?;
    match ret_ty {
        LlTy::Void => {
            ctx.push_line("  ret void");
        }
        _ => {
            if let Some(v) = ret_val {
                if v.ty == ret_ty {
                    ctx.push_line(&format!("  ret {} {}", ret_ty.ir(), v.repr));
                } else {
                    llvm_codegen_bail!(
                        "internal compiler error: return type mismatch in '{}' ({:?} -> {:?})",
                        func.name,
                        v.ty,
                        ret_ty
                    );
                }
            } else {
                let zero = match ret_ty {
                    LlTy::I32 => "0",
                    LlTy::I64 => "0",
                    LlTy::F32 => "0.0",
                    LlTy::F64 => "0.0",
                    LlTy::Void => "",
                };
                ctx.push_line(&format!("  ret {} {}", ret_ty.ir(), zero));
            }
        }
    }
    ctx.end_scope();
    ctx.push_line("}");
    Ok(ctx.out)
}

fn lower_hir_block(
    types: &TypeCtx,
    ctx: &mut LowerCtx<'_>,
    block: &HirBlock,
) -> Result<Option<LlValue>, LlvmCodegenError> {
    ctx.begin_scope();
    predeclare_block_locals(types, ctx, block);
    let mut last = None;
    for line in &block.lines {
        let v = lower_hir_expr(types, ctx, &line.expr)?;
        if !line.drop_result {
            last = v;
        }
    }
    ctx.end_scope();
    Ok(last)
}

fn predeclare_block_locals(types: &TypeCtx, ctx: &mut LowerCtx<'_>, block: &HirBlock) {
    for line in &block.lines {
        if let HirExprKind::Let { name, value, .. } = &line.expr.kind {
            if ctx.lookup_local_current(name.as_str()).is_some() {
                continue;
            }
            let llty = llty_for_type(types, value.ty);
            if llty == LlTy::Void {
                ctx.bind_zero_sized_local(name.as_str());
                continue;
            }
            let ptr = ctx.next_tmp();
            ctx.push_line(&format!("  {} = alloca {}", ptr, llty.ir()));
            ctx.bind_local(name.as_str(), ptr, llty);
        }
    }
}

fn lower_i32_arithmetic_intrinsic(
    types: &TypeCtx,
    ctx: &mut LowerCtx<'_>,
    kind: I32ArithmeticPrimitive,
    args: &[HirExpr],
) -> Result<Option<LlValue>, LlvmCodegenError> {
    let Some(expected) = kind.codegen_argument_count() else {
        llvm_codegen_bail!(
            "internal compiler error: unsupported i32 arithmetic intrinsic in '{}'",
            ctx.function_name
        );
    };
    if args.len() != expected {
        llvm_codegen_bail!(
            "internal compiler error: i32 arithmetic intrinsic expects {} arguments in '{}'",
            expected,
            ctx.function_name
        );
    }
    let Some(a) = lower_hir_expr(types, ctx, &args[0])? else {
        llvm_codegen_bail!(
            "internal compiler error: i32 arithmetic intrinsic lhs must produce a value in '{}'",
            ctx.function_name
        );
    };
    let Some(b) = lower_hir_expr(types, ctx, &args[1])? else {
        llvm_codegen_bail!(
            "internal compiler error: i32 arithmetic intrinsic rhs must produce a value in '{}'",
            ctx.function_name
        );
    };
    if a.ty != LlTy::I32 || b.ty != LlTy::I32 {
        llvm_codegen_bail!(
            "internal compiler error: i32 arithmetic intrinsic supports i32 only in '{}' ({:?}, {:?})",
            ctx.function_name,
            a.ty,
            b.ty
        );
    }
    let op = match kind {
        I32ArithmeticPrimitive::Add => "add",
        I32ArithmeticPrimitive::Sub | I32ArithmeticPrimitive::Mul => {
            llvm_codegen_bail!(
                "internal compiler error: unsupported i32 arithmetic intrinsic in '{}'",
                ctx.function_name
            );
        }
    };
    let out = ctx.next_tmp();
    ctx.push_line(&format!("  {} = {} i32 {}, {}", out, op, a.repr, b.repr));
    Ok(Some(LlValue {
        ty: LlTy::I32,
        repr: out,
    }))
}

fn lower_hir_expr(
    types: &TypeCtx,
    ctx: &mut LowerCtx<'_>,
    expr: &HirExpr,
) -> Result<Option<LlValue>, LlvmCodegenError> {
    match &expr.kind {
        HirExprKind::LiteralI32(v) => Ok(Some(LlValue {
            ty: LlTy::I32,
            repr: format!("{}", v),
        })),
        HirExprKind::LiteralF32(v) => Ok(Some(LlValue {
            ty: LlTy::F32,
            repr: llvm_f32_literal(*v),
        })),
        HirExprKind::LiteralBool(v) => Ok(Some(LlValue {
            ty: LlTy::I32,
            repr: if *v {
                String::from("1")
            } else {
                String::from("0")
            },
        })),
        HirExprKind::LiteralStr(id) => lower_hir_string_literal(types, ctx, *id as usize),
        HirExprKind::Unit => Ok(None),
        HirExprKind::Var(name) => {
            let Some(binding) = ctx.lookup_local_fuzzy(name.as_str()) else {
                if let Some(fid) = ctx.function_id_of(name.as_str()) {
                    return Ok(Some(LlValue {
                        ty: LlTy::I32,
                        repr: format!("{}", fid),
                    }));
                }
                return Err(llvm_codegen_error(
                    format!("unknown variable '{}' reached llvm codegen", name),
                    expr.span,
                    DiagnosticCode::Backend(crate::diagnostic_codes::BackendDiagnosticCode::Llvm(
                        crate::diagnostic_codes::LlvmDiagnosticCode::VariableUnknown,
                    )),
                ));
            };
            let bty = binding.ty;
            if bty == LlTy::Void {
                return Ok(None);
            }
            let bptr = binding.ptr.clone();
            let tmp = ctx.next_tmp();
            ctx.push_line(&format!(
                "  {} = load {}, {}* {}",
                tmp,
                bty.ir(),
                bty.ir(),
                bptr
            ));
            Ok(Some(LlValue { ty: bty, repr: tmp }))
        }
        HirExprKind::Let { name, value, .. } => {
            let Some(v) = lower_hir_expr(types, ctx, value)? else {
                if llty_for_type(types, value.ty) == LlTy::Void
                    && ctx.lookup_local_fuzzy(name.as_str()).is_none()
                {
                    ctx.bind_zero_sized_local(name.as_str());
                }
                return Ok(None);
            };
            let (ptr, pty) = if let Some(binding) = ctx.lookup_local_fuzzy(name.as_str()).cloned() {
                (binding.ptr, binding.ty)
            } else {
                let ptr = ctx.next_tmp();
                ctx.push_line(&format!("  {} = alloca {}", ptr, v.ty.ir()));
                ctx.bind_local(name.as_str(), ptr.clone(), v.ty);
                (ptr, v.ty)
            };
            if v.ty != pty {
                llvm_codegen_bail!(
                    "internal compiler error: let type mismatch in llvm codegen ({:?} -> {:?})",
                    v.ty,
                    pty
                );
            }
            ctx.push_line(&format!(
                "  store {} {}, {}* {}",
                v.ty.ir(),
                v.repr,
                pty.ir(),
                ptr
            ));
            Ok(None)
        }
        HirExprKind::Set { name, value } => {
            let Some(binding) = ctx.lookup_local_fuzzy(name.as_str()).cloned() else {
                return Err(llvm_codegen_error(
                    format!("set on unknown variable '{}' reached llvm codegen", name),
                    expr.span,
                    DiagnosticCode::Backend(crate::diagnostic_codes::BackendDiagnosticCode::Llvm(
                        crate::diagnostic_codes::LlvmDiagnosticCode::VariableUnknown,
                    )),
                ));
            };
            let Some(v) = lower_hir_expr(types, ctx, value)? else {
                if binding.ty != LlTy::Void {
                    llvm_codegen_bail!(
                        "internal compiler error: set value did not produce a value in llvm codegen for '{}'",
                        name
                    );
                }
                return Ok(None);
            };
            if v.ty != binding.ty {
                llvm_codegen_bail!(
                    "internal compiler error: set type mismatch in llvm codegen ({:?} -> {:?})",
                    v.ty,
                    binding.ty
                );
            }
            ctx.push_line(&format!(
                "  store {} {}, {}* {}",
                v.ty.ir(),
                v.repr,
                binding.ty.ir(),
                binding.ptr
            ));
            Ok(None)
        }
        HirExprKind::FnValue(name) | HirExprKind::MemoizedFunctionValue(name) => {
            if let Some(fid) = ctx.function_id_of(name.as_str()) {
                Ok(Some(LlValue {
                    ty: LlTy::I32,
                    repr: format!("{}", fid),
                }))
            } else {
                Err(llvm_codegen_error(
                    format!("unknown function value '{}' reached llvm codegen", name),
                    expr.span,
                    DiagnosticCode::Backend(crate::diagnostic_codes::BackendDiagnosticCode::Llvm(
                        crate::diagnostic_codes::LlvmDiagnosticCode::FunctionValueUnknown,
                    )),
                ))
            }
        }
        HirExprKind::Call { callee, args } => {
            let callee_name = match callee {
                FuncRef::Builtin(name) | FuncRef::User(name, _, _) => name.as_str(),
                FuncRef::Trait {
                    application,
                    method,
                    ..
                } => {
                    return Err(llvm_codegen_error(
                        format!(
                            "unresolved trait call {}::{} reached llvm codegen",
                            application.display_name(types),
                            method.as_str()
                        ),
                        expr.span,
                        DiagnosticCode::Backend(
                            crate::diagnostic_codes::BackendDiagnosticCode::Llvm(
                                crate::diagnostic_codes::LlvmDiagnosticCode::FunctionUnknown,
                            ),
                        ),
                    ));
                }
            };
            let mut lowered_args = Vec::new();
            for a in args {
                if let Some(v) = lower_hir_expr(types, ctx, a)? {
                    lowered_args.push(v);
                }
            }
            let Some(sig) = ctx.sigs.get(callee_name) else {
                return Err(llvm_codegen_error(
                    format!(
                        "missing function signature for '{}' in llvm codegen",
                        callee_name
                    ),
                    expr.span,
                    DiagnosticCode::Backend(crate::diagnostic_codes::BackendDiagnosticCode::Llvm(
                        crate::diagnostic_codes::LlvmDiagnosticCode::FunctionUnknown,
                    )),
                ));
            };
            let mut args_ir = Vec::new();
            for (idx, v) in lowered_args.iter().enumerate() {
                let ty = sig.params.get(idx).copied().unwrap_or(v.ty);
                if ty != v.ty {
                    llvm_codegen_bail!(
                        "internal compiler error: call argument type mismatch in llvm codegen for '{}' ({:?} vs {:?})",
                        callee_name, ty, v.ty
                    );
                }
                args_ir.push(format!("{} {}", ty.ir(), v.repr));
            }
            match sig.ret {
                LlTy::Void => {
                    ctx.push_line(&format!(
                        "  call {} {}({})",
                        sig.ret.ir(),
                        ll_symbol(callee_name),
                        args_ir.join(", ")
                    ));
                    Ok(None)
                }
                ret => {
                    let tmp = ctx.next_tmp();
                    ctx.push_line(&format!(
                        "  {} = call {} {}({})",
                        tmp,
                        ret.ir(),
                        ll_symbol(callee_name),
                        args_ir.join(", ")
                    ));
                    Ok(Some(LlValue { ty: ret, repr: tmp }))
                }
            }
        }
        HirExprKind::CallIndirect {
            callee,
            params,
            result,
            args,
            ..
        } => {
            let Some(callee_v) = lower_hir_expr(types, ctx, callee)? else {
                llvm_codegen_bail!(
                    "internal compiler error: call_indirect callee must produce a value in '{}'",
                    ctx.function_name
                );
            };
            if callee_v.ty != LlTy::I32 {
                llvm_codegen_bail!(
                    "internal compiler error: call_indirect callee must be i32 function id in '{}'",
                    ctx.function_name
                );
            }

            let mut lowered_args = Vec::new();
            for a in args {
                let Some(v) = lower_hir_expr(types, ctx, a)? else {
                    llvm_codegen_bail!(
                        "internal compiler error: call_indirect argument must produce a value in '{}'",
                        ctx.function_name
                    );
                };
                lowered_args.push(v);
            }
            let param_ll = params
                .iter()
                .map(|p| llty_for_type(types, *p))
                .collect::<Vec<_>>();
            let ret_ll = llty_for_type(types, *result);
            if lowered_args.len() != param_ll.len() {
                llvm_codegen_bail!(
                    "internal compiler error: call_indirect argument length mismatch in '{}'",
                    ctx.function_name
                );
            }
            for (idx, v) in lowered_args.iter().enumerate() {
                if v.ty != param_ll[idx] {
                    llvm_codegen_bail!(
                        "internal compiler error: call_indirect argument type mismatch at {} in '{}' ({:?} vs {:?})",
                        idx, ctx.function_name, param_ll[idx], v.ty
                    );
                }
            }

            let mut candidates = Vec::new();
            for (name, sig) in ctx.sigs.iter() {
                if !ctx.reachable.contains(name) {
                    continue;
                }
                if sig.params == param_ll && sig.ret == ret_ll {
                    if let Some(fid) = ctx.function_id_of(name.as_str()) {
                        candidates.push((name.clone(), fid));
                    }
                }
            }
            if candidates.is_empty() {
                llvm_codegen_bail!(
                    "internal compiler error: call_indirect has no matching candidate in '{}'",
                    ctx.function_name
                );
            }

            let end_label = ctx.next_label("calli_end");
            let default_label = ctx.next_label("calli_default");
            let mut case_labels = Vec::new();
            for _ in &candidates {
                case_labels.push(ctx.next_label("calli_case"));
            }
            let result_slot = if ret_ll != LlTy::Void {
                let s = ctx.next_tmp();
                ctx.push_line(&format!("  {} = alloca {}", s, ret_ll.ir()));
                Some(s)
            } else {
                None
            };
            ctx.push_line(&format!(
                "  switch i32 {}, label %{} [",
                callee_v.repr, default_label
            ));
            for (idx, (_, fid)) in candidates.iter().enumerate() {
                ctx.push_line(&format!("    i32 {}, label %{}", fid, case_labels[idx]));
            }
            ctx.push_line("  ]");

            let args_ir = lowered_args
                .iter()
                .map(|v| format!("{} {}", v.ty.ir(), v.repr))
                .collect::<Vec<_>>()
                .join(", ");

            for (idx, (name, _)) in candidates.iter().enumerate() {
                ctx.push_line(&format!("{}:", case_labels[idx]));
                if ret_ll == LlTy::Void {
                    ctx.push_line(&format!(
                        "  call void {}({})",
                        ll_symbol(name.as_str()),
                        args_ir
                    ));
                } else {
                    let r = ctx.next_tmp();
                    ctx.push_line(&format!(
                        "  {} = call {} {}({})",
                        r,
                        ret_ll.ir(),
                        ll_symbol(name.as_str()),
                        args_ir
                    ));
                    if let Some(slot) = result_slot.as_ref() {
                        ctx.push_line(&format!(
                            "  store {} {}, {}* {}, align 1",
                            ret_ll.ir(),
                            r,
                            ret_ll.ir(),
                            slot
                        ));
                    }
                }
                ctx.push_line(&format!("  br label %{}", end_label));
            }
            ctx.push_line(&format!("{}:", default_label));
            ctx.push_line("  unreachable");
            ctx.push_line(&format!("{}:", end_label));
            if let Some(slot) = result_slot {
                let out = ctx.next_tmp();
                ctx.push_line(&format!(
                    "  {} = load {}, {}* {}, align 1",
                    out,
                    ret_ll.ir(),
                    ret_ll.ir(),
                    slot
                ));
                Ok(Some(LlValue {
                    ty: ret_ll,
                    repr: out,
                }))
            } else {
                Ok(None)
            }
        }
        HirExprKind::If {
            cond,
            then_branch,
            else_branch,
        } => {
            let Some(cond_v) = lower_hir_expr(types, ctx, cond)? else {
                llvm_codegen_bail!(
                    "internal compiler error: if condition must produce a value in '{}'",
                    ctx.function_name
                );
            };
            if cond_v.ty != LlTy::I32 {
                llvm_codegen_bail!(
                    "internal compiler error: if condition must be i32/bool-compatible in '{}' (got {:?})",
                    ctx.function_name, cond_v.ty
                );
            }
            let cond_i1 = ctx.next_tmp();
            ctx.push_line(&format!("  {} = icmp ne i32 {}, 0", cond_i1, cond_v.repr));
            let result_ty = llty_for_type(types, expr.ty);
            let result_slot = if result_ty != LlTy::Void {
                let slot = ctx.next_tmp();
                ctx.push_line(&format!("  {} = alloca {}", slot, result_ty.ir()));
                Some(slot)
            } else {
                None
            };
            let then_label = ctx.next_label("if_then");
            let else_label = ctx.next_label("if_else");
            let end_label = ctx.next_label("if_end");
            ctx.push_line(&format!(
                "  br i1 {}, label %{}, label %{}",
                cond_i1, then_label, else_label
            ));

            ctx.push_line(&format!("{}:", then_label));
            if let Some(tv) = lower_hir_expr(types, ctx, then_branch)? {
                if let Some(slot) = result_slot.as_ref() {
                    if tv.ty != result_ty {
                        llvm_codegen_bail!(
                            "internal compiler error: then branch result type mismatch in '{}' ({:?} vs {:?})",
                            ctx.function_name, tv.ty, result_ty
                        );
                    }
                    ctx.push_line(&format!(
                        "  store {} {}, {}* {}",
                        tv.ty.ir(),
                        tv.repr,
                        tv.ty.ir(),
                        slot
                    ));
                }
            }
            ctx.push_line(&format!("  br label %{}", end_label));

            ctx.push_line(&format!("{}:", else_label));
            if let Some(ev) = lower_hir_expr(types, ctx, else_branch)? {
                if let Some(slot) = result_slot.as_ref() {
                    if ev.ty != result_ty {
                        llvm_codegen_bail!(
                            "internal compiler error: else branch result type mismatch in '{}' ({:?} vs {:?})",
                            ctx.function_name, ev.ty, result_ty
                        );
                    }
                    ctx.push_line(&format!(
                        "  store {} {}, {}* {}",
                        ev.ty.ir(),
                        ev.repr,
                        ev.ty.ir(),
                        slot
                    ));
                }
            }
            ctx.push_line(&format!("  br label %{}", end_label));
            ctx.push_line(&format!("{}:", end_label));
            if let Some(slot) = result_slot {
                let tmp = ctx.next_tmp();
                ctx.push_line(&format!(
                    "  {} = load {}, {}* {}",
                    tmp,
                    result_ty.ir(),
                    result_ty.ir(),
                    slot
                ));
                Ok(Some(LlValue {
                    ty: result_ty,
                    repr: tmp,
                }))
            } else {
                Ok(None)
            }
        }
        HirExprKind::While { cond, body } => {
            let cond_label = ctx.next_label("while_cond");
            let body_label = ctx.next_label("while_body");
            let end_label = ctx.next_label("while_end");
            ctx.push_line(&format!("  br label %{}", cond_label));
            ctx.push_line(&format!("{}:", cond_label));
            let Some(cond_v) = lower_hir_expr(types, ctx, cond)? else {
                llvm_codegen_bail!(
                    "internal compiler error: while condition must produce a value in '{}'",
                    ctx.function_name
                );
            };
            if cond_v.ty != LlTy::I32 {
                llvm_codegen_bail!(
                    "internal compiler error: while condition must be i32/bool-compatible in '{}' (got {:?})",
                    ctx.function_name, cond_v.ty
                );
            }
            let cmp = ctx.next_tmp();
            ctx.push_line(&format!("  {} = icmp ne i32 {}, 0", cmp, cond_v.repr));
            ctx.push_line(&format!(
                "  br i1 {}, label %{}, label %{}",
                cmp, body_label, end_label
            ));
            ctx.push_line(&format!("{}:", body_label));
            let _ = lower_hir_expr(types, ctx, body)?;
            ctx.push_line(&format!("  br label %{}", cond_label));
            ctx.push_line(&format!("{}:", end_label));
            Ok(None)
        }
        HirExprKind::EnumConstruct {
            name: _,
            variant,
            payload,
            type_args: _,
        } => {
            let payload_offset = enum_payload_offset_bytes() as i64;
            let payload_storage_size = payload
                .as_ref()
                .map(|p| payload_offset + storage_size_bytes(types, p.ty) as i64)
                .unwrap_or(payload_offset);
            let total_size =
                ((storage_size_bytes(types, expr.ty) as i64).max(payload_storage_size)) as i32;

            let Some(alloc_name) = resolve_alloc_symbol(ctx) else {
                return Err(llvm_codegen_error(
                    format!(
                        "alloc function is required for enum construction in '{}'",
                        ctx.function_name
                    ),
                    Span::dummy(),
                    DiagnosticCode::Backend(crate::diagnostic_codes::BackendDiagnosticCode::Llvm(
                        crate::diagnostic_codes::LlvmDiagnosticCode::FunctionUnknown,
                    )),
                ));
            };

            let ptr = ctx.next_tmp();
            ctx.push_line(&format!(
                "  {} = call i32 {}(i32 {})",
                ptr,
                ll_symbol(alloc_name.as_str()),
                total_size
            ));

            let base_ptr8 = ctx.linear_i8_ptr_from_i32(ptr.as_str());
            emit_zero_linear_bytes_llvm(ctx, base_ptr8.as_str(), total_size);

            let tag = enum_variant_tag(types, expr.ty, variant.as_str());
            let tag_ptr = ctx.next_tmp();
            ctx.push_line(&format!(
                "  {} = bitcast i8* {} to i32*",
                tag_ptr, base_ptr8
            ));
            ctx.push_line(&format!("  store i32 {}, i32* {}, align 1", tag, tag_ptr));

            if let Some(p) = payload {
                let pv = lower_hir_expr(types, ctx, p)?;
                if is_aggregate_storage_type(types, p.ty) {
                    let Some(pv) = pv else {
                        llvm_codegen_bail!(
                            "internal compiler error: enum aggregate payload must produce a value in '{}'",
                            ctx.function_name
                        );
                    };
                    if pv.ty != LlTy::I32 {
                        llvm_codegen_bail!(
                            "internal compiler error: enum aggregate payload type mismatch in '{}' ({:?})",
                            ctx.function_name, pv.ty
                        );
                    }
                    let src_ptr8 = ctx.linear_i8_ptr_from_i32(pv.repr.as_str());
                    let payload_size = storage_size_bytes(types, p.ty) as i64;
                    emit_copy_linear_bytes_llvm(
                        ctx,
                        base_ptr8.as_str(),
                        payload_offset,
                        src_ptr8.as_str(),
                        0,
                        payload_size,
                    );
                } else {
                    let vty = llty_for_type(types, p.ty);
                    if vty == LlTy::Void {
                        return Ok(Some(LlValue {
                            ty: LlTy::I32,
                            repr: ptr,
                        }));
                    }
                    let Some(pv) = pv else {
                        llvm_codegen_bail!(
                            "internal compiler error: enum payload must produce a value in '{}'",
                            ctx.function_name
                        );
                    };
                    if pv.ty != vty {
                        llvm_codegen_bail!(
                            "internal compiler error: enum payload type mismatch in '{}' ({:?} vs {:?})",
                            ctx.function_name, vty, pv.ty
                        );
                    }
                    let payload_ptr8 = ctx.next_tmp();
                    let typed_ptr = ctx.next_tmp();
                    ctx.push_line(&format!(
                        "  {} = getelementptr i8, i8* {}, i64 {}",
                        payload_ptr8, base_ptr8, payload_offset
                    ));
                    ctx.push_line(&format!(
                        "  {} = bitcast i8* {} to {}*",
                        typed_ptr,
                        payload_ptr8,
                        vty.ir()
                    ));
                    ctx.push_line(&format!(
                        "  store {} {}, {}* {}, align 1",
                        vty.ir(),
                        pv.repr,
                        vty.ir(),
                        typed_ptr
                    ));
                }
            }

            Ok(Some(LlValue {
                ty: LlTy::I32,
                repr: ptr,
            }))
        }
        HirExprKind::StructConstruct {
            name: _,
            fields,
            type_args: _,
        } => {
            let mut offsets = Vec::with_capacity(fields.len());
            let mut total_size: i32 = 0;
            for f in fields.iter() {
                offsets.push(total_size as i64);
                total_size += storage_size_bytes(types, f.ty) as i32;
            }
            let Some(alloc_name) = resolve_alloc_symbol(ctx) else {
                return Err(llvm_codegen_error(
                    format!(
                        "alloc function is required for struct construction in '{}'",
                        ctx.function_name
                    ),
                    Span::dummy(),
                    DiagnosticCode::Backend(crate::diagnostic_codes::BackendDiagnosticCode::Llvm(
                        crate::diagnostic_codes::LlvmDiagnosticCode::FunctionUnknown,
                    )),
                ));
            };
            let ptr = ctx.next_tmp();
            ctx.push_line(&format!(
                "  {} = call i32 {}(i32 {})",
                ptr,
                ll_symbol(alloc_name.as_str()),
                total_size
            ));
            for (idx, f) in fields.iter().enumerate() {
                let field_ty = f.ty;
                let fty = llty_for_type(types, field_ty);
                let fv = lower_hir_expr(types, ctx, f)?;
                if is_aggregate_storage_type(types, field_ty) {
                    let Some(fv) = fv else {
                        llvm_codegen_bail!(
                            "internal compiler error: aggregate struct field must produce a value in '{}'",
                            ctx.function_name
                        );
                    };
                    if fv.ty != LlTy::I32 {
                        llvm_codegen_bail!(
                            "internal compiler error: aggregate struct field must lower to pointer in '{}'",
                            ctx.function_name
                        );
                    }
                    let base_ptr8 = ctx.linear_i8_ptr_from_i32(ptr.as_str());
                    let field_ptr8 = ctx.next_tmp();
                    ctx.push_line(&format!(
                        "  {} = getelementptr i8, i8* {}, i64 {}",
                        field_ptr8, base_ptr8, offsets[idx]
                    ));
                    let src_ptr8 = ctx.linear_i8_ptr_from_i32(fv.repr.as_str());
                    let size = storage_size_bytes(types, field_ty) as i64;
                    for off in 0..size {
                        let dst_byte_ptr = ctx.next_tmp();
                        let src_byte_ptr = ctx.next_tmp();
                        let byte_val = ctx.next_tmp();
                        ctx.push_line(&format!(
                            "  {} = getelementptr i8, i8* {}, i64 {}",
                            dst_byte_ptr, field_ptr8, off
                        ));
                        ctx.push_line(&format!(
                            "  {} = getelementptr i8, i8* {}, i64 {}",
                            src_byte_ptr, src_ptr8, off
                        ));
                        ctx.push_line(&format!(
                            "  {} = load i8, i8* {}, align 1",
                            byte_val, src_byte_ptr
                        ));
                        ctx.push_line(&format!(
                            "  store i8 {}, i8* {}, align 1",
                            byte_val, dst_byte_ptr
                        ));
                    }
                    continue;
                }
                if fty == LlTy::Void {
                    continue;
                }
                let Some(fv) = fv else {
                    llvm_codegen_bail!(
                        "internal compiler error: struct field must produce a value in '{}'",
                        ctx.function_name
                    );
                };
                if fv.ty != fty {
                    llvm_codegen_bail!(
                        "internal compiler error: struct field type mismatch in '{}' ({:?} vs {:?})",
                        ctx.function_name, fty, fv.ty
                    );
                }
                let base_ptr8 = ctx.linear_i8_ptr_from_i32(ptr.as_str());
                let field_ptr8 = ctx.next_tmp();
                let typed_ptr = ctx.next_tmp();
                ctx.push_line(&format!(
                    "  {} = getelementptr i8, i8* {}, i64 {}",
                    field_ptr8, base_ptr8, offsets[idx]
                ));
                ctx.push_line(&format!(
                    "  {} = bitcast i8* {} to {}*",
                    typed_ptr,
                    field_ptr8,
                    fty.ir()
                ));
                ctx.push_line(&format!(
                    "  store {} {}, {}* {}, align 1",
                    fty.ir(),
                    fv.repr,
                    fty.ir(),
                    typed_ptr
                ));
            }
            Ok(Some(LlValue {
                ty: LlTy::I32,
                repr: ptr,
            }))
        }
        HirExprKind::TupleConstruct { items } => {
            let mut offsets = Vec::with_capacity(items.len());
            let mut total_size: i32 = 0;
            for item in items.iter() {
                offsets.push(total_size as i64);
                total_size += storage_size_bytes(types, item.ty) as i32;
            }
            let Some(alloc_name) = resolve_alloc_symbol(ctx) else {
                return Err(llvm_codegen_error(
                    format!(
                        "alloc function is required for tuple construction in '{}'",
                        ctx.function_name
                    ),
                    Span::dummy(),
                    DiagnosticCode::Backend(crate::diagnostic_codes::BackendDiagnosticCode::Llvm(
                        crate::diagnostic_codes::LlvmDiagnosticCode::FunctionUnknown,
                    )),
                ));
            };
            let ptr = ctx.next_tmp();
            ctx.push_line(&format!(
                "  {} = call i32 {}(i32 {})",
                ptr,
                ll_symbol(alloc_name.as_str()),
                total_size
            ));
            for (idx, item) in items.iter().enumerate() {
                let item_ty = item.ty;
                let ity = llty_for_type(types, item_ty);
                let iv = lower_hir_expr(types, ctx, item)?;
                if is_aggregate_storage_type(types, item_ty) {
                    let Some(iv) = iv else {
                        llvm_codegen_bail!(
                            "internal compiler error: aggregate tuple item must produce a value in '{}'",
                            ctx.function_name
                        );
                    };
                    if iv.ty != LlTy::I32 {
                        llvm_codegen_bail!(
                            "internal compiler error: aggregate tuple item must lower to pointer in '{}'",
                            ctx.function_name
                        );
                    }
                    let base_ptr8 = ctx.linear_i8_ptr_from_i32(ptr.as_str());
                    let item_ptr8 = ctx.next_tmp();
                    ctx.push_line(&format!(
                        "  {} = getelementptr i8, i8* {}, i64 {}",
                        item_ptr8, base_ptr8, offsets[idx]
                    ));
                    let src_ptr8 = ctx.linear_i8_ptr_from_i32(iv.repr.as_str());
                    let size = storage_size_bytes(types, item_ty) as i64;
                    for off in 0..size {
                        let dst_byte_ptr = ctx.next_tmp();
                        let src_byte_ptr = ctx.next_tmp();
                        let byte_val = ctx.next_tmp();
                        ctx.push_line(&format!(
                            "  {} = getelementptr i8, i8* {}, i64 {}",
                            dst_byte_ptr, item_ptr8, off
                        ));
                        ctx.push_line(&format!(
                            "  {} = getelementptr i8, i8* {}, i64 {}",
                            src_byte_ptr, src_ptr8, off
                        ));
                        ctx.push_line(&format!(
                            "  {} = load i8, i8* {}, align 1",
                            byte_val, src_byte_ptr
                        ));
                        ctx.push_line(&format!(
                            "  store i8 {}, i8* {}, align 1",
                            byte_val, dst_byte_ptr
                        ));
                    }
                    continue;
                }
                if ity == LlTy::Void {
                    continue;
                }
                let Some(iv) = iv else {
                    llvm_codegen_bail!(
                        "internal compiler error: tuple item must produce a value in '{}'",
                        ctx.function_name
                    );
                };
                if iv.ty != ity {
                    llvm_codegen_bail!(
                        "internal compiler error: tuple item type mismatch in '{}' ({:?} vs {:?})",
                        ctx.function_name,
                        ity,
                        iv.ty
                    );
                }
                let base_ptr8 = ctx.linear_i8_ptr_from_i32(ptr.as_str());
                let item_ptr8 = ctx.next_tmp();
                let typed_ptr = ctx.next_tmp();
                ctx.push_line(&format!(
                    "  {} = getelementptr i8, i8* {}, i64 {}",
                    item_ptr8, base_ptr8, offsets[idx]
                ));
                ctx.push_line(&format!(
                    "  {} = bitcast i8* {} to {}*",
                    typed_ptr,
                    item_ptr8,
                    ity.ir()
                ));
                ctx.push_line(&format!(
                    "  store {} {}, {}* {}, align 1",
                    ity.ir(),
                    iv.repr,
                    ity.ir(),
                    typed_ptr
                ));
            }
            Ok(Some(LlValue {
                ty: LlTy::I32,
                repr: ptr,
            }))
        }
        HirExprKind::Match { scrutinee, arms } => {
            let Some(scr_v) = lower_hir_expr(types, ctx, scrutinee)? else {
                llvm_codegen_bail!(
                    "internal compiler error: match scrutinee must produce a value in '{}'",
                    ctx.function_name
                );
            };
            if scr_v.ty != LlTy::I32 {
                llvm_codegen_bail!(
                    "internal compiler error: match scrutinee must be enum pointer (i32) in '{}' (got {:?})",
                    ctx.function_name, scr_v.ty
                );
            }
            if arms.is_empty() {
                llvm_codegen_bail!(
                    "internal compiler error: match must have at least one arm in '{}'",
                    ctx.function_name
                );
            }

            let enum_match_ty = enum_match_type(types, scrutinee.ty);
            let is_enum_match = enum_match_ty.is_some()
                && arms.iter().all(|arm| {
                    matches!(
                        arm.pattern,
                        HirMatchPattern::Variant(_) | HirMatchPattern::Wildcard
                    )
                });
            let selector = if is_enum_match {
                let scr_ptr = ctx.linear_typed_ptr_from_i32(scr_v.repr.as_str(), LlTy::I32);
                let tag = ctx.next_tmp();
                ctx.push_line(&format!("  {} = load i32, i32* {}, align 1", tag, scr_ptr));
                tag
            } else {
                scr_v.repr.clone()
            };

            let result_ty = llty_for_type(types, expr.ty);
            let result_slot = if result_ty != LlTy::Void {
                let slot = ctx.next_tmp();
                ctx.push_line(&format!("  {} = alloca {}", slot, result_ty.ir()));
                Some(slot)
            } else {
                None
            };
            let end_label = ctx.next_label("match_end");
            let default_label = ctx.next_label("match_default");
            let mut arm_labels = Vec::with_capacity(arms.len());
            for _ in arms {
                arm_labels.push(ctx.next_label("match_arm"));
            }

            let wildcard_idx = arms
                .iter()
                .position(|arm| matches!(arm.pattern, HirMatchPattern::Wildcard));
            let switch_default = wildcard_idx
                .map(|idx| arm_labels[idx].clone())
                .unwrap_or_else(|| default_label.clone());

            ctx.push_line(&format!(
                "  switch i32 {}, label %{} [",
                selector, switch_default
            ));
            for (idx, arm) in arms.iter().enumerate() {
                match &arm.pattern {
                    HirMatchPattern::Variant(variant) if is_enum_match => {
                        let arm_tag =
                            enum_variant_tag(types, enum_match_ty.unwrap(), variant.as_str());
                        ctx.push_line(&format!("    i32 {}, label %{}", arm_tag, arm_labels[idx]));
                    }
                    HirMatchPattern::IntLiteral(value) => {
                        ctx.push_line(&format!("    i32 {}, label %{}", value, arm_labels[idx]));
                    }
                    HirMatchPattern::BoolLiteral(value) => {
                        let raw = if *value { 1 } else { 0 };
                        ctx.push_line(&format!("    i32 {}, label %{}", raw, arm_labels[idx]));
                    }
                    HirMatchPattern::Wildcard => {}
                    HirMatchPattern::Variant(_) => {}
                }
            }
            ctx.push_line("  ]");

            for (idx, arm) in arms.iter().enumerate() {
                ctx.push_line(&format!("{}:", arm_labels[idx]));
                ctx.begin_scope();
                if is_enum_match {
                    if let (HirMatchPattern::Variant(variant), Some(bind)) =
                        (&arm.pattern, &arm.bind_local)
                    {
                        if let Some(payload_ty) =
                            enum_variant_payload(types, enum_match_ty.unwrap(), variant)
                        {
                            let payload_ll = llty_for_type(types, payload_ty);
                            if matches!(arm.bind_mode, Some(HirMatchBindMode::Borrowed { .. })) {
                                let payload_offset = enum_payload_offset_bytes() as i64;
                                let payload_addr = if payload_offset == 0 {
                                    scr_v.repr.clone()
                                } else {
                                    let out = ctx.next_tmp();
                                    ctx.push_line(&format!(
                                        "  {} = add i32 {}, {}",
                                        out, scr_v.repr, payload_offset
                                    ));
                                    out
                                };
                                let local_ptr = ctx.next_tmp();
                                ctx.push_line(&format!("  {} = alloca i32", local_ptr));
                                ctx.push_line(&format!(
                                    "  store i32 {}, i32* {}, align 1",
                                    payload_addr, local_ptr
                                ));
                                ctx.bind_local(bind.as_str(), local_ptr, LlTy::I32);
                            } else if payload_ll == LlTy::Void {
                                ctx.bind_zero_sized_local(bind.as_str());
                            } else {
                                let payload_offset = enum_payload_offset_bytes() as i64;
                                let base_ptr8 = ctx.linear_i8_ptr_from_i32(scr_v.repr.as_str());
                                if is_aggregate_storage_type(types, payload_ty) {
                                    let Some(alloc_name) = resolve_alloc_symbol(ctx) else {
                                        return Err(llvm_codegen_error(
                                            format!(
                                                "alloc function is required for enum payload binding in '{}'",
                                                ctx.function_name
                                            ),
                                            Span::dummy(),
                                            DiagnosticCode::Backend(crate::diagnostic_codes::BackendDiagnosticCode::Llvm(crate::diagnostic_codes::LlvmDiagnosticCode::FunctionUnknown)),
                                        ));
                                    };
                                    let payload_size = storage_size_bytes(types, payload_ty) as i64;
                                    let payload_obj = ctx.next_tmp();
                                    ctx.push_line(&format!(
                                        "  {} = call i32 {}(i32 {})",
                                        payload_obj,
                                        ll_symbol(alloc_name.as_str()),
                                        payload_size
                                    ));
                                    let dst_ptr8 = ctx.linear_i8_ptr_from_i32(payload_obj.as_str());
                                    emit_copy_linear_bytes_llvm(
                                        ctx,
                                        dst_ptr8.as_str(),
                                        0,
                                        base_ptr8.as_str(),
                                        payload_offset,
                                        payload_size,
                                    );
                                    let local_ptr = ctx.next_tmp();
                                    ctx.push_line(&format!("  {} = alloca i32", local_ptr));
                                    ctx.push_line(&format!(
                                        "  store i32 {}, i32* {}, align 1",
                                        payload_obj, local_ptr
                                    ));
                                    ctx.bind_local(bind.as_str(), local_ptr, LlTy::I32);
                                } else {
                                    let payload_ptr8 = ctx.next_tmp();
                                    ctx.push_line(&format!(
                                        "  {} = getelementptr i8, i8* {}, i64 {}",
                                        payload_ptr8, base_ptr8, payload_offset
                                    ));

                                    let local_ptr = ctx.next_tmp();
                                    let local_val = if matches!(
                                        types.get(types.resolve_id(payload_ty)),
                                        TypeKind::U8
                                    ) {
                                        let p = ctx.next_tmp();
                                        let raw = ctx.next_tmp();
                                        let z = ctx.next_tmp();
                                        ctx.push_line(&format!(
                                            "  {} = bitcast i8* {} to i8*",
                                            p, payload_ptr8
                                        ));
                                        ctx.push_line(&format!(
                                            "  {} = load i8, i8* {}, align 1",
                                            raw, p
                                        ));
                                        ctx.push_line(&format!("  {} = zext i8 {} to i32", z, raw));
                                        z
                                    } else {
                                        let typed_ptr = ctx.next_tmp();
                                        let loaded = ctx.next_tmp();
                                        ctx.push_line(&format!(
                                            "  {} = bitcast i8* {} to {}*",
                                            typed_ptr,
                                            payload_ptr8,
                                            payload_ll.ir()
                                        ));
                                        ctx.push_line(&format!(
                                            "  {} = load {}, {}* {}, align 1",
                                            loaded,
                                            payload_ll.ir(),
                                            payload_ll.ir(),
                                            typed_ptr
                                        ));
                                        loaded
                                    };
                                    ctx.push_line(&format!(
                                        "  {} = alloca {}",
                                        local_ptr,
                                        payload_ll.ir()
                                    ));
                                    ctx.push_line(&format!(
                                        "  store {} {}, {}* {}, align 1",
                                        payload_ll.ir(),
                                        local_val,
                                        payload_ll.ir(),
                                        local_ptr
                                    ));
                                    ctx.bind_local(bind.as_str(), local_ptr, payload_ll);
                                }
                            }
                        }
                    }
                }

                let arm_val = lower_hir_expr(types, ctx, &arm.body)?;
                if let Some(slot) = result_slot.as_ref() {
                    let Some(v) = arm_val else {
                        ctx.end_scope();
                        continue;
                    };
                    if v.ty != result_ty {
                        llvm_codegen_bail!(
                            "internal compiler error: match arm result type mismatch in '{}' ({:?} vs {:?})",
                            ctx.function_name, result_ty, v.ty
                        );
                    }
                    ctx.push_line(&format!(
                        "  store {} {}, {}* {}, align 1",
                        v.ty.ir(),
                        v.repr,
                        v.ty.ir(),
                        slot
                    ));
                }
                ctx.end_scope();
                ctx.push_line(&format!("  br label %{}", end_label));
            }

            if wildcard_idx.is_none() {
                ctx.push_line(&format!("{}:", default_label));
                ctx.push_line("  unreachable");
            }

            ctx.push_line(&format!("{}:", end_label));
            if let Some(slot) = result_slot {
                let tmp = ctx.next_tmp();
                ctx.push_line(&format!(
                    "  {} = load {}, {}* {}, align 1",
                    tmp,
                    result_ty.ir(),
                    result_ty.ir(),
                    slot
                ));
                Ok(Some(LlValue {
                    ty: result_ty,
                    repr: tmp,
                }))
            } else {
                Ok(None)
            }
        }
        HirExprKind::Block(block) => lower_hir_block(types, ctx, block),
        HirExprKind::Intrinsic {
            name,
            type_args,
            args,
        } => {
            if name == "size_of" || name == "align_of" {
                if let Some(ty) = type_args.first() {
                    let size = if name == "size_of" {
                        storage_size_bytes(types, *ty) as i64
                    } else {
                        storage_align_bytes(types, *ty) as i64
                    };
                    return Ok(Some(LlValue {
                        ty: LlTy::I32,
                        repr: format!("{}", size),
                    }));
                }
            }
            if name == "load" {
                if type_args.len() != 1 || args.len() != 1 {
                    llvm_codegen_bail!(
                        "internal compiler error: intrinsic load requires one type arg and one value arg in '{}'",
                        ctx.function_name
                    );
                }
                let Some(ptr_v) = lower_hir_expr(types, ctx, &args[0])? else {
                    llvm_codegen_bail!(
                        "internal compiler error: intrinsic load pointer must produce a value in '{}'",
                        ctx.function_name
                    );
                };
                if ptr_v.ty != LlTy::I32 {
                    llvm_codegen_bail!(
                        "internal compiler error: intrinsic load pointer must be i32 in '{}' (got {:?})",
                        ctx.function_name, ptr_v.ty
                    );
                }
                let ty_id = intrinsic_storage_type(types, type_args[0], expr.ty);
                let ty_kind = types.get(ty_id);
                if is_aggregate_storage_type(types, ty_id) {
                    let size = storage_size_bytes(types, ty_id) as i64;
                    let Some(alloc_name) = resolve_alloc_symbol(ctx) else {
                        return Err(llvm_codegen_error(
                            format!(
                                "alloc function is required for intrinsic load in '{}'",
                                ctx.function_name
                            ),
                            Span::dummy(),
                            DiagnosticCode::Backend(
                                crate::diagnostic_codes::BackendDiagnosticCode::Llvm(
                                    crate::diagnostic_codes::LlvmDiagnosticCode::FunctionUnknown,
                                ),
                            ),
                        ));
                    };
                    let dst = ctx.next_tmp();
                    ctx.push_line(&format!(
                        "  {} = call i32 {}(i32 {})",
                        dst,
                        ll_symbol(alloc_name.as_str()),
                        size
                    ));
                    let dst_ptr = ctx.linear_i8_ptr_from_i32(dst.as_str());
                    let src_ptr = ctx.linear_i8_ptr_from_i32(ptr_v.repr.as_str());
                    for off in 0..size {
                        let src_byte_ptr = ctx.next_tmp();
                        let byte = ctx.next_tmp();
                        let dst_byte_ptr = ctx.next_tmp();
                        ctx.push_line(&format!(
                            "  {} = getelementptr i8, i8* {}, i64 {}",
                            src_byte_ptr, src_ptr, off
                        ));
                        ctx.push_line(&format!(
                            "  {} = load i8, i8* {}, align 1",
                            byte, src_byte_ptr
                        ));
                        ctx.push_line(&format!(
                            "  {} = getelementptr i8, i8* {}, i64 {}",
                            dst_byte_ptr, dst_ptr, off
                        ));
                        ctx.push_line(&format!(
                            "  store i8 {}, i8* {}, align 1",
                            byte, dst_byte_ptr
                        ));
                    }
                    return Ok(Some(LlValue {
                        ty: LlTy::I32,
                        repr: dst,
                    }));
                }
                if matches!(ty_kind, TypeKind::U8) {
                    let p_ptr = ctx.linear_i8_ptr_from_i32(ptr_v.repr.as_str());
                    let raw = ctx.next_tmp();
                    let out = ctx.next_tmp();
                    ctx.push_line(&format!("  {} = load i8, i8* {}, align 1", raw, p_ptr));
                    ctx.push_line(&format!("  {} = zext i8 {} to i32", out, raw));
                    return Ok(Some(LlValue {
                        ty: LlTy::I32,
                        repr: out,
                    }));
                }
                let out_ty = llty_for_type(types, ty_id);
                let p_ptr = ctx.linear_typed_ptr_from_i32(ptr_v.repr.as_str(), out_ty);
                let out = ctx.next_tmp();
                ctx.push_line(&format!(
                    "  {} = load {}, {}* {}, align 1",
                    out,
                    out_ty.ir(),
                    out_ty.ir(),
                    p_ptr
                ));
                return Ok(Some(LlValue {
                    ty: out_ty,
                    repr: out,
                }));
            }
            if name == "store" {
                if type_args.len() != 1 || args.len() != 2 {
                    llvm_codegen_bail!(
                        "internal compiler error: intrinsic store requires one type arg and two value args in '{}'",
                        ctx.function_name
                    );
                }
                let Some(ptr_v) = lower_hir_expr(types, ctx, &args[0])? else {
                    llvm_codegen_bail!(
                        "internal compiler error: intrinsic store pointer must produce a value in '{}'",
                        ctx.function_name
                    );
                };
                let Some(val_v) = lower_hir_expr(types, ctx, &args[1])? else {
                    llvm_codegen_bail!(
                        "internal compiler error: intrinsic store value must produce a value in '{}'",
                        ctx.function_name
                    );
                };
                if ptr_v.ty != LlTy::I32 {
                    llvm_codegen_bail!(
                        "internal compiler error: intrinsic store pointer must be i32 in '{}' (got {:?})",
                        ctx.function_name, ptr_v.ty
                    );
                }
                let ty_id = intrinsic_storage_type(types, type_args[0], args[1].ty);
                let ty_kind = types.get(ty_id);
                if is_aggregate_storage_type(types, ty_id) {
                    if val_v.ty != LlTy::I32 {
                        llvm_codegen_bail!(
                            "internal compiler error: intrinsic store aggregate expects i32 handle in '{}' (got {:?})",
                            ctx.function_name, val_v.ty
                        );
                    }
                    let dst_ptr = ctx.linear_i8_ptr_from_i32(ptr_v.repr.as_str());
                    let src_ptr = ctx.linear_i8_ptr_from_i32(val_v.repr.as_str());
                    let size = storage_size_bytes(types, ty_id) as i64;
                    for off in 0..size {
                        let src_byte_ptr = ctx.next_tmp();
                        let byte = ctx.next_tmp();
                        let dst_byte_ptr = ctx.next_tmp();
                        ctx.push_line(&format!(
                            "  {} = getelementptr i8, i8* {}, i64 {}",
                            src_byte_ptr, src_ptr, off
                        ));
                        ctx.push_line(&format!(
                            "  {} = load i8, i8* {}, align 1",
                            byte, src_byte_ptr
                        ));
                        ctx.push_line(&format!(
                            "  {} = getelementptr i8, i8* {}, i64 {}",
                            dst_byte_ptr, dst_ptr, off
                        ));
                        ctx.push_line(&format!(
                            "  store i8 {}, i8* {}, align 1",
                            byte, dst_byte_ptr
                        ));
                    }
                    return Ok(None);
                }
                if matches!(ty_kind, TypeKind::U8) {
                    if val_v.ty != LlTy::I32 {
                        llvm_codegen_bail!(
                            "internal compiler error: intrinsic store<u8> expects i32 value in '{}' (got {:?})",
                            ctx.function_name, val_v.ty
                        );
                    }
                    let p_ptr = ctx.linear_i8_ptr_from_i32(ptr_v.repr.as_str());
                    let b = ctx.next_tmp();
                    ctx.push_line(&format!("  {} = trunc i32 {} to i8", b, val_v.repr));
                    ctx.push_line(&format!("  store i8 {}, i8* {}, align 1", b, p_ptr));
                    return Ok(None);
                }
                let store_ty = llty_for_type(types, ty_id);
                if val_v.ty != store_ty {
                    llvm_codegen_bail!(
                        "internal compiler error: intrinsic store type mismatch in '{}' ({:?} vs {:?})",
                        ctx.function_name, store_ty, val_v.ty
                    );
                }
                let p_ptr = ctx.linear_typed_ptr_from_i32(ptr_v.repr.as_str(), store_ty);
                ctx.push_line(&format!(
                    "  store {} {}, {}* {}, align 1",
                    store_ty.ir(),
                    val_v.repr,
                    store_ty.ir(),
                    p_ptr
                ));
                return Ok(None);
            }
            if name == "unreachable" {
                ctx.push_line("  unreachable");
                return Ok(None);
            }
            if name == "get_field" {
                if args.len() != 2 {
                    llvm_codegen_bail!(
                        "internal compiler error: intrinsic get_field requires two args in '{}'",
                        ctx.function_name
                    );
                }
                let Some((_field_ty, offset)) =
                    aggregate_field_layout(types, ctx, args[0].ty, &args[1])
                else {
                    llvm_codegen_bail!(
                        "internal compiler error: unsupported get_field selector reached llvm lowering in '{}'",
                        ctx.function_name
                    );
                };
                let field_ty = expr.ty;
                let Some(base_v) = lower_hir_expr(types, ctx, &args[0])? else {
                    llvm_codegen_bail!(
                        "internal compiler error: intrinsic get_field base must produce a value in '{}'",
                        ctx.function_name
                    );
                };
                if base_v.ty != LlTy::I32 {
                    llvm_codegen_bail!(
                        "internal compiler error: intrinsic get_field base must be i32 in '{}' (got {:?})",
                        ctx.function_name, base_v.ty
                    );
                }
                if is_aggregate_storage_type(types, field_ty) {
                    let size = storage_size_bytes(types, field_ty) as i64;
                    let Some(alloc_name) = resolve_alloc_symbol(ctx) else {
                        return Err(llvm_codegen_error(
                            format!(
                                "alloc function is required for aggregate get_field in '{}'",
                                ctx.function_name
                            ),
                            Span::dummy(),
                            DiagnosticCode::Backend(
                                crate::diagnostic_codes::BackendDiagnosticCode::Llvm(
                                    crate::diagnostic_codes::LlvmDiagnosticCode::FunctionUnknown,
                                ),
                            ),
                        ));
                    };
                    let dst = ctx.next_tmp();
                    ctx.push_line(&format!(
                        "  {} = call i32 {}(i32 {})",
                        dst,
                        ll_symbol(alloc_name.as_str()),
                        size
                    ));
                    let dst_ptr8 = ctx.linear_i8_ptr_from_i32(dst.as_str());
                    let src_base8 = ctx.linear_i8_ptr_from_i32(base_v.repr.as_str());
                    let src_ptr8 = ctx.next_tmp();
                    ctx.push_line(&format!(
                        "  {} = getelementptr i8, i8* {}, i64 {}",
                        src_ptr8, src_base8, offset
                    ));
                    for off in 0..size {
                        let dst_byte_ptr = ctx.next_tmp();
                        let src_byte_ptr = ctx.next_tmp();
                        let byte_val = ctx.next_tmp();
                        ctx.push_line(&format!(
                            "  {} = getelementptr i8, i8* {}, i64 {}",
                            dst_byte_ptr, dst_ptr8, off
                        ));
                        ctx.push_line(&format!(
                            "  {} = getelementptr i8, i8* {}, i64 {}",
                            src_byte_ptr, src_ptr8, off
                        ));
                        ctx.push_line(&format!(
                            "  {} = load i8, i8* {}, align 1",
                            byte_val, src_byte_ptr
                        ));
                        ctx.push_line(&format!(
                            "  store i8 {}, i8* {}, align 1",
                            byte_val, dst_byte_ptr
                        ));
                    }
                    return Ok(Some(LlValue {
                        ty: LlTy::I32,
                        repr: dst,
                    }));
                }
                let out_ty = llty_for_type(types, field_ty);
                if out_ty == LlTy::Void {
                    return Ok(None);
                }
                let base_ptr8 = ctx.linear_i8_ptr_from_i32(base_v.repr.as_str());
                let field_ptr8 = ctx.next_tmp();
                let typed_ptr = ctx.next_tmp();
                let out = ctx.next_tmp();
                ctx.push_line(&format!(
                    "  {} = getelementptr i8, i8* {}, i64 {}",
                    field_ptr8, base_ptr8, offset
                ));
                ctx.push_line(&format!(
                    "  {} = bitcast i8* {} to {}*",
                    typed_ptr,
                    field_ptr8,
                    out_ty.ir()
                ));
                ctx.push_line(&format!(
                    "  {} = load {}, {}* {}, align 1",
                    out,
                    out_ty.ir(),
                    out_ty.ir(),
                    typed_ptr
                ));
                return Ok(Some(LlValue {
                    ty: out_ty,
                    repr: out,
                }));
            }
            if name == "get_field_ref" {
                if args.len() != 2 {
                    llvm_codegen_bail!(
                        "internal compiler error: intrinsic get_field_ref requires two args in '{}'",
                        ctx.function_name
                    );
                }
                let base_ty = match types.get(types.resolve_id(args[0].ty)) {
                    TypeKind::Reference(inner, _) => inner,
                    _ => args[0].ty,
                };
                let Some((_field_ty, offset)) =
                    aggregate_field_layout(types, ctx, base_ty, &args[1])
                else {
                    llvm_codegen_bail!(
                        "internal compiler error: unsupported get_field_ref selector reached llvm lowering in '{}'",
                        ctx.function_name
                    );
                };
                let Some(base_v) = lower_hir_expr(types, ctx, &args[0])? else {
                    llvm_codegen_bail!(
                        "internal compiler error: intrinsic get_field_ref base must produce a value in '{}'",
                        ctx.function_name
                    );
                };
                if base_v.ty != LlTy::I32 {
                    llvm_codegen_bail!(
                        "internal compiler error: intrinsic get_field_ref base must be i32 in '{}' (got {:?})",
                        ctx.function_name, base_v.ty
                    );
                }
                if offset == 0 {
                    return Ok(Some(base_v));
                }
                let out = ctx.next_tmp();
                ctx.push_line(&format!("  {} = add i32 {}, {}", out, base_v.repr, offset));
                return Ok(Some(LlValue {
                    ty: LlTy::I32,
                    repr: out,
                }));
            }
            if name == "set_field" {
                if args.len() != 3 {
                    llvm_codegen_bail!(
                        "internal compiler error: intrinsic set_field requires three args in '{}'",
                        ctx.function_name
                    );
                }
                let Some((_field_ty, offset)) =
                    aggregate_field_layout(types, ctx, args[0].ty, &args[1])
                else {
                    llvm_codegen_bail!(
                        "internal compiler error: unsupported set_field selector reached llvm lowering in '{}'",
                        ctx.function_name
                    );
                };
                let field_ty = args[2].ty;
                let Some(base_v) = lower_hir_expr(types, ctx, &args[0])? else {
                    llvm_codegen_bail!(
                        "internal compiler error: intrinsic set_field base must produce a value in '{}'",
                        ctx.function_name
                    );
                };
                if base_v.ty != LlTy::I32 {
                    llvm_codegen_bail!(
                        "internal compiler error: intrinsic set_field base must be i32 in '{}' (got {:?})",
                        ctx.function_name, base_v.ty
                    );
                }
                if is_aggregate_storage_type(types, field_ty) {
                    let Some(src_v) = lower_hir_expr(types, ctx, &args[2])? else {
                        llvm_codegen_bail!(
                            "internal compiler error: aggregate set_field value must produce a value in '{}'",
                            ctx.function_name
                        );
                    };
                    if src_v.ty != LlTy::I32 {
                        llvm_codegen_bail!(
                            "internal compiler error: aggregate set_field value must lower to i32 handle in '{}'",
                            ctx.function_name
                        );
                    }
                    let dst_base8 = ctx.linear_i8_ptr_from_i32(base_v.repr.as_str());
                    let dst_ptr8 = ctx.next_tmp();
                    ctx.push_line(&format!(
                        "  {} = getelementptr i8, i8* {}, i64 {}",
                        dst_ptr8, dst_base8, offset
                    ));
                    let src_ptr8 = ctx.linear_i8_ptr_from_i32(src_v.repr.as_str());
                    let size = storage_size_bytes(types, field_ty) as i64;
                    for off in 0..size {
                        let dst_byte_ptr = ctx.next_tmp();
                        let src_byte_ptr = ctx.next_tmp();
                        let byte_val = ctx.next_tmp();
                        ctx.push_line(&format!(
                            "  {} = getelementptr i8, i8* {}, i64 {}",
                            dst_byte_ptr, dst_ptr8, off
                        ));
                        ctx.push_line(&format!(
                            "  {} = getelementptr i8, i8* {}, i64 {}",
                            src_byte_ptr, src_ptr8, off
                        ));
                        ctx.push_line(&format!(
                            "  {} = load i8, i8* {}, align 1",
                            byte_val, src_byte_ptr
                        ));
                        ctx.push_line(&format!(
                            "  store i8 {}, i8* {}, align 1",
                            byte_val, dst_byte_ptr
                        ));
                    }
                    return Ok(None);
                }
                let val_ty = llty_for_type(types, field_ty);
                if val_ty == LlTy::Void {
                    let _ = lower_hir_expr(types, ctx, &args[2])?;
                    return Ok(None);
                }
                let Some(val_v) = lower_hir_expr(types, ctx, &args[2])? else {
                    llvm_codegen_bail!(
                        "internal compiler error: set_field value must produce a value in '{}'",
                        ctx.function_name
                    );
                };
                if val_v.ty != val_ty {
                    llvm_codegen_bail!(
                        "internal compiler error: set_field value type mismatch in '{}' ({:?} vs {:?})",
                        ctx.function_name, val_ty, val_v.ty
                    );
                }
                let base_ptr8 = ctx.linear_i8_ptr_from_i32(base_v.repr.as_str());
                let field_ptr8 = ctx.next_tmp();
                let typed_ptr = ctx.next_tmp();
                ctx.push_line(&format!(
                    "  {} = getelementptr i8, i8* {}, i64 {}",
                    field_ptr8, base_ptr8, offset
                ));
                ctx.push_line(&format!(
                    "  {} = bitcast i8* {} to {}*",
                    typed_ptr,
                    field_ptr8,
                    val_ty.ir()
                ));
                ctx.push_line(&format!(
                    "  store {} {}, {}* {}, align 1",
                    val_ty.ir(),
                    val_v.repr,
                    val_ty.ir(),
                    typed_ptr
                ));
                return Ok(None);
            }
            if let Some(kind) = I32ArithmeticPrimitive::from_codegen_intrinsic_name(name) {
                return lower_i32_arithmetic_intrinsic(types, ctx, kind, args);
            }
            if let Some(kind) = ScalarIntrinsicKind::from_intrinsic_name(name) {
                return scalar_intrinsic::lower_scalar_intrinsic(types, ctx, kind, args);
            }
            if CollectionSlotBorrowPrimitive::from_intrinsic_name(name).is_some() {
                if args.len() != 2 {
                    llvm_codegen_bail!(
                        "internal compiler error: collection slot borrow intrinsic '{}' expects two args in '{}'",
                        name,
                        ctx.function_name
                    );
                }
                let Some(token_v) = lower_hir_expr(types, ctx, &args[0])? else {
                    llvm_codegen_bail!(
                        "internal compiler error: collection slot borrow target must produce an address in '{}'",
                        ctx.function_name
                    );
                };
                if token_v.ty != LlTy::I32 {
                    llvm_codegen_bail!(
                        "internal compiler error: collection slot borrow target must lower to i32 in '{}' (got {:?})",
                        ctx.function_name,
                        token_v.ty
                    );
                }
                let Some(offset_v) = lower_hir_expr(types, ctx, &args[1])? else {
                    llvm_codegen_bail!(
                        "internal compiler error: collection slot borrow offset must produce i32 in '{}'",
                        ctx.function_name
                    );
                };
                if offset_v.ty != LlTy::I32 {
                    llvm_codegen_bail!(
                        "internal compiler error: collection slot borrow offset must lower to i32 in '{}' (got {:?})",
                        ctx.function_name,
                        offset_v.ty
                    );
                }
                let token_ptr8 = ctx.linear_i8_ptr_from_i32(token_v.repr.as_str());
                let raw_ptr = ctx.next_tmp();
                let raw = ctx.next_tmp();
                let addr = ctx.next_tmp();
                ctx.push_line(&format!(
                    "  {} = bitcast i8* {} to i32*",
                    raw_ptr, token_ptr8
                ));
                ctx.push_line(&format!("  {} = load i32, i32* {}, align 1", raw, raw_ptr));
                ctx.push_line(&format!("  {} = add i32 {}, {}", addr, raw, offset_v.repr));
                return Ok(Some(LlValue {
                    ty: LlTy::I32,
                    repr: addr,
                }));
            }
            if CollectionSlotLifecyclePrimitive::from_intrinsic_name(name).is_some() {
                for arg in args {
                    let _ = lower_hir_expr(types, ctx, arg)?;
                }
                return Ok(None);
            }
            Err(llvm_codegen_error(
                format!(
                    "unsupported intrinsic '{}' reached llvm lowering in '{}'",
                    name, ctx.function_name
                ),
                expr.span,
                DiagnosticCode::Backend(crate::diagnostic_codes::BackendDiagnosticCode::Llvm(
                    crate::diagnostic_codes::LlvmDiagnosticCode::IntrinsicUnknown,
                )),
            ))
        }
        HirExprKind::AddrOf(inner) => {
            let inner_ty = types.resolve_id(inner.ty);
            if is_aggregate_storage_type(types, inner_ty) {
                let Some(v) = lower_hir_expr(types, ctx, inner)? else {
                    llvm_codegen_bail!(
                        "internal compiler error: aggregate address-of must produce a value in '{}'",
                        ctx.function_name
                    );
                };
                if v.ty != LlTy::I32 {
                    llvm_codegen_bail!(
                        "internal compiler error: aggregate address-of must lower to i32 handle in '{}' (got {:?})",
                        ctx.function_name, v.ty
                    );
                }
                return Ok(Some(v));
            }

            let Some(v) = lower_hir_expr(types, ctx, inner)? else {
                return Ok(Some(LlValue {
                    ty: LlTy::I32,
                    repr: String::from("0"),
                }));
            };
            let Some(alloc_name) = resolve_alloc_symbol(ctx) else {
                return Err(llvm_codegen_error(
                    format!(
                        "alloc function is required for address-of in '{}'",
                        ctx.function_name
                    ),
                    Span::dummy(),
                    DiagnosticCode::Backend(crate::diagnostic_codes::BackendDiagnosticCode::Llvm(
                        crate::diagnostic_codes::LlvmDiagnosticCode::FunctionUnknown,
                    )),
                ));
            };
            let ptr = ctx.next_tmp();
            let size = storage_size_bytes(types, inner_ty) as i64;
            ctx.push_line(&format!(
                "  {} = call i32 {}(i32 {})",
                ptr,
                ll_symbol(alloc_name.as_str()),
                size
            ));
            let ty_kind = types.get(inner_ty);
            if matches!(ty_kind, TypeKind::U8) {
                if v.ty != LlTy::I32 {
                    llvm_codegen_bail!(
                        "internal compiler error: address-of u8 expects i32 value in '{}' (got {:?})",
                        ctx.function_name, v.ty
                    );
                }
                let p_ptr = ctx.linear_i8_ptr_from_i32(ptr.as_str());
                let b = ctx.next_tmp();
                ctx.push_line(&format!("  {} = trunc i32 {} to i8", b, v.repr));
                ctx.push_line(&format!("  store i8 {}, i8* {}, align 1", b, p_ptr));
            } else {
                let store_ty = llty_for_type(types, inner_ty);
                if v.ty != store_ty {
                    llvm_codegen_bail!(
                        "internal compiler error: address-of type mismatch in '{}' ({:?} vs {:?})",
                        ctx.function_name,
                        store_ty,
                        v.ty
                    );
                }
                let p_ptr = ctx.linear_typed_ptr_from_i32(ptr.as_str(), store_ty);
                ctx.push_line(&format!(
                    "  store {} {}, {}* {}, align 1",
                    store_ty.ir(),
                    v.repr,
                    store_ty.ir(),
                    p_ptr
                ));
            }
            Ok(Some(LlValue {
                ty: LlTy::I32,
                repr: ptr,
            }))
        }
        HirExprKind::Deref(inner) => {
            let Some(ptr_v) = lower_hir_expr(types, ctx, inner)? else {
                llvm_codegen_bail!(
                    "internal compiler error: deref pointer must produce a value in '{}'",
                    ctx.function_name
                );
            };
            if ptr_v.ty != LlTy::I32 {
                llvm_codegen_bail!(
                    "internal compiler error: deref pointer must be i32 in '{}' (got {:?})",
                    ctx.function_name,
                    ptr_v.ty
                );
            }
            let ty = types.resolve_id(expr.ty);
            if is_aggregate_storage_type(types, ty) {
                let size = storage_size_bytes(types, ty) as i64;
                let Some(alloc_name) = resolve_alloc_symbol(ctx) else {
                    return Err(llvm_codegen_error(
                        format!(
                            "alloc function is required for aggregate deref in '{}'",
                            ctx.function_name
                        ),
                        Span::dummy(),
                        DiagnosticCode::Backend(
                            crate::diagnostic_codes::BackendDiagnosticCode::Llvm(
                                crate::diagnostic_codes::LlvmDiagnosticCode::FunctionUnknown,
                            ),
                        ),
                    ));
                };
                let dst = ctx.next_tmp();
                ctx.push_line(&format!(
                    "  {} = call i32 {}(i32 {})",
                    dst,
                    ll_symbol(alloc_name.as_str()),
                    size
                ));
                let dst_ptr = ctx.linear_i8_ptr_from_i32(dst.as_str());
                let src_ptr = ctx.linear_i8_ptr_from_i32(ptr_v.repr.as_str());
                emit_copy_linear_bytes_llvm(ctx, dst_ptr.as_str(), 0, src_ptr.as_str(), 0, size);
                return Ok(Some(LlValue {
                    ty: LlTy::I32,
                    repr: dst,
                }));
            }

            let ty_kind = types.get(ty);
            if matches!(ty_kind, TypeKind::U8) {
                let p_ptr = ctx.linear_i8_ptr_from_i32(ptr_v.repr.as_str());
                let raw = ctx.next_tmp();
                let out = ctx.next_tmp();
                ctx.push_line(&format!("  {} = load i8, i8* {}, align 1", raw, p_ptr));
                ctx.push_line(&format!("  {} = zext i8 {} to i32", out, raw));
                return Ok(Some(LlValue {
                    ty: LlTy::I32,
                    repr: out,
                }));
            }
            let out_ty = llty_for_type(types, ty);
            if out_ty == LlTy::Void {
                return Ok(None);
            }
            let p_ptr = ctx.linear_typed_ptr_from_i32(ptr_v.repr.as_str(), out_ty);
            let out = ctx.next_tmp();
            ctx.push_line(&format!(
                "  {} = load {}, {}* {}, align 1",
                out,
                out_ty.ir(),
                out_ty.ir(),
                p_ptr
            ));
            Ok(Some(LlValue {
                ty: out_ty,
                repr: out,
            }))
        }
        HirExprKind::Drop { .. } => Ok(None),
    }
}

fn lower_hir_string_literal(
    _types: &TypeCtx,
    ctx: &mut LowerCtx<'_>,
    id: usize,
) -> Result<Option<LlValue>, LlvmCodegenError> {
    let Some(s) = ctx.strings.get(id) else {
        llvm_codegen_bail!(
            "internal compiler error: string literal id {} was out of bounds in '{}'",
            id,
            ctx.function_name
        );
    };
    let bytes = s.as_bytes();
    let Some(alloc_name) = resolve_alloc_symbol(ctx) else {
        return Err(llvm_codegen_error(
            format!(
                "alloc function is required to materialize string literals in '{}'",
                ctx.function_name
            ),
            Span::dummy(),
            DiagnosticCode::Backend(crate::diagnostic_codes::BackendDiagnosticCode::Llvm(
                crate::diagnostic_codes::LlvmDiagnosticCode::FunctionUnknown,
            )),
        ));
    };
    let ptr_tmp = ctx.next_tmp();
    let total_len = (bytes.len() + 4) as i32;
    ctx.push_line(&format!(
        "  {} = call i32 {}(i32 {})",
        ptr_tmp,
        ll_symbol(alloc_name.as_str()),
        total_len
    ));
    let len_ptr = ctx.linear_typed_ptr_from_i32(ptr_tmp.as_str(), LlTy::I32);
    ctx.push_line(&format!(
        "  store i32 {}, i32* {}, align 1",
        bytes.len(),
        len_ptr
    ));
    for (idx, b) in bytes.iter().enumerate() {
        let off = ctx.next_tmp();
        ctx.push_line(&format!("  {} = add i32 {}, {}", off, ptr_tmp, idx + 4));
        let ptr8 = ctx.linear_i8_ptr_from_i32(off.as_str());
        ctx.push_line(&format!("  store i8 {}, i8* {}, align 1", *b as i32, ptr8));
    }
    Ok(Some(LlValue {
        ty: LlTy::I32,
        repr: ptr_tmp,
    }))
}

fn ll_symbol(name: &str) -> String {
    let escaped = name.replace('\\', "\\5C").replace('"', "\\22");
    format!("@\"{}\"", escaped)
}

fn llvm_f32_literal(v: f32) -> String {
    if v.is_nan() {
        return String::from("0x7FC00000");
    }
    if v == f32::INFINITY {
        return String::from("0x7F800000");
    }
    if v == f32::NEG_INFINITY {
        return String::from("0xFF800000");
    }
    format!("{:.9e}", v)
}

fn resolve_alloc_symbol(ctx: &LowerCtx<'_>) -> Option<String> {
    resolve_runtime_helper_symbol(
        ctx.sigs,
        helper_candidates(RuntimeHelperKind::Alloc),
        &[LlTy::I32],
        LlTy::I32,
    )
    .map(String::from)
    .or_else(|| ctx.fallback_alloc_symbol.map(String::from))
}

fn resolve_runtime_helper_symbol<'a>(
    sigs: &'a BTreeMap<String, FnSig>,
    preferred_names: &[&str],
    params: &[LlTy],
    ret: LlTy,
) -> Option<&'a str> {
    for preferred in preferred_names {
        if let Some(name) = resolve_symbol_name(sigs, preferred, params, ret) {
            return Some(name);
        }
    }
    None
}

fn resolve_symbol_name<'a>(
    sigs: &'a BTreeMap<String, FnSig>,
    preferred: &str,
    params: &[LlTy],
    ret: LlTy,
) -> Option<&'a str> {
    let signature_matches = |sig: &FnSig| sig.ret == ret && sig.params.as_slice() == params;

    if let Some((name, sig)) = sigs.get_key_value(preferred) {
        if signature_matches(sig) {
            return Some(name.as_str());
        }
    }

    let mut candidates = sigs
        .iter()
        .filter_map(|(name, sig)| {
            if !signature_matches(sig) {
                return None;
            }
            if helper_base_name(name.as_str()) == preferred {
                Some(name.as_str())
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    if candidates.is_empty() {
        return None;
    }
    candidates.sort_unstable();
    candidates.first().copied()
}

fn enum_variant_tag(ctx: &TypeCtx, enum_ty: TypeId, variant: &str) -> i32 {
    let name = crate::qualified_name::member_tail(variant);
    let enum_ty = ctx.resolve_named_type_id(enum_ty);
    match ctx.get(enum_ty) {
        TypeKind::Enum { variants, .. } => {
            variants.iter().position(|v| v.name == name).unwrap_or(0) as i32
        }
        TypeKind::Apply { base, .. } => enum_variant_tag(ctx, base, name),
        _ => 0,
    }
}

fn enum_match_type(ctx: &TypeCtx, ty: TypeId) -> Option<TypeId> {
    let ty = ctx.resolve_named_type_id(ctx.resolve_id(ty));
    match ctx.get(ty) {
        TypeKind::Enum { .. } => Some(ty),
        TypeKind::Apply { base, .. } => {
            if matches!(
                ctx.get(ctx.resolve_named_type_id(base)),
                TypeKind::Enum { .. }
            ) {
                Some(ty)
            } else {
                None
            }
        }
        TypeKind::Reference(target, _) => enum_match_type(ctx, target),
        _ => None,
    }
}

fn enum_variant_payload(ctx: &TypeCtx, enum_ty: TypeId, variant: &str) -> Option<TypeId> {
    let name = crate::qualified_name::member_tail(variant);
    let enum_ty = ctx.resolve_named_type_id(enum_ty);
    match ctx.get(enum_ty) {
        TypeKind::Enum { variants, .. } => variants
            .iter()
            .find(|v| v.name == name)
            .and_then(|v| v.payload),
        TypeKind::Apply { base, args } => match ctx.get(ctx.resolve_named_type_id(base)) {
            TypeKind::Enum {
                variants,
                type_params,
                ..
            } => {
                let payload = variants
                    .iter()
                    .find(|v| v.name == name)
                    .and_then(|v| v.payload);
                payload.map(|pty| {
                    if let Some(pos) = type_params.iter().position(|tp| *tp == pty) {
                        if let Some(arg) = args.get(pos) {
                            return *arg;
                        }
                    }
                    pty
                })
            }
            _ => None,
        },
        _ => None,
    }
}

fn summarize_diagnostics_for_message(diags: &[crate::diagnostic::Diagnostic]) -> String {
    let errs = diags
        .iter()
        .filter(|d| matches!(d.severity, crate::diagnostic::Severity::Error))
        .collect::<Vec<_>>();
    if errs.is_empty() {
        return String::from("no diagnostic details");
    }
    let mut uniq = BTreeSet::new();
    for d in errs.iter().take(8) {
        let code_prefix = format!("[{}] ", d.code.as_str());
        uniq.insert(format!(
            "{}{} (file={}, start={}, end={})",
            code_prefix,
            d.message,
            d.primary.span.file_id.0,
            d.primary.span.start,
            d.primary.span.end
        ));
    }
    let total = errs.len();
    let mut parts = uniq.into_iter().collect::<Vec<_>>();
    if total > parts.len() {
        parts.push(format!("... and {} more diagnostics", total - parts.len()));
    }
    parts.join(" / ")
}

fn llvm_output_has_function(out: &str, name: &str) -> bool {
    out.lines()
        .filter_map(parse_declared_or_defined_function_name)
        .any(|n| n == name)
}

fn collect_active_entry_names(
    module: &Module,
    target: CompileTarget,
    profile: BuildProfile,
) -> Vec<String> {
    let mut out = Vec::new();
    for idx in target_precheck::active_stmt_indices(&module.root, target, profile) {
        let stmt = &module.root.items[idx];
        if let Stmt::Directive(Directive::Entry { name }) = stmt {
            out.push(name.name.clone());
        }
    }
    out
}

pub fn minify_ll_text(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for line in input.lines() {
        let mut in_quote = false;
        let mut comment_start = None;
        for (i, c) in line.char_indices() {
            if c == '"' {
                in_quote = !in_quote;
            } else if c == ';' && !in_quote {
                comment_start = Some(i);
                break;
            }
        }
        let content = if let Some(idx) = comment_start {
            &line[..idx]
        } else {
            line
        };
        let trimmed = content.trim();
        if trimmed.is_empty() {
            continue;
        }
        out.push_str(trimmed);
        out.push('\n');
    }
    out
}

fn append_llvmir_block(out: &mut String, block: &crate::ast::LlvmIrBlock) {
    for line in &block.lines {
        out.push_str(line);
        out.push('\n');
    }
    out.push('\n');
}

fn lower_parsed_fn_with_gates(
    name: &str,
    signature: &TypeExpr,
    params: &[Ident],
    body: &Block,
    target: CompileTarget,
    profile: BuildProfile,
) -> Option<String> {
    if !params.is_empty() {
        return None;
    }

    let result_ty = match signature.as_unspanned() {
        TypeExpr::Function { result, .. } => result.as_ref(),
        _ => return None,
    };
    if !matches!(result_ty.as_unspanned(), TypeExpr::I32) {
        return None;
    }

    let active = target_precheck::active_stmt_indices(body, target, profile);
    if active.len() != 1 {
        return None;
    }
    let ret_value = match &body.items[active[0]] {
        Stmt::Expr(expr) => lower_i32_literal_expr(expr)?,
        _ => return None,
    };

    Some(format!(
        "define i32 @{}() {{\nentry:\n  ret i32 {}\n}}",
        name, ret_value
    ))
}

fn lower_i32_literal_expr(expr: &PrefixExpr) -> Option<i32> {
    if expr.items.len() != 1 {
        return None;
    }
    match &expr.items[0] {
        PrefixItem::Literal(Literal::Int(text), _) => parse_i32_literal(text),
        _ => None,
    }
}

fn parse_i32_literal(text: &str) -> Option<i32> {
    if let Some(hex) = text.strip_prefix("0x") {
        i32::from_str_radix(hex, 16).ok()
    } else if let Some(hex) = text.strip_prefix("-0x") {
        i32::from_str_radix(hex, 16).ok().map(|v| -v)
    } else {
        text.parse::<i32>().ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagnostic::Severity;
    use crate::lexer;
    use crate::parser;
    use crate::span::FileId;

    fn parse_module(src: &str) -> Module {
        let file_id = FileId(0);
        let lexed = lexer::lex(file_id, src);
        let parsed = parser::parse_tokens(file_id, lexed);
        let has_error = parsed
            .diagnostics
            .iter()
            .any(|d| matches!(d.severity, Severity::Error));
        assert!(!has_error, "parse diagnostics: {:?}", parsed.diagnostics);
        parsed.module.expect("module should parse")
    }

    #[test]
    fn emit_ll_collects_top_and_fn_blocks() {
        let src = r#"
#indent 4
#target llvm

#llvmir:
    ; module header
    target triple = "x86_64-pc-linux-gnu"

fn body %fn void i32 \void:
    #llvmir:
        define i32 @body() {
        entry:
            ret i32 7
        }
"#;
        let module = parse_module(src);
        let ll = emit_ll_from_module(&module).expect("llvm ir should be emitted");
        assert!(ll.contains("; module header"));
        assert!(ll.contains("define i32 @body()"));
        assert!(ll.contains("    ret i32 7"));
    }

    #[test]
    fn emit_ll_skips_unsupported_parsed_function_body() {
        // Functions with parameters are not lowered by lower_parsed_fn_with_gates
        // (only zero-parameter i32-returning functions with a literal body are supported).
        // The function typechecks fine but must not appear in the LLVM output.
        let src = r#"
#target llvm
fn body %fn i32 i32 \x:
    x
"#;
        let module = parse_module(src);
        let ll =
            emit_ll_from_module(&module).expect("unsupported parsed function should be skipped");
        assert!(!ll.contains("define i32 @body("));
    }

    #[test]
    fn emit_ll_supports_parsed_const_i32_function() {
        let src = r#"
#target llvm
fn c %fn void i32 \void:
    123
"#;
        let module = parse_module(src);
        let ll = emit_ll_from_module(&module).expect("const i32 function should be lowered");
        assert!(ll.contains("define i32 @c()"));
        assert!(ll.contains("ret i32 123"));
    }

    #[test]
    fn emit_ll_respects_if_target_gate() {
        let src = r#"
#target llvm
#if[target=wasm]
fn w %fn void i32 \void:
    #wasm:
        i32.const 1

#if[target=llvm]
fn l %fn void i32 \void:
    #llvmir:
        define i32 @l() {
        entry:
            ret i32 9
        }
"#;
        let module = parse_module(src);
        let ll = emit_ll_from_module_for_target(
            &module,
            CompileTarget::Llvm,
            BuildProfile::Debug,
            false,
        )
        .expect("llvm-gated items should compile");
        assert!(ll.contains("define i32 @l()"));
        assert!(!ll.contains("define i32 @w()"));
    }

    #[test]
    fn emit_ll_supports_function_body_if_target_raw() {
        let src = r#"
#target llvm
fn f %fn void i32 \void:
    #if[target=wasm]
    #wasm:
        i32.const 1
    #if[target=llvm]
    #llvmir:
        define i32 @f() {
        entry:
            ret i32 42
        }
"#;
        let module = parse_module(src);
        let ll = emit_ll_from_module_for_target(
            &module,
            CompileTarget::Llvm,
            BuildProfile::Debug,
            false,
        )
        .expect("llvm raw function body should be selected");
        assert!(ll.contains("define i32 @f()"));
        assert!(ll.contains("ret i32 42"));
    }

    #[test]
    fn emit_ll_rejects_entry_with_wasm_body() {
        let src = r#"
#target llvm
#entry main
fn main %fn void i32 \void:
    #wasm:
        i32.const 1
"#;
        let module = parse_module(src);
        let err = emit_ll_from_module(&module).expect_err("entry with #wasm body must fail");
        assert!(matches!(err, LlvmCodegenError::TypecheckFailed { .. }));
    }

    #[test]
    fn emit_ll_generates_main_bridge_from_entry() {
        let src = r#"
#target llvm
#entry boot
fn boot %fn void i32 \void:
    #llvmir:
        define i32 @boot() {
        entry:
            ret i32 9
        }
"#;
        let module = parse_module(src);
        let ll = emit_ll_from_module(&module).expect("entry bridge should be emitted");
        assert!(ll.contains("define i32 @boot()"));
        assert!(ll.contains("define i32 @main()"));
        assert!(ll.contains("call i32 @boot()"));
    }

    #[test]
    fn emit_ll_accepts_top_level_raw_main_entry() {
        let src = r#"
#target llvm
#entry main
#llvmir:
    define i32 @main() {
    entry:
        ret i32 7
    }
"#;
        let module = parse_module(src);
        let ll = emit_ll_from_module(&module).expect("raw top-level main should satisfy #entry");
        assert!(ll.contains("define i32 @main()"));
        assert!(ll.contains("ret i32 7"));
    }

    #[test]
    fn emit_ll_bridges_top_level_raw_non_main_entry() {
        let src = r#"
#target llvm
#entry boot
#llvmir:
    define i32 @boot() {
    entry:
        ret i32 11
    }
"#;
        let module = parse_module(src);
        let ll = emit_ll_from_module(&module).expect("raw top-level entry should be bridged");
        assert!(ll.contains("define i32 @boot()"));
        assert!(ll.contains("define i32 @main()"));
        assert!(ll.contains("call i32 @boot()"));
    }
}
