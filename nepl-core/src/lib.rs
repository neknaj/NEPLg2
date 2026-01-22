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

pub mod diagnostic;
pub mod error;
pub mod span;

pub mod ast;
pub mod builtins;
pub mod codegen_wasm;
pub mod compiler;
pub mod hir;
pub mod lexer;
pub mod parser;
pub mod typecheck;
pub mod types;

pub use compiler::{compile_wasm, CompilationArtifact, CompileOptions, CompileTarget};
pub use error::CoreError;
