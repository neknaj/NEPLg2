#![no_std]
extern crate std;

use alloc::collections::BTreeMap;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

use crate::ast;
use crate::codegen_wasm;
use crate::diagnostic::Diagnostic;
use crate::error::CoreError;
use crate::lexer;
use crate::monomorphize;
use crate::parser;
use crate::passes;
use crate::span::FileId;
use crate::span::Span;
use crate::typecheck;
use wasmparser::{Imports, Parser, Payload, TypeRef, Validator};

/// コンパイル対象プラットフォーム。
///
/// - `Wasm`: 素の wasm 実行環境を想定
/// - `Wasi`: WASI 実行環境を想定（`wasm` の上位互換として扱う）
/// - `Llvm`: LLVM IR 出力向けのネイティブ経路（`nepl-cli` 側で処理）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompileTarget {
    Wasm,
    Wasi,
    Llvm,
}

impl CompileTarget {
    pub fn allows(&self, gate: &str) -> bool {
        match gate {
            "wasm" => true, // wasi includes wasm
            "wasi" => matches!(self, CompileTarget::Wasi),
            "llvm" => matches!(self, CompileTarget::Llvm),
            _ => false,
        }
    }
}

/// ビルドプロファイル。
///
/// 条件付きコンパイル（`#if[profile=...]`）の判定に使用する。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuildProfile {
    Debug,
    Release,
}

impl BuildProfile {
    pub fn detect() -> Self {
        if cfg!(debug_assertions) {
            BuildProfile::Debug
        } else {
            BuildProfile::Release
        }
    }
}

/// コンパイル実行オプション。
#[derive(Debug, Clone, Copy)]
pub struct CompileOptions {
    /// Explicit target override (e.g., CLI flag). If None, #target or default is used.
    pub target: Option<CompileTarget>,
    /// Emit verbose compiler logs for debugging.
    pub verbose: bool,
    /// Explicit profile override for conditional compilation.
    pub profile: Option<BuildProfile>,
}

impl Default for CompileOptions {
    fn default() -> Self {
        Self {
            target: None,
            verbose: false,
            profile: None,
        }
    }
}

/// コンパイル成果物。
#[derive(Debug, Clone)]
pub struct CompilationArtifact {
    pub wasm: Vec<u8>,
    /// WAT 向けの補助情報（関数・ローカル変数・型）。
    /// 先頭コメントとして付与することを想定し、プレーンテキストで保持する。
    pub wat_comments: String,
}

/// 解析済みモジュールを最終成果物へ変換する。
///
/// この関数はコンパイルパイプラインの中核であり、以下の段階を順番に実行する。
/// 1. target/profile の確定
/// 2. typecheck
/// 3. monomorphize
/// 4. move check
/// 5. drop 挿入
/// 6. wasm 生成と妥当性検証
pub fn compile_module(
    module: ast::Module,
    options: CompileOptions,
) -> Result<CompilationArtifact, CoreError> {
    crate::log::set_verbose(options.verbose);
    let target = resolve_target(&module, options)?;
    if matches!(target, CompileTarget::Llvm) {
        let mut diags = Vec::new();
        diags.push(Diagnostic::error(
            "llvm target is CLI-only and is not handled by the wasm backend; use nepl-cli LLVM pipeline",
            Span::dummy(),
        ));
        return Err(CoreError::from_diagnostics(diags));
    }
    let profile = options.profile.unwrap_or(BuildProfile::detect());
    let tc = run_typecheck(&module, target, profile)?;
    let mut types = tc.types;
    let mut hir_module = monomorphize::monomorphize(&mut types, tc.module);

    let mut diagnostics = tc.diagnostics;
    run_move_check(&hir_module, &types, &mut diagnostics)?;
    passes::insert_drops(&mut hir_module, types.unit());

    emit_wasm(&types, &hir_module, diagnostics)
}

