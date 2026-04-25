#![no_std]

//! Core utilities for the NEPLG2 language toolchain.
//!
//! Pipeline (wasm only):
//!   source
//!     -> lexer (indent aware)
//!     -> parser (prefix + block AST)
//!     -> typecheck (stack-based inference, hoisting)
//!     -> codegen_wasm

extern crate alloc;
#[cfg(not(target_arch = "wasm32"))]
extern crate std;

pub mod diagnostic;
pub mod diagnostic_ids;
pub mod effects;
pub mod error;
pub mod span;

pub mod ast;
pub mod builtins;
pub mod codegen_llvm;
pub mod codegen_wasm;
pub mod compiler;
pub mod hir;
pub mod lexer;
pub mod loader;
pub mod log;
pub mod module_graph;
pub mod monomorphize;
pub mod nm;
pub mod parser;
pub mod passes;
pub mod resolve;
pub mod runtime_helpers;
pub mod target_precheck;
pub mod typecheck;
pub mod types;
pub mod wasm_shared;

pub use compiler::{
    check_module, check_module_with_source_map, compile_module, compile_module_with_source_map,
    compile_wasm, BuildProfile, CompilationArtifact, CompileOptions, CompileTarget,
};
pub use error::CoreError;
