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

#[derive(Debug, Clone)]
pub struct CompilationArtifact {
    pub wasm: Vec<u8>,
}

pub fn compile_module(
    module: ast::Module,
    options: CompileOptions,
) -> Result<CompilationArtifact, CoreError> {
    crate::log::set_verbose(options.verbose);
    std::eprintln!("DEBUG: Starting compile_module");
    let target = resolve_target(&module, options)?;
    let profile = options.profile.unwrap_or(BuildProfile::detect());
    
    std::eprintln!("DEBUG: Starting typecheck");
    let tc = typecheck::typecheck(&module, target, profile);
    if tc.module.is_none() {
        return Err(CoreError::from_diagnostics(tc.diagnostics));
    }
    std::eprintln!("DEBUG: Typecheck done, starting monomorphize");
    let mut types = tc.types;
    let mut hir_module = monomorphize::monomorphize(&mut types, tc.module.unwrap());
    std::eprintln!("DEBUG: Monomorphize done, starting move_check");

    // Move Check
    let move_errors = passes::move_check::run(&hir_module, &types);
    if !move_errors.is_empty() {
        let mut diags = tc.diagnostics;
        diags.extend(move_errors);
        return Err(CoreError::from_diagnostics(diags));
    }
    std::eprintln!("DEBUG: Move check done, starting insert_drops");

    // Insert drop calls for automatic cleanup
    passes::insert_drops(&mut hir_module, types.unit());
    std::eprintln!("DEBUG: Insert drops done, starting codegen");

    let cg = codegen_wasm::generate_wasm(&types, &hir_module);
    std::eprintln!("DEBUG: Codegen done");
    let mut diagnostics = tc.diagnostics;
    diagnostics.extend(cg.diagnostics);
    if let Some(bytes) = cg.bytes {
        let mut validator = Validator::new();
        if let Err(err) = validator.validate_all(&bytes) {
            diagnostics.push(Diagnostic::error(
                alloc::format!("invalid wasm generated: {}", err),
                Span::dummy(),
            ));
            return Err(CoreError::from_diagnostics(diagnostics));
        }
        Ok(CompilationArtifact { wasm: bytes })
    } else {
        Err(CoreError::from_diagnostics(diagnostics))
    }
}

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