/// ソーステキストから wasm を生成する。
///
/// lexer/parser の診断がある場合は早期にエラーを返し、
/// その後の段階は `compile_module` に委譲する。
pub fn compile_wasm(
    file_id: FileId,
    source: &str,
    options: CompileOptions,
) -> Result<CompilationArtifact, CoreError> {
    let lex = lexer::lex(file_id, source);
    let parse = parser::parse_tokens(file_id, lex);
    let module = match parse.module {
        Some(m) => m,
        None => return Err(CoreError::from_diagnostics(parse.diagnostics)),
    };
    if parse
        .diagnostics
        .iter()
        .any(|d| matches!(d.severity, crate::diagnostic::Severity::Error))
    {
        return Err(CoreError::from_diagnostics(parse.diagnostics));
    }

    match compile_module(module, options) {
        Ok(artifact) => Ok(artifact),
        Err(CoreError::Diagnostics(mut ds)) => {
            let mut diags = parse.diagnostics;
            diags.append(&mut ds);
            Err(CoreError::from_diagnostics(diags))
        }
        Err(e) => Err(e),
    }
}

struct TypedProgram {
    types: crate::types::TypeCtx,
    module: crate::hir::HirModule,
    diagnostics: Vec<Diagnostic>,
}

fn run_typecheck(
    module: &ast::Module,
    target: CompileTarget,
    profile: BuildProfile,
) -> Result<TypedProgram, CoreError> {
    let tc = typecheck::typecheck(module, target, profile);
    match tc.module {
        Some(m) => Ok(TypedProgram {
            types: tc.types,
            module: m,
            diagnostics: tc.diagnostics,
        }),
        None => Err(CoreError::from_diagnostics(tc.diagnostics)),
    }
}

fn run_move_check(
    hir_module: &crate::hir::HirModule,
    types: &crate::types::TypeCtx,
    diagnostics: &mut Vec<Diagnostic>,
) -> Result<(), CoreError> {
    let move_errors = passes::move_check::run(hir_module, types);
    if move_errors.is_empty() {
        return Ok(());
    }
    diagnostics.extend(move_errors);
    Err(CoreError::from_diagnostics(diagnostics.clone()))
}

fn emit_wasm(
    types: &crate::types::TypeCtx,
    hir_module: &crate::hir::HirModule,
    mut diagnostics: Vec<Diagnostic>,
) -> Result<CompilationArtifact, CoreError> {
    let cg = codegen_wasm::generate_wasm(types, hir_module);
    diagnostics.extend(cg.diagnostics);
    let Some(bytes) = cg.bytes else {
        return Err(CoreError::from_diagnostics(diagnostics));
    };

    let mut validator = Validator::new();
    if let Err(err) = validator.validate_all(&bytes) {
        let err_msg = alloc::format!("invalid wasm generated: {}", err);
        diagnostics.push(Diagnostic::error(err_msg.clone(), Span::dummy()));
        if let Some(offset) = parse_wasm_error_offset(&err_msg) {
            if let Some(loc) = locate_wasm_function_at_offset(&bytes, offset) {
                let near_name = hir_module
                    .functions
                    .get(loc.defined_func_index as usize)
                    .map(|f| f.name.as_str())
                    .unwrap_or("<unknown>");
                diagnostics.push(Diagnostic::warning(
                    alloc::format!(
                        "validation failed near function body: func_index={}, defined_func_index={}, name={}, body_range=0x{:x}..0x{:x}",
                        loc.func_index, loc.defined_func_index, near_name, loc.body_start, loc.body_end
                    ),
                    Span::dummy(),
                ));
            }
        }
        return Err(CoreError::from_diagnostics(diagnostics));
    }
    Ok(CompilationArtifact {
        wasm: bytes,
        wat_comments: build_wat_comments(types, hir_module),
    })
}

#[derive(Debug, Clone, Copy)]
struct WasmFuncLocation {
    func_index: u32,
    defined_func_index: u32,
    body_start: usize,
    body_end: usize,
}

fn parse_wasm_error_offset(message: &str) -> Option<usize> {
    let marker = "offset 0x";
    let start = message.find(marker)? + marker.len();
    let hex = message[start..]
        .chars()
        .take_while(|c| c.is_ascii_hexdigit())
        .collect::<String>();
    if hex.is_empty() {
        return None;
    }
    usize::from_str_radix(&hex, 16).ok()
}

