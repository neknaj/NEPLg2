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

pub mod span;
pub mod diagnostic;
pub mod error;

pub mod ast;
pub mod lexer;
pub mod parser;
pub mod types;
pub mod typecheck;
pub mod hir;
pub mod builtins;
pub mod codegen_wasm;
pub mod compiler;

pub use compiler::{compile_wasm, CompilationArtifact};
pub use error::CoreError;
