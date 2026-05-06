//! Resource IR used as the typed input for ownership and lifetime checks.
//!
//! The compiler pipeline treats this module as the authority for ownership,
//! lifetime, effect, and initialized-state checks after typed HIR is lowered.

extern crate alloc;

mod borrow_call;
mod borrow_check;
mod borrow_scope;
mod borrow_state;
mod borrow_summary;
mod borrow_usage;
mod cell_state;
mod condition_fact;
mod coverage;
mod coverage_hir;
mod coverage_hir_place;
mod coverage_hir_projection;
mod coverage_hir_projection_aggregate;
mod coverage_hir_raw;
mod coverage_resource;
mod drop_elaboration;
mod drop_elaboration_bindings;
mod drop_elaboration_hir_bridge;
mod drop_model;
mod drop_plan;
mod drop_point_path;
mod drop_point_resolve;
mod drop_requirement;
mod dump;
mod effect;
mod effect_check;
mod effect_counts;
mod effect_counts_host;
mod effect_counts_raw;
mod effect_identity;
mod effect_summary;
mod function_alias;
mod initialized;
mod initialized_alias;
mod initialized_alias_flow;
mod initialized_alias_i32;
mod initialized_alias_rank;
mod initialized_control;
mod initialized_drop_scope;
mod initialized_external_io;
mod initialized_external_io_effect;
mod initialized_external_io_input;
mod initialized_raw_memory;
mod initialized_rekey;
mod initialized_summary;
mod initialized_summary_apply;
mod initialized_summary_build;
mod initialized_summary_cells;
mod initialized_summary_condition;
mod initialized_summary_indirect_release;
mod initialized_summary_raw_release;
mod initialized_summary_release;
mod initialized_summary_release_build;
mod initialized_summary_variant_build;
mod initialized_summary_variant_condition;
mod initialized_summary_variant_requirement;
mod initialized_variant;
mod lower;
mod lower_aggregate;
mod lower_aggregate_projection;
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
mod owner_raw_view;
mod owner_return;
mod owner_return_apply;
mod owner_return_view;
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
pub use drop_elaboration::{
    compute_resource_drop_elaboration_plan, ResourceDropElaborationDrop,
    ResourceDropElaborationFunction, ResourceDropElaborationPlan, ResourceDropElaborationPlanError,
    ResourceDropElaborationPoint,
};
pub use drop_elaboration_hir_bridge::{
    validate_resource_drop_elaboration_hir_bridge, ResourceDropElaborationHirBridgeError,
};
pub use drop_model::{
    ResourceAutoDrop, ResourceAutoDropKind, ResourceDropFunctionPlan, ResourceDropPlan,
    ResourceDropPoint,
};
pub use drop_plan::compute_resource_drop_plan;
pub use drop_point_path::{ResourceDropPointPath, ResourceDropPointStep};
pub use drop_point_resolve::{
    resolve_resource_drop_point_end_scope, resolve_resource_drop_point_path,
    ResourceDropPointEndScope, ResourceDropPointOpKind, ResourceDropPointResolutionError,
};
pub use drop_requirement::{
    resource_drop_requirement_for_type, resource_type_needs_drop_code, ResourceDropField,
    ResourceDropRequirement,
};
pub use effect::{
    check_resource_effect_boundaries, ResourceEffectBoundaryDiagnostic,
    ResourceEffectBoundaryReport, ResourceEffectCallKind, ResourceEffectFunctionCheck,
};
pub use effect_counts::ResourceEffectCounts;
pub use effect_counts_host::{ExternalIoEffectCounts, NondetEffectCounts};
pub use effect_counts_raw::RawMemoryEffectCounts;
pub use initialized::check_resource_initialized_moves;
pub use lower::{lower_hir_module, lower_hir_module_skeleton};
pub use model::{
    AggregateKind, BorrowKind, BorrowState, BorrowStateEntry, CellState, CellStateEntry, EffectOp,
    ExternalIoOp, NondetOp, OwnerState, OwnerStateEntry, Place, PlaceProjection, PlaceRoot,
    PointerProvenance, RawBodyKind, RawMemoryOp, ResourceBlock, ResourceBlockId,
    ResourceCallTarget, ResourceExprKind, ResourceFunction, ResourceId, ResourceLocal,
    ResourceMatchArm, ResourceMatchPattern, ResourceModule, ResourceOffset, ResourceOp,
    ResourceState, ResourceTerminator, StorageId, StorageOrigin, StorageOriginEntry,
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
