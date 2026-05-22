//! Resource IR used as the typed input for ownership and lifetime checks.
//!
//! The compiler pipeline treats this module as the authority for ownership,
//! lifetime, effect, and initialized-state checks after typed HIR is lowered.

extern crate alloc;

mod address_projection;
mod borrow_call;
mod borrow_check;
mod borrow_scope;
mod borrow_state;
mod borrow_summary;
mod borrow_usage;
mod cell_state;
mod cell_state_raw_copy;
mod cell_state_raw_range;
mod cell_state_raw_range_append;
mod cell_state_raw_range_copy;
mod cell_state_raw_range_count;
mod cell_state_raw_range_cover;
#[cfg(test)]
mod cell_state_raw_range_cover_tests;
mod cell_state_raw_range_merge;
mod cell_state_raw_range_model;
mod cell_state_raw_range_offset;
mod cell_state_raw_range_value;
mod cell_state_raw_range_value_alias;
#[cfg(test)]
mod cell_state_raw_range_value_alias_tests;
#[cfg(test)]
mod cell_state_tests;
mod collection_slot_drop_proof;
mod collection_slot_drop_traversal;
mod collection_slot_drop_traversal_certified;
mod collection_slot_drop_traversal_known_range;
mod collection_slot_drop_traversal_range;
mod collection_slot_drop_traversal_slots;
mod collection_slot_drop_traversal_summary;
mod collection_slot_drop_traversal_summary_proof;
mod collection_slot_drop_traversal_symbolic_range;
mod collection_slot_lifecycle;
mod collection_slot_lifecycle_model;
#[cfg(test)]
mod collection_slot_lifecycle_storage_tests;
#[cfg(test)]
mod collection_slot_lifecycle_tests;
mod collection_slot_lifecycle_transition;
#[cfg(test)]
mod collection_slot_lifecycle_type_tests;
mod collection_slot_owner_carrier;
#[cfg(test)]
mod collection_slot_owner_carrier_tests;
mod collection_slot_owner_transfer;
mod collection_slot_owner_transfer_proof;
mod collection_slot_state_alias;
mod collection_slot_state_identity;
mod collection_slot_state_merge;
#[cfg(test)]
mod collection_slot_state_merge_tests;
mod collection_slot_state_release;
mod collection_slot_state_release_alias;
mod collection_slot_state_release_alias_precondition;
#[cfg(test)]
mod collection_slot_state_release_tests;
mod collection_slot_state_relocate;
#[cfg(test)]
mod collection_slot_state_relocate_tests;
mod collection_slot_state_return;
mod collection_slot_state_table;
#[cfg(test)]
mod collection_slot_state_table_tests;
mod collection_slot_state_transfer;
#[cfg(test)]
mod collection_slot_state_transfer_tests;
mod collection_slot_storage_release_proof;
mod collection_slot_summary_apply;
mod collection_slot_summary_apply_return_path;
mod collection_slot_summary_build;
mod collection_slot_summary_build_drop_traversal;
mod collection_slot_summary_build_event;
mod collection_slot_summary_build_nested;
mod collection_slot_summary_build_ops;
#[cfg(test)]
mod collection_slot_summary_build_ops_tests;
mod collection_slot_summary_build_range_bound;
mod collection_slot_summary_build_range_certificate;
#[cfg(test)]
mod collection_slot_summary_build_range_certificate_test_support;
#[cfg(test)]
mod collection_slot_summary_build_range_certificate_tests;
mod collection_slot_summary_build_range_lifetime;
#[cfg(test)]
mod collection_slot_summary_build_range_lifetime_test_support;
#[cfg(test)]
mod collection_slot_summary_build_range_lifetime_tests;
mod collection_slot_summary_build_range_preserve;
mod collection_slot_summary_build_range_preserve_op;
#[cfg(test)]
mod collection_slot_summary_build_range_preserve_tests;
mod collection_slot_summary_build_range_preserve_witness;
mod collection_slot_summary_build_range_preserve_witness_op;
mod collection_slot_summary_build_range_step;
mod collection_slot_summary_build_range_step_expr;
mod collection_slot_summary_build_range_witness;
mod collection_slot_summary_build_range_witness_drop;
mod collection_slot_summary_build_state;
mod collection_slot_summary_event_apply_proof;
mod collection_slot_summary_event_proof;
mod collection_slot_summary_match_state;
mod collection_slot_summary_model;
mod collection_slot_summary_projection;
mod collection_slot_summary_replay;
mod collection_slot_summary_replay_drop_traversal;
#[cfg(test)]
mod collection_slot_summary_replay_range_certificate_tests;
mod collection_slot_summary_return;
mod collection_slot_summary_return_build;
mod collection_slot_summary_return_call;
mod collection_slot_summary_return_collect;
mod collection_slot_summary_return_model;
mod collection_slot_summary_return_path;
mod collection_slot_summary_return_path_call;
mod collection_slot_summary_return_path_control;
mod collection_slot_summary_return_path_model;
mod collection_slot_summary_return_path_slots;
mod collection_slot_summary_return_path_state;
mod collection_slot_summary_return_path_value;
mod collection_slot_summary_return_state;
mod collection_slot_summary_return_unique;
mod collection_slot_summary_return_value;
mod collection_slot_summary_target;
#[cfg(test)]
mod collection_slot_summary_target_tests;
mod collection_slot_summary_translate;
mod collection_slot_summary_translate_drop;
mod compiler_memory_place;
mod condition_fact;
mod coverage;
mod coverage_hir;
mod coverage_hir_match;
mod coverage_hir_place;
mod coverage_hir_projection;
mod coverage_hir_projection_aggregate;
mod coverage_hir_raw;
mod coverage_hir_scope;
mod coverage_hir_transparent;
mod coverage_kind;
mod coverage_operation;
mod coverage_resource;
mod coverage_resource_collection_slot;
mod coverage_resource_place;
mod drop_call_identity;
mod drop_elaboration;
mod drop_elaboration_bindings;
mod drop_elaboration_hir_bridge;
mod drop_elaboration_validate;
mod drop_model;
mod drop_plan;
mod drop_plan_assignment;
mod drop_point_path;
mod drop_point_resolve;
mod drop_point_resolve_assignment;
mod drop_requirement;
mod dump;
mod effect;
mod effect_check;
mod effect_checked_memptr;
mod effect_counts;
mod effect_counts_host;
mod effect_counts_raw;
mod effect_diagnostic;
mod effect_identity;
mod effect_match;
mod effect_place_prefix;
mod effect_pointer_alias;
mod effect_raw_memory_identity;
mod effect_raw_provenance;
mod effect_return_escape;
#[cfg(test)]
mod effect_return_escape_tests;
mod effect_return_identity;
mod effect_return_owner_type;
mod effect_return_pointer;
mod effect_return_protection;
mod effect_return_summary_filter;
mod effect_summary;
mod effect_summary_identity;
#[cfg(test)]
mod effect_summary_identity_replay_tests;
mod effect_summary_identity_seed;
#[cfg(test)]
mod effect_summary_identity_tests;
mod effect_summary_pointer;
mod effect_summary_pointer_filter;
mod effect_summary_pointer_seed;
mod effect_summary_projection;
mod effect_summary_seed;
mod effect_summary_seed_alias;
mod effect_summary_seed_walk;
mod external_io_iov_layout;
mod function_alias;
mod host_dependent_length;
mod host_memory_address;
mod host_memory_contract;
#[cfg(test)]
mod host_memory_contract_tests;
mod host_size_contract;
mod i32_call_facts;
mod i32_call_facts_scale;
#[cfg(test)]
mod i32_call_facts_scale_tests;
#[cfg(test)]
mod i32_call_facts_tests;
mod i32_extent_proof;
mod i32_scalar_return_facts;
mod initialized;
mod initialized_alias;
mod initialized_alias_difference;
mod initialized_alias_difference_flow;
mod initialized_alias_flow;
mod initialized_alias_flow_apply;
mod initialized_alias_flow_projection;
mod initialized_alias_flow_raw;
mod initialized_alias_flow_value_projection;
mod initialized_alias_host_size;
mod initialized_alias_i32;
mod initialized_alias_i32_bounds;
mod initialized_alias_i32_condition;
mod initialized_alias_i32_condition_context;
#[cfg(test)]
mod initialized_alias_i32_condition_tests;
mod initialized_alias_i32_facts;
mod initialized_alias_i32_relation_condition;
mod initialized_alias_offset;
mod initialized_alias_origin;
#[cfg(test)]
mod initialized_alias_origin_tests;
mod initialized_alias_rank;
mod initialized_alias_raw_view;
#[cfg(test)]
mod initialized_alias_raw_view_tests;
mod initialized_alias_relation;
mod initialized_alias_relation_flow;
mod initialized_alias_relation_op;
mod initialized_alias_scalar;
mod initialized_alias_scalar_copy;
mod initialized_alias_scale;
#[cfg(test)]
mod initialized_alias_test_support;
#[cfg(test)]
mod initialized_alias_tests;
mod initialized_alias_type_size;
mod initialized_alias_utils;
mod initialized_availability;
mod initialized_availability_collection;
mod initialized_availability_diagnostic;
mod initialized_call;
mod initialized_call_args;
mod initialized_call_effect;
#[cfg(test)]
mod initialized_collection_slot;
mod initialized_collection_slot_alias;
mod initialized_collection_slot_apply;
mod initialized_collection_slot_dispatch;
mod initialized_collection_slot_proof;
mod initialized_collection_slot_relocate;
#[cfg(test)]
mod initialized_collection_slot_tests;
mod initialized_collection_slot_transfer;
mod initialized_control;
mod initialized_control_slot_transfer;
mod initialized_drop_assignment;
mod initialized_drop_requirement;
mod initialized_drop_scope;
mod initialized_external_io;
mod initialized_external_io_effect;
mod initialized_external_io_input;
mod initialized_external_io_iov;
mod initialized_external_io_payload;
mod initialized_external_seed;
mod initialized_host_dependent;
mod initialized_path_state;
mod initialized_raw_fill;
mod initialized_raw_memory;
mod initialized_raw_memory_access;
mod initialized_raw_memory_bulk;
mod initialized_raw_memory_dealloc_collection;
mod initialized_raw_view;
mod initialized_rekey;
mod initialized_scalar_flow;
mod initialized_scalar_flow_ops;
mod initialized_str_layout;
mod initialized_summary;
mod initialized_summary_apply;
mod initialized_summary_apply_param;
#[cfg(test)]
mod initialized_summary_apply_param_tests;
mod initialized_summary_apply_return;
mod initialized_summary_build;
mod initialized_summary_byte_range_model;
mod initialized_summary_cells;
mod initialized_summary_condition;
mod initialized_summary_engine;
mod initialized_summary_indirect_release;
mod initialized_summary_param_byte_range_count;
mod initialized_summary_param_byte_ranges;
mod initialized_summary_param_cells;
mod initialized_summary_raw_release;
mod initialized_summary_release;
mod initialized_summary_release_build;
#[cfg(test)]
mod initialized_summary_release_build_tests;
mod initialized_summary_release_model;
mod initialized_summary_return_byte_range_count;
mod initialized_summary_return_byte_ranges;
mod initialized_summary_seed;
#[cfg(test)]
mod initialized_summary_seed_tests;
mod initialized_summary_variant_build;
#[cfg(test)]
mod initialized_summary_variant_build_tests;
mod initialized_summary_variant_condition;
mod initialized_summary_variant_model;
mod initialized_summary_variant_requirement;
mod initialized_summary_variant_type;
mod initialized_summary_variant_unique;
mod initialized_variant;
mod initialized_variant_count;
mod lower;
mod lower_aggregate;
mod lower_aggregate_projection;
mod lower_aggregate_selector;
mod lower_call;
mod lower_collection_slot;
#[cfg(test)]
mod lower_collection_slot_relocate_tests;
#[cfg(test)]
mod lower_collection_slot_tests;
mod lower_condition;
mod lower_drop_call;
mod lower_layout_intrinsic;
mod lower_match;
mod lower_raw_address;
mod lower_raw_address_offset;
mod lower_raw_address_place;
mod lower_raw_address_return;
mod lower_raw_address_return_util;
mod lower_raw_address_source;
mod lower_raw_memory;
mod lower_temporary_scope;
mod lower_temporary_scope_op;
mod model;
mod owner_alias;
mod owner_check;
mod owner_check_utils;
mod owner_consumption;
mod owner_consumption_extent;
mod owner_control;
mod owner_drop;
mod owner_drop_scope;
mod owner_entry;
mod owner_expr;
mod owner_extent;
mod owner_extent_check;
mod owner_extent_compare;
mod owner_extent_coverage;
mod owner_extent_coverage_place;
mod owner_extent_expected;
mod owner_extent_summary;
mod owner_external_io;
mod owner_external_io_payload;
mod owner_flow;
mod owner_host_dependent_span;
mod owner_host_direct_span;
mod owner_host_iov_descriptor;
mod owner_host_memory_span;
mod owner_host_memory_summary;
mod owner_host_payload_extent;
mod owner_host_size_outputs;
mod owner_match_payload;
mod owner_raw_address;
mod owner_raw_memory;
mod owner_raw_memory_cell;
mod owner_raw_memory_span;
mod owner_raw_view;
mod owner_raw_view_model;
mod owner_raw_view_table;
mod owner_release;
mod owner_return;
mod owner_return_apply;
mod owner_return_apply_consumption;
mod owner_return_apply_extent;
mod owner_return_apply_place;
mod owner_return_apply_projection;
mod owner_return_apply_source;
mod owner_return_unknown;
mod owner_return_view;
mod owner_state;
mod owner_summary;
mod owner_summary_canonicalize;
mod owner_summary_consumed;
mod owner_summary_host_size_return;
mod owner_summary_i32_condition_leaf;
mod owner_summary_i32_leaf;
mod owner_summary_leaf;
mod owner_summary_owner_token_leaf;
mod owner_summary_owner_token_type;
mod owner_summary_parameters;
mod owner_summary_raw_alias;
mod owner_summary_raw_alias_branch;
mod owner_summary_raw_alias_walk;
mod owner_summary_raw_consumption;
mod owner_summary_raw_i32_leaf;
mod owner_summary_raw_transfer;
mod owner_summary_raw_use;
mod owner_summary_raw_use_branch;
mod owner_summary_raw_use_call;
mod owner_summary_raw_use_return;
mod owner_summary_raw_use_walk;
mod owner_summary_raw_view_return;
mod owner_summary_record;
mod owner_summary_resolved_variant;
mod owner_summary_seed_leaf;
mod owner_summary_size_return;
mod owner_summary_storage_origin;
mod owner_summary_type_params;
mod owner_summary_type_size_return;
mod owner_summary_update;
mod owner_summary_variant_ambiguous;
mod owner_summary_variant_build;
mod owner_summary_variant_conditions;
mod owner_summary_variant_construct;
mod owner_summary_variant_i32_conditions;
mod owner_summary_variant_leaf;
mod owner_summary_variant_match;
mod owner_summary_variant_path_conditions;
mod owner_summary_variant_paths;
mod owner_summary_variant_payload_conditions;
mod owner_summary_variant_projection;
mod owner_summary_variant_return;
mod owner_summary_variant_return_sources;
mod owner_transfer;
mod owner_variant;
mod owner_variant_apply;
mod owner_variant_condition_truth;
mod owner_variant_lifecycle;
mod owner_variant_record;
mod owner_variant_source_list;
mod owner_variant_unreachable;
mod owner_variant_utils;
mod owner_variant_value_condition;
mod place_utils;
mod raw_cell_lifecycle;
mod raw_cell_lifecycle_alias;
#[cfg(test)]
mod raw_cell_lifecycle_alias_tests;
mod raw_cell_value_flow;
mod raw_cell_value_flow_alias;
mod raw_cell_value_flow_alias_candidates;
#[cfg(test)]
mod raw_cell_value_flow_alias_scaled_tests;
#[cfg(test)]
mod raw_cell_value_flow_alias_tests;
mod raw_cell_value_flow_cell;
mod raw_cell_value_flow_proof;
#[cfg(test)]
mod raw_cell_value_flow_tests;
#[cfg(test)]
mod raw_pointer_owner_carrier_tests;
mod raw_pointer_type;
#[cfg(test)]
mod raw_pointer_type_tests;
mod raw_realloc;
mod report;
mod report_collection_slot;
mod result_variant;
mod scalar_primitive;
mod shadow;
mod storage_origin;
mod summary;
mod summary_dependency;
mod summary_index;
mod summary_projection;
mod summary_worklist;
mod summary_worklist_order;
mod timing;
mod trait_identity;
mod type_pattern;
mod type_var;
mod variant_name;

