//! Resource IR used as the typed input for ownership and lifetime checks.
//!
//! This module is intentionally separate from the current HIR move checker.
//! Stage 2 exposes the data model, a non-enforcing lowering skeleton, and a
//! deterministic dump so later stages can migrate checks one operation class
//! at a time.

extern crate alloc;

mod borrow_check;
mod borrow_state;
mod borrow_summary;
mod cell_state;
mod condition_fact;
mod coverage;
mod coverage_hir;
mod coverage_resource;
mod dump;
mod effect;
mod effect_check;
mod effect_identity;
mod effect_summary;
mod function_alias;
mod initialized;
mod initialized_alias;
mod initialized_alias_flow;
mod initialized_control;
mod initialized_external_io;
mod initialized_external_io_effect;
mod initialized_raw_memory;
mod initialized_rekey;
mod initialized_summary;
mod initialized_summary_apply;
mod initialized_summary_build;
mod initialized_summary_variant_build;
mod initialized_variant;
mod lower;
mod lower_raw_address;
mod lower_raw_address_source;
mod lower_raw_memory;
mod model;
mod owner_alias;
mod owner_check;
mod owner_control;
mod owner_flow;
mod owner_raw_address;
mod owner_raw_view;
mod owner_return;
mod owner_state;
mod owner_summary;
mod owner_summary_leaf;
mod owner_summary_record;
mod owner_summary_variant_build;
mod owner_summary_variant_conditions;
mod owner_summary_variant_construct;
mod owner_summary_variant_match;
mod owner_summary_variant_paths;
mod owner_summary_variant_return;
mod owner_transfer;
mod owner_variant;
mod place_utils;
mod raw_realloc;
mod report;
mod shadow;
mod storage_origin;
mod summary;
mod type_pattern;

pub use borrow_check::check_resource_borrow_lifetimes;
pub use coverage::{
    compare_hir_resource_lowering, compare_hir_resource_lowering_typed, ResourceCoverageCounts,
    ResourceCoverageDiagnostic, ResourceCoverageKind, ResourceFunctionCoverage,
    ResourceLoweringCoverage,
};
pub use effect::{
    check_resource_effect_boundaries, ResourceEffectBoundaryDiagnostic,
    ResourceEffectBoundaryReport, ResourceEffectCallKind, ResourceEffectCounts,
    ResourceEffectFunctionCheck,
};
pub use initialized::check_resource_initialized_moves;
pub use lower::{lower_hir_module, lower_hir_module_skeleton};
pub use model::{
    AggregateKind, BorrowKind, BorrowState, BorrowStateEntry, CellState, CellStateEntry, EffectOp,
    OwnerState, OwnerStateEntry, Place, PlaceProjection, PlaceRoot, PointerProvenance, RawBodyKind,
    RawMemoryOp, ResourceBlock, ResourceBlockId, ResourceCallTarget, ResourceExprKind,
    ResourceFunction, ResourceId, ResourceLocal, ResourceMatchArm, ResourceMatchPattern,
    ResourceModule, ResourceOffset, ResourceOp, ResourceState, ResourceTerminator, StorageId,
    StorageOrigin, StorageOriginEntry,
};
pub use owner_check::check_resource_owner_obligations;
pub use report::{
    ResourceBorrowCheckDeferred, ResourceBorrowCheckReport, ResourceBorrowDiagnostic,
    ResourceBorrowFunctionCheck, ResourceBorrowOperation, ResourceCheckDeferred,
    ResourceCheckDiagnostic, ResourceCheckOperation, ResourceCheckReport, ResourceFunctionCheck,
    ResourceOwnerCheckDeferred, ResourceOwnerCheckReport, ResourceOwnerDiagnostic,
    ResourceOwnerFunctionCheck, ResourceOwnerOperation, ResourceSafetyShadowReport,
};
pub use shadow::check_hir_resource_safety_shadow;
