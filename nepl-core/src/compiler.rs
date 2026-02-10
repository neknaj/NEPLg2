#![no_std]
extern crate std;

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
use wasmparser::Validator;

/// コンパイル対象プラットフォーム。
///
/// - `Wasm`: 素の wasm 実行環境を想定
/// - `Wasi`: WASI 実行環境を想定（`wasm` の上位互換として扱う）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompileTarget {
    Wasm,
    Wasi,
}

impl CompileTarget {
    pub fn allows(&self, gate: &str) -> bool {
        match gate {
            "wasm" => true, // wasi includes wasm
            "wasi" => matches!(self, CompileTarget::Wasi),
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
        diagnostics.push(Diagnostic::error(
            alloc::format!("invalid wasm generated: {}", err),
            Span::dummy(),
        ));
        return Err(CoreError::from_diagnostics(diagnostics));
    }
    Ok(CompilationArtifact { wasm: bytes })
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
