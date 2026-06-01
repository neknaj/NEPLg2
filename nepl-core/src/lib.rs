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
#[cfg(not(target_os = "none"))]
extern crate std;

pub mod diagnostic;
pub mod diagnostic_codes;
pub mod effects;
pub mod error;
pub mod function_identity;
pub mod span;

pub mod artifact;
pub mod ast;
mod backend_scalar_type;
pub mod builtins;
pub mod codegen_llvm;
pub mod codegen_wasm;
pub mod compiler;
pub mod hir;
mod intrinsic_kinds;
pub mod layout;
pub mod lexer;
mod llvm_ir;
#[cfg(not(target_os = "none"))]
pub mod loader;
pub mod log;
#[cfg(not(target_os = "none"))]
pub mod module_graph;
pub mod monomorphize;
pub mod nm;
pub mod parser;
pub mod passes;
mod qualified_name;
pub mod resolve;
pub mod resource;
mod resource_primitives;
pub mod runtime_helpers;
mod scalar_primitives;
pub mod source_cache_key;
mod source_capability;
pub mod source_map;
pub mod target_gate;
pub mod target_precheck;
pub mod typecheck;
pub mod types;
pub mod wasm_shared;

pub use artifact::{
    nepl_meta_artifact_header_for_public_surface, nepl_meta_compiler_identity_hash,
    nepl_meta_profile_hash, nepl_meta_target_hash, NeplMetaArtifact,
    NeplMetaArtifactCompatibilityReject, NeplMetaArtifactHeader, NeplMetaArtifactPayloadReject,
    NeplMetaExportEntry, NeplMetaExportKind, NeplMetaExportSurface, NeplMetaImportClause,
    NeplMetaImportItem, NeplMetaMaterializerMvpReject, NeplMetaModuleDependencyEdge,
    NeplMetaModuleDependencyKind, NeplMetaModuleSurface, NeplMetaReexportProjection,
    NeplMetaVisibility,
};
pub use compiler::{
    check_module, check_module_with_source_map, compile_module,
    compile_module_with_artifact_options, compile_module_with_source_map,
    compile_module_with_source_map_and_artifact_options,
    compile_module_with_source_map_artifact_options_and_dependency_public_surface_hash_resource_summary_value_cache_and_neplproof,
    compile_module_with_source_map_artifact_options_and_dependency_public_surface_hash_resource_summary_value_cache_neplproof_and_stage_timings,
    compile_module_with_source_map_artifact_options_dependency_public_surface_hash_module_surface_resource_summary_value_cache_and_neplproof,
    compile_module_with_source_map_artifact_options_dependency_public_surface_hash_module_surface_resource_summary_value_cache_neplproof_and_stage_timings,
    compile_wasm, resource_summary_private_effect_policy_hash,
    resource_summary_proof_compiler_identity_hash, resource_summary_proof_profile_hash,
    resource_summary_proof_target_hash, resource_summary_source_capability_policy_set_hash,
    BuildProfile, CompilationArtifact, CompilationArtifactOptions, CompileOptions,
    CompileStageTimings, CompileTarget, ResourceSummaryProofArtifactCacheOptions,
};
pub use error::CoreError;
