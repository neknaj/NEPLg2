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
pub mod monomorphize;
pub mod module_graph;
pub mod parser;
pub mod passes;
pub mod resolve;
pub mod typecheck;
pub mod types;

pub use compiler::{
    compile_module, compile_wasm, BuildProfile, CompilationArtifact, CompileOptions, CompileTarget,
};
pub use error::CoreError;