fn locate_wasm_function_at_offset(bytes: &[u8], offset: usize) -> Option<WasmFuncLocation> {
    let mut imported_func_count: u32 = 0;
    let mut defined_func_index: u32 = 0;
    for payload in Parser::new(0).parse_all(bytes) {
        let Ok(payload) = payload else {
            return None;
        };
        match payload {
            Payload::ImportSection(reader) => {
                for imp in reader {
                    let Ok(imp) = imp else {
                        return None;
                    };
                    match imp {
                        Imports::Single(_, import) => {
                            if matches!(import.ty, TypeRef::Func(_) | TypeRef::FuncExact(_)) {
                                imported_func_count += 1;
                            }
                        }
                        Imports::Compact1 { items, .. } => {
                            for item in items {
                                let Ok(item) = item else {
                                    return None;
                                };
                                if matches!(item.ty, TypeRef::Func(_) | TypeRef::FuncExact(_)) {
                                    imported_func_count += 1;
                                }
                            }
                        }
                        Imports::Compact2 { ty, names, .. } => {
                            if matches!(ty, TypeRef::Func(_) | TypeRef::FuncExact(_)) {
                                imported_func_count += names.count();
                            }
                        }
                    }
                }
            }
            Payload::CodeSectionEntry(body) => {
                let range = body.range();
                if offset >= range.start && offset < range.end {
                    return Some(WasmFuncLocation {
                        func_index: imported_func_count + defined_func_index,
                        defined_func_index,
                        body_start: range.start,
                        body_end: range.end,
                    });
                }
                defined_func_index += 1;
            }
            _ => {}
        }
    }
    None
}

/// WAT 先頭に付与するための補助情報を生成する。
///
/// 含める情報:
/// - 関数名
/// - 関数シグネチャ
/// - 引数名と型
/// - `let` で導入されたローカル変数名と型
fn build_wat_comments(types: &crate::types::TypeCtx, module: &crate::hir::HirModule) -> String {
    let mut out = String::new();
    out.push_str("NEPL WAT debug info\n");
    for func in &module.functions {
        out.push_str(&format!(
            "func {} : {}\n",
            func.name,
            types.type_to_string(func.func_ty)
        ));
        if !func.params.is_empty() {
            out.push_str("  params:\n");
            for p in &func.params {
                out.push_str(&format!("    {} : {}\n", p.name, types.type_to_string(p.ty)));
            }
        }
        let mut locals: BTreeMap<String, crate::types::TypeId> = BTreeMap::new();
        if let crate::hir::HirBody::Block(block) = &func.body {
            collect_block_locals(block, &mut locals);
        }
        if !locals.is_empty() {
            out.push_str("  locals:\n");
            for (name, ty) in locals {
                out.push_str(&format!("    {} : {}\n", name, types.type_to_string(ty)));
            }
        }
    }
    out
}

fn collect_block_locals(
    block: &crate::hir::HirBlock,
    locals: &mut BTreeMap<String, crate::types::TypeId>,
) {
    for line in &block.lines {
        collect_expr_locals(&line.expr, locals);
    }
}

