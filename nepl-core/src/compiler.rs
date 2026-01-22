#![no_std]
extern crate alloc;

use alloc::vec::Vec;

use crate::codegen_wasm;
use crate::error::CoreError;
use crate::lexer;
use crate::parser;
use crate::span::FileId;
use crate::typecheck;

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

#[derive(Debug, Clone, Copy)]
pub struct CompileOptions {
    pub target: CompileTarget,
}

impl Default for CompileOptions {
    fn default() -> Self {
        Self {
            target: CompileTarget::Wasm,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CompilationArtifact {
    pub wasm: Vec<u8>,
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

    let tc = typecheck::typecheck(&module, options.target);
    if tc.module.is_none() {
        let mut diags = parse.diagnostics;
        diags.extend(tc.diagnostics);
        return Err(CoreError::from_diagnostics(diags));
    }
    let hir_module = tc.module.unwrap();

    let cg = codegen_wasm::generate_wasm(&tc.types, &hir_module);
    let mut diagnostics = parse.diagnostics;
    diagnostics.extend(tc.diagnostics);
    diagnostics.extend(cg.diagnostics);
    if let Some(bytes) = cg.bytes {
        Ok(CompilationArtifact { wasm: bytes })
    } else {
        Err(CoreError::from_diagnostics(diagnostics))
    }
}
