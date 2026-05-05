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
mod gate_demand;
mod initialized;
mod initialized_alias;
mod initialized_alias_flow;
mod initialized_alias_i32;
mod initialized_alias_order;
mod initialized_alias_type;
mod initialized_control;
mod initialized_external_io;
mod initialized_external_io_effect;
mod initialized_projection_domain;
mod initialized_raw_memory;
mod initialized_rekey;
mod initialized_summary;
mod initialized_summary_apply;
mod initialized_summary_build;
mod initialized_summary_cells;
mod initialized_summary_condition;
mod initialized_summary_destruction;
mod initialized_summary_destruction_address;
mod initialized_summary_variant_build;
mod initialized_summary_variant_condition;
mod initialized_summary_variant_requirement;
mod initialized_variant;
mod lower;
mod lower_condition;
mod lower_raw_address;
mod lower_raw_address_place;
mod lower_raw_address_source;
mod lower_raw_memory;
mod model;
mod owner_alias;
mod owner_check;
mod owner_control;
mod owner_flow;
mod owner_raw_address;
mod owner_raw_memory;
mod owner_raw_view;
mod owner_return;
mod owner_return_summary;
mod owner_state;
mod owner_summary;
mod owner_summary_cleanup;
mod owner_summary_leaf;
mod owner_summary_record;
mod owner_summary_variant_build;
mod owner_summary_variant_conditions;
mod owner_summary_variant_construct;
mod owner_summary_variant_entry;
mod owner_summary_variant_paths;
mod owner_summary_variant_return;
mod owner_summary_variant_unique;
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
pub(crate) use gate_demand::{resource_safety_gate_demand, ResourceSafetyGateDemand};
pub use initialized::check_resource_initialized_moves;
pub use lower::{lower_hir_module, lower_hir_module_skeleton};
pub use model::{
    AggregateKind, BorrowKind, BorrowState, BorrowStateEntry, CellState, CellStateEntry, EffectOp,
    OwnerState, OwnerStateEntry, Place, PlaceProjection, PlaceRoot, PointerProvenance, RawBodyKind,
    RawMemoryFillUnit, RawMemoryOp, ResourceBlock, ResourceBlockId, ResourceCallTarget,
    ResourceExprKind, ResourceFunction, ResourceId, ResourceLocal, ResourceMatchArm,
    ResourceMatchPattern, ResourceModule, ResourceOffset, ResourceOp, ResourceState,
    ResourceTerminator, StorageId, StorageOrigin, StorageOriginEntry,
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