pub use borrow_check::check_resource_borrow_lifetimes;
pub use collection_slot_lifecycle::{
    apply_collection_slot_lifecycle_event, CollectionSlotLifecycleEvent, CollectionSlotLifecycleOp,
    CollectionSlotLifecycleRefutation, CollectionSlotReplacement, CollectionSlotState,
};
pub use collection_slot_state_table::{
    CollectionSlotStateEntry, CollectionSlotStateTable, CollectionSlotTableRefutation,
};
pub use coverage::{
    compare_hir_resource_lowering, compare_hir_resource_lowering_typed, ResourceCoverageCounts,
    ResourceCoverageDiagnostic, ResourceFunctionCoverage, ResourceLoweringCoverage,
};
pub use coverage_kind::ResourceCoverageKind;
pub use coverage_operation::ResourceCoveragePlaceOperation;
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
pub use drop_point_resolve_assignment::{
    resolve_resource_drop_point_assignment, ResourceDropPointAssignment,
};
pub use drop_requirement::{
    resource_drop_requirement_for_type, resource_type_needs_drop_code, ResourceDropField,
    ResourceDropRequirement,
};
pub use effect::{
    check_resource_effect_boundaries, check_resource_effect_boundaries_typed,
    ResourceEffectBoundaryReport, ResourceEffectFunctionCheck,
};
pub use effect_counts::ResourceEffectCounts;
pub use effect_counts_host::{ExternalIoEffectCounts, NondetEffectCounts};
pub use effect_counts_raw::RawMemoryEffectCounts;
pub use effect_diagnostic::{ResourceEffectBoundaryDiagnostic, ResourceEffectCallKind};
pub use initialized::check_resource_initialized_moves;
pub use lower::{lower_hir_module, lower_hir_module_skeleton};
pub use model::{
    AggregateKind, BorrowKind, BorrowState, BorrowStateEntry, CellState, CellStateEntry, EffectOp,
    ExternalIoOp, NondetOp, OwnerState, OwnerStateEntry, OwnerStorageExtent, Place,
    PlaceProjection, PlaceRoot, PointerProvenance, RawAddressAliasKind, RawAddressViewKind,
    RawBodyKind, RawMemoryOp, ResourceBlock, ResourceBlockId, ResourceCallTarget,
    ResourceConditionFact, ResourceExprKind, ResourceFunction, ResourceI32RelationOp, ResourceId,
    ResourceLocal, ResourceMatchArm, ResourceMatchBindMode, ResourceMatchPattern, ResourceModule,
    ResourceOffset, ResourceOp, ResourceState, ResourceTerminator, StorageId, StorageOrigin,
    StorageOriginEntry, UnknownEffectReason,
};
pub use owner_entry::check_resource_owner_obligations;
pub use report::{
    ResourceBorrowCheckDeferred, ResourceBorrowCheckReport, ResourceBorrowDiagnostic,
    ResourceBorrowFunctionCheck, ResourceBorrowOperation, ResourceCheckDeferred,
    ResourceCheckDiagnostic, ResourceCheckOperation, ResourceCheckReport, ResourceFunctionCheck,
    ResourceOwnerCheckDeferred, ResourceOwnerCheckReport, ResourceOwnerDiagnostic,
    ResourceOwnerFunctionCheck, ResourceOwnerOperation, ResourceSafetyShadowReport,
};
pub use shadow::check_hir_resource_safety_shadow;