fn collect_expr_locals(
    expr: &crate::hir::HirExpr,
    locals: &mut BTreeMap<String, crate::types::TypeId>,
) {
    match &expr.kind {
        crate::hir::HirExprKind::Let { name, value, .. } => {
            locals.entry(name.clone()).or_insert(value.ty);
            collect_expr_locals(value, locals);
        }
        crate::hir::HirExprKind::Set { value, .. } => {
            collect_expr_locals(value, locals);
        }
        crate::hir::HirExprKind::Call { args, .. } => {
            for arg in args {
                collect_expr_locals(arg, locals);
            }
        }
        crate::hir::HirExprKind::CallIndirect { callee, args, .. } => {
            collect_expr_locals(callee, locals);
            for arg in args {
                collect_expr_locals(arg, locals);
            }
        }
        crate::hir::HirExprKind::If {
            cond,
            then_branch,
            else_branch,
        } => {
            collect_expr_locals(cond, locals);
            collect_expr_locals(then_branch, locals);
            collect_expr_locals(else_branch, locals);
        }
        crate::hir::HirExprKind::While { cond, body } => {
            collect_expr_locals(cond, locals);
            collect_expr_locals(body, locals);
        }
        crate::hir::HirExprKind::Match { scrutinee, arms } => {
            collect_expr_locals(scrutinee, locals);
            for arm in arms {
                collect_expr_locals(&arm.body, locals);
            }
        }
        crate::hir::HirExprKind::StructConstruct { fields, .. } => {
            for f in fields {
                collect_expr_locals(f, locals);
            }
        }
        crate::hir::HirExprKind::TupleConstruct { items } => {
            for item in items {
                collect_expr_locals(item, locals);
            }
        }
        crate::hir::HirExprKind::EnumConstruct { payload, .. } => {
            if let Some(p) = payload {
                collect_expr_locals(p, locals);
            }
        }
        crate::hir::HirExprKind::Intrinsic { args, .. } => {
            for arg in args {
                collect_expr_locals(arg, locals);
            }
        }
        crate::hir::HirExprKind::AddrOf(inner)
        | crate::hir::HirExprKind::Deref(inner) => {
            collect_expr_locals(inner, locals);
        }
        crate::hir::HirExprKind::Block(block) => {
            collect_block_locals(block, locals);
        }
        crate::hir::HirExprKind::Var(_)
        | crate::hir::HirExprKind::FnValue(_)
        | crate::hir::HirExprKind::LiteralI32(_)
        | crate::hir::HirExprKind::LiteralF32(_)
        | crate::hir::HirExprKind::LiteralBool(_)
        | crate::hir::HirExprKind::LiteralStr(_)
        | crate::hir::HirExprKind::Unit
        | crate::hir::HirExprKind::Drop { .. } => {}
    }
}

fn resolve_target(
    module: &ast::Module,
    options: CompileOptions,
) -> Result<CompileTarget, CoreError> {
    if let Some(t) = options.target {
        return Ok(t);
    }
    let mut found: Option<(CompileTarget, Span)> = None;
    let mut diags = Vec::new();
    // First, check explicit module-level directives parsed into module.directives
    for d in &module.directives {
        if let ast::Directive::Target { target, span } = d {
            let parsed = match target.as_str() {
                "wasm" => Some(CompileTarget::Wasm),
                "wasi" => Some(CompileTarget::Wasi),
                "llvm" => Some(CompileTarget::Llvm),
                _ => None,
            };
            if let Some(t) = parsed {
                if let Some((_, prev_span)) = found {
                    diags.push(
                        Diagnostic::error("multiple #target directives are not allowed", *span)
                            .with_secondary_label(prev_span, Some("previous #target here".into())),
                    );
                } else {
                    found = Some((t, *span));
                }
            } else {
                diags.push(Diagnostic::error("unknown target in #target", *span));
            }
        }
    }

    // Fallback: some parsers/merging steps may leave a file-scoped #target as a top-level
    // statement rather than in module.directives; inspect root items as a safeguard.
    if found.is_none() {
        for it in &module.root.items {
            if let ast::Stmt::Directive(ast::Directive::Target { target, span }) = it {
                let parsed = match target.as_str() {
                    "wasm" => Some(CompileTarget::Wasm),
                    "wasi" => Some(CompileTarget::Wasi),
                    "llvm" => Some(CompileTarget::Llvm),
                    _ => None,
                };
                if let Some(t) = parsed {
                    if let Some((_, prev_span)) = found {
                        diags.push(Diagnostic::error(
                            "multiple #target directives are not allowed",
                            *span,
                        )
                        .with_secondary_label(prev_span, Some("previous #target here".into())));
                    } else {
                        found = Some((t, *span));
                    }
                } else {
                    diags.push(Diagnostic::error("unknown target in #target", *span));
                }
            }
        }
    }
    if !diags.is_empty() {
        return Err(CoreError::from_diagnostics(diags));
    }
    Ok(found.map(|(t, _)| t).unwrap_or(CompileTarget::Wasm))
}
