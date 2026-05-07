#!/usr/bin/env node

const fs = require('node:fs');
const path = require('node:path');

const ROOT = path.resolve(__dirname, '..');
const RESOURCE_DIR = path.join(ROOT, 'nepl-core', 'src', 'resource');

function readResource(name) {
    return fs.readFileSync(path.join(RESOURCE_DIR, name), 'utf8').replace(/\r\n/g, '\n');
}

function lineCount(text) {
    return text.split('\n').length;
}

function assert(condition, message) {
    if (!condition) {
        throw new Error(message);
    }
}

function assertFile(name) {
    const filePath = path.join(RESOURCE_DIR, name);
    assert(fs.existsSync(filePath), `missing resource module: ${name}`);
    return readResource(name);
}

function assertMissing(name) {
    const filePath = path.join(RESOURCE_DIR, name);
    assert(!fs.existsSync(filePath), `${name} must not be reintroduced as a monolithic checker`);
}

function assertContains(text, needle, source) {
    assert(text.includes(needle), `${source} must contain ${needle}`);
}

function assertNotContains(text, needle, source) {
    assert(!text.includes(needle), `${source} must not contain ${needle}`);
}

function escapeRegExp(text) {
    return text.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
}

function assertUsesResourceModuleSymbol(text, moduleName, symbolName, source) {
    const directImport = `super::${moduleName}::${symbolName}`;
    const groupedImport = new RegExp(
        `super::${escapeRegExp(moduleName)}::\\{[^}]*\\b${escapeRegExp(symbolName)}\\b[^}]*\\}`,
    );
    assert(
        text.includes(directImport) || groupedImport.test(text),
        `${source} must import ${symbolName} from super::${moduleName}`,
    );
}

const mod = assertFile('mod.rs');
assertMissing('check.rs');

for (const moduleName of [
    'initialized.rs',
    'borrow_check.rs',
    'borrow_summary.rs',
    'cell_state_raw_range.rs',
    'cell_state_raw_range_append.rs',
    'cell_state_raw_range_cover.rs',
    'cell_state_raw_range_cover_tests.rs',
    'cell_state_raw_range_count.rs',
    'cell_state_raw_range_merge.rs',
    'cell_state_raw_range_model.rs',
    'cell_state_raw_range_value.rs',
    'cell_state_raw_range_value_alias.rs',
    'owner_check.rs',
    'owner_consumption.rs',
    'owner_flow.rs',
    'owner_raw_view.rs',
    'owner_raw_view_table.rs',
    'owner_summary.rs',
    'owner_summary_variant_build.rs',
    'owner_summary_variant_conditions.rs',
    'owner_summary_variant_construct.rs',
    'owner_summary_variant_match.rs',
    'owner_summary_variant_paths.rs',
    'owner_summary_variant_return.rs',
    'owner_summary_update.rs',
    'owner_summary_leaf.rs',
    'owner_summary_raw_consumption.rs',
    'owner_summary_record.rs',
    'owner_summary_variant_leaf.rs',
    'owner_return.rs',
    'owner_return_apply.rs',
    'owner_return_unknown.rs',
    'owner_return_view.rs',
    'summary.rs',
    'summary_dependency.rs',
    'summary_worklist.rs',
    'timing.rs',
    'effect.rs',
    'effect_check.rs',
    'effect_counts.rs',
    'effect_counts_host.rs',
    'effect_counts_raw.rs',
    'effect_summary.rs',
    'effect_identity.rs',
    'coverage.rs',
    'coverage_hir.rs',
    'coverage_hir_projection.rs',
    'coverage_hir_projection_aggregate.rs',
    'coverage_hir_raw.rs',
    'coverage_hir_scope.rs',
    'coverage_resource.rs',
    'drop_elaboration.rs',
    'drop_elaboration_bindings.rs',
    'drop_elaboration_validate.rs',
    'drop_model.rs',
    'drop_plan.rs',
    'drop_plan_assignment.rs',
    'drop_point_path.rs',
    'drop_point_resolve.rs',
    'drop_point_resolve_assignment.rs',
    'drop_requirement.rs',
    'lower_raw_address.rs',
    'lower_raw_address_place.rs',
    'lower_raw_address_return.rs',
    'lower_raw_memory.rs',
    'report.rs',
    'shadow.rs',
    'initialized_alias.rs',
    'initialized_alias_difference.rs',
    'initialized_alias_difference_flow.rs',
    'initialized_alias_flow.rs',
    'initialized_alias_i32.rs',
    'initialized_alias_i32_facts.rs',
    'initialized_alias_origin.rs',
    'initialized_alias_rank.rs',
    'initialized_alias_relation.rs',
    'initialized_alias_relation_flow.rs',
    'initialized_alias_relation_op.rs',
    'initialized_alias_scalar.rs',
    'initialized_alias_scale.rs',
    'initialized_alias_tests.rs',
    'initialized_availability.rs',
    'initialized_drop_assignment.rs',
    'initialized_drop_scope.rs',
    'initialized_direct_call_scalar.rs',
    'initialized_external_io.rs',
    'initialized_external_io_effect.rs',
    'initialized_external_io_input.rs',
    'initialized_external_io_iov.rs',
    'initialized_external_io_iov_layout.rs',
    'initialized_external_io_payload.rs',
    'initialized_raw_fill.rs',
    'initialized_raw_memory.rs',
    'initialized_rekey.rs',
    'initialized_summary.rs',
    'initialized_alias_flow_apply.rs',
    'initialized_alias_flow_projection.rs',
    'initialized_alias_flow_raw.rs',
    'initialized_alias_flow_value_projection.rs',
    'initialized_summary_apply.rs',
    'initialized_summary_apply_param.rs',
    'initialized_summary_apply_return.rs',
    'initialized_summary_build.rs',
    'initialized_summary_byte_range_model.rs',
    'initialized_summary_cells.rs',
    'initialized_summary_condition.rs',
    'initialized_summary_indirect_release.rs',
    'initialized_summary_param_byte_range_count.rs',
    'initialized_summary_param_byte_ranges.rs',
    'initialized_summary_param_cells.rs',
    'initialized_summary_raw_release.rs',
    'initialized_summary_release.rs',
    'initialized_summary_release_build.rs',
    'initialized_summary_release_model.rs',
    'initialized_summary_return_byte_range_count.rs',
    'initialized_summary_return_byte_ranges.rs',
    'initialized_summary_variant_build.rs',
    'initialized_summary_variant_condition.rs',
    'initialized_summary_variant_requirement.rs',
    'initialized_summary_variant_unique.rs',
    'lower_aggregate.rs',
    'lower_aggregate_projection.rs',
    'lower_aggregate_selector.rs',
    'lower_condition.rs',
]) {
    assertFile(moduleName);
}

for (const moduleDecl of [
    'mod initialized;',
    'mod borrow_check;',
    'mod borrow_summary;',
    'mod cell_state_raw_range;',
    'mod cell_state_raw_range_append;',
    'mod cell_state_raw_range_cover;',
    'mod cell_state_raw_range_cover_tests;',
    'mod cell_state_raw_range_count;',
    'mod cell_state_raw_range_merge;',
    'mod cell_state_raw_range_model;',
    'mod cell_state_raw_range_value;',
    'mod cell_state_raw_range_value_alias;',
    'mod owner_check;',
    'mod owner_consumption;',
    'mod owner_flow;',
    'mod owner_raw_view;',
    'mod owner_raw_view_table;',
    'mod owner_summary;',
    'mod owner_summary_update;',
    'mod owner_summary_variant_build;',
    'mod owner_summary_raw_consumption;',
    'mod owner_summary_variant_conditions;',
    'mod owner_summary_variant_construct;',
    'mod owner_summary_variant_leaf;',
    'mod owner_summary_variant_match;',
    'mod owner_summary_variant_paths;',
    'mod owner_summary_variant_return;',
    'mod owner_summary_leaf;',
    'mod owner_summary_record;',
    'mod owner_return;',
    'mod owner_return_apply;',
    'mod owner_return_unknown;',
    'mod owner_return_view;',
    'mod summary;',
    'mod effect;',
    'mod effect_check;',
    'mod effect_counts;',
    'mod effect_counts_host;',
    'mod effect_counts_raw;',
    'mod effect_summary;',
    'mod effect_identity;',
    'mod coverage;',
    'mod coverage_hir;',
    'mod coverage_hir_projection;',
    'mod coverage_hir_projection_aggregate;',
    'mod coverage_hir_raw;',
    'mod coverage_hir_scope;',
    'mod coverage_resource;',
    'mod drop_elaboration;',
    'mod drop_elaboration_bindings;',
    'mod drop_elaboration_validate;',
    'mod drop_model;',
    'mod drop_plan;',
    'mod drop_plan_assignment;',
    'mod drop_point_path;',
    'mod drop_point_resolve;',
    'mod drop_point_resolve_assignment;',
    'mod drop_requirement;',
    'mod lower_raw_address;',
    'mod lower_raw_address_place;',
    'mod lower_raw_address_return;',
    'mod lower_raw_memory;',
    'mod report;',
    'mod shadow;',
    'mod initialized_alias;',
    'mod initialized_alias_difference;',
    'mod initialized_alias_difference_flow;',
    'mod initialized_alias_flow;',
    'mod initialized_alias_i32;',
    'mod initialized_alias_i32_facts;',
    'mod initialized_alias_origin;',
    'mod initialized_alias_rank;',
    'mod initialized_alias_relation;',
    'mod initialized_alias_relation_flow;',
    'mod initialized_alias_relation_op;',
    'mod initialized_alias_scalar;',
    'mod initialized_alias_scale;',
    'mod initialized_alias_tests;',
    'mod initialized_availability;',
    'mod initialized_drop_assignment;',
    'mod initialized_drop_scope;',
    'mod initialized_direct_call_scalar;',
    'mod initialized_external_io;',
    'mod initialized_external_io_input;',
    'mod initialized_external_io_effect;',
    'mod initialized_external_io_iov;',
    'mod initialized_external_io_iov_layout;',
    'mod initialized_external_io_payload;',
    'mod initialized_raw_fill;',
    'mod initialized_raw_memory;',
    'mod initialized_rekey;',
    'mod initialized_summary;',
    'mod initialized_alias_flow_apply;',
    'mod initialized_alias_flow_projection;',
    'mod initialized_alias_flow_raw;',
    'mod initialized_alias_flow_value_projection;',
    'mod initialized_summary_apply;',
    'mod initialized_summary_apply_param;',
    'mod initialized_summary_apply_return;',
    'mod initialized_summary_build;',
    'mod initialized_summary_byte_range_model;',
    'mod initialized_summary_cells;',
    'mod initialized_summary_condition;',
    'mod initialized_summary_indirect_release;',
    'mod initialized_summary_param_byte_range_count;',
    'mod initialized_summary_param_byte_ranges;',
    'mod initialized_summary_param_cells;',
    'mod initialized_summary_raw_release;',
    'mod initialized_summary_release;',
    'mod initialized_summary_release_build;',
    'mod initialized_summary_release_model;',
    'mod initialized_summary_return_byte_range_count;',
    'mod initialized_summary_return_byte_ranges;',
    'mod initialized_summary_variant_build;',
    'mod initialized_summary_variant_condition;',
    'mod initialized_summary_variant_requirement;',
    'mod initialized_summary_variant_unique;',
    'mod summary_dependency;',
    'mod summary_worklist;',
    'mod timing;',
    'mod lower_aggregate;',
    'mod lower_aggregate_projection;',
    'mod lower_aggregate_selector;',
    'mod lower_condition;',
]) {
    assertContains(mod, moduleDecl, 'resource/mod.rs');
}

assertNotContains(mod, 'mod check;', 'resource/mod.rs');

const initialized = readResource('initialized.rs');
const borrowCheck = readResource('borrow_check.rs');
const borrowSummary = readResource('borrow_summary.rs');
const ownerCheck = readResource('owner_check.rs');
const ownerConsumption = readResource('owner_consumption.rs');
const ownerSummary = readResource('owner_summary.rs');
const ownerReturn = readResource('owner_return.rs');
const ownerReturnApply = readResource('owner_return_apply.rs');
const ownerReturnUnknown = readResource('owner_return_unknown.rs');
const ownerReturnView = readResource('owner_return_view.rs');
const summary = readResource('summary.rs');
const effect = readResource('effect.rs');
const effectCheck = readResource('effect_check.rs');
const effectSummary = readResource('effect_summary.rs');
const coverage = readResource('coverage.rs');
const coverageHir = readResource('coverage_hir.rs');
const coverageHirProjection = readResource('coverage_hir_projection.rs');
const coverageHirProjectionAggregate = readResource('coverage_hir_projection_aggregate.rs');
const coverageHirRaw = readResource('coverage_hir_raw.rs');
const coverageHirScope = readResource('coverage_hir_scope.rs');
const coverageResource = readResource('coverage_resource.rs');
const dropElaboration = readResource('drop_elaboration.rs');
const dropElaborationBindings = readResource('drop_elaboration_bindings.rs');
const dropElaborationValidate = readResource('drop_elaboration_validate.rs');
const dropModel = readResource('drop_model.rs');
const dropPlan = readResource('drop_plan.rs');
const dropPlanAssignment = readResource('drop_plan_assignment.rs');
const dropPointPath = readResource('drop_point_path.rs');
const dropPointResolve = readResource('drop_point_resolve.rs');
const dropPointResolveAssignment = readResource('drop_point_resolve_assignment.rs');
const dropRequirement = readResource('drop_requirement.rs');
const lower = readResource('lower.rs');
const lowerAggregate = readResource('lower_aggregate.rs');
const lowerAggregateProjection = readResource('lower_aggregate_projection.rs');
const lowerAggregateSelector = readResource('lower_aggregate_selector.rs');
const lowerRawAddress = readResource('lower_raw_address.rs');
const lowerRawAddressPlace = readResource('lower_raw_address_place.rs');
const lowerRawAddressReturn = readResource('lower_raw_address_return.rs');
const lowerRawMemory = readResource('lower_raw_memory.rs');
const initializedAliasOrigin = readResource('initialized_alias_origin.rs');
const initializedAliasRelation = readResource('initialized_alias_relation.rs');
const initializedAliasRelationFlow = readResource('initialized_alias_relation_flow.rs');
const initializedAliasRelationOp = readResource('initialized_alias_relation_op.rs');
const initializedAliasScalar = readResource('initialized_alias_scalar.rs');

assertContains(initialized, 'struct ResourceCheckEngine', 'initialized.rs');
assertContains(borrowCheck, 'struct ResourceBorrowCheckEngine', 'borrow_check.rs');
assertContains(ownerCheck, 'struct ResourceOwnerCheckEngine', 'owner_check.rs');
assertContains(effectCheck, 'struct ResourceEffectBoundaryEngine', 'effect_check.rs');

assertNotContains(effect, 'struct ResourceEffectBoundaryEngine', 'effect.rs');
assertContains(effect, 'pub fn check_resource_effect_boundaries', 'effect.rs');
assertContains(coverage, 'pub fn compare_hir_resource_lowering_typed', 'coverage.rs');
assertContains(coverageHir, 'pub(super) fn hir_function_coverage', 'coverage_hir.rs');
assertContains(
    coverageHirProjection,
    'pub(super) fn field_get_call_owner',
    'coverage_hir_projection.rs',
);
assertContains(
    coverageHirProjectionAggregate,
    'pub(super) fn aggregate_field_exists',
    'coverage_hir_projection_aggregate.rs',
);
assertContains(
    coverageHirProjectionAggregate,
    'pub(super) fn aggregate_field_matches_selector',
    'coverage_hir_projection_aggregate.rs',
);
assertContains(
    coverageHirRaw,
    'pub(super) fn should_count_raw_memory_call',
    'coverage_hir_raw.rs',
);
assertContains(
    coverageHirScope,
    'struct HirCoverageContext',
    'coverage_hir_scope.rs',
);
assertContains(
    coverageResource,
    'pub(super) fn resource_function_coverage',
    'coverage_resource.rs',
);
assertContains(
    dropElaboration,
    'pub fn compute_resource_drop_elaboration_plan',
    'drop_elaboration.rs',
);
assertContains(
    dropElaborationBindings,
    'pub(super) fn function_source_bindings',
    'drop_elaboration_bindings.rs',
);
assertContains(
    dropElaborationValidate,
    'pub(super) fn validate_drop_point_kind',
    'drop_elaboration_validate.rs',
);
assertContains(
    dropModel,
    'pub struct ResourceDropPoint',
    'drop_model.rs',
);
assertContains(
    dropPlan,
    'pub fn compute_resource_drop_plan',
    'drop_plan.rs',
);
assertContains(
    dropPlanAssignment,
    'pub(super) fn assignment_overwrite_drop_point',
    'drop_plan_assignment.rs',
);
assertContains(
    dropPointPath,
    'pub enum ResourceDropPointStep',
    'drop_point_path.rs',
);
assertContains(
    dropPointResolve,
    'pub fn resolve_resource_drop_point_end_scope',
    'drop_point_resolve.rs',
);
assertContains(
    dropPointResolveAssignment,
    'pub fn resolve_resource_drop_point_assignment',
    'drop_point_resolve_assignment.rs',
);
assertContains(
    dropRequirement,
    'pub fn resource_drop_requirement_for_type',
    'drop_requirement.rs',
);
assertNotContains(
    lowerRawAddress,
    'pub(super) fn push_user_raw_address_return_semantics',
    'lower_raw_address.rs',
);
assertContains(
    lowerRawAddress,
    'pub(super) fn push_named_raw_address_semantics',
    'lower_raw_address.rs',
);
assertContains(
    lowerRawAddressPlace,
    'pub(super) fn raw_address_place_from_actual_argument',
    'lower_raw_address_place.rs',
);
assertContains(
    lowerRawAddressReturn,
    'pub(super) fn push_transparent_raw_address_return_projection',
    'lower_raw_address_return.rs',
);
assertContains(
    lowerRawMemory,
    'pub(super) fn raw_memory_op_from_name',
    'lower_raw_memory.rs',
);
assertContains(
    initializedAliasOrigin,
    'pub(super) struct RawValueOrigins',
    'initialized_alias_origin.rs',
);
assertContains(
    initializedAliasScalar,
    'pub(super) struct I32AliasFacts',
    'initialized_alias_scalar.rs',
);
assertContains(
    initializedAliasRelation,
    'pub(super) struct I32RelationFact',
    'initialized_alias_relation.rs',
);
assertContains(
    initializedAliasRelation,
    'pub(super) struct I32RelationFacts',
    'initialized_alias_relation.rs',
);
assertContains(
    initializedAliasRelationFlow,
    'pub(super) fn facts_with_replaced_prefix',
    'initialized_alias_relation_flow.rs',
);
assertContains(
    initializedAliasRelationOp,
    'pub(super) fn relation_negation',
    'initialized_alias_relation_op.rs',
);
assertContains(
    initializedAliasRelationOp,
    'pub(super) fn relation_holds',
    'initialized_alias_relation_op.rs',
);
assertNotContains(lower, 'struct RawAddressSource', 'lower.rs');
assertContains(
    lowerAggregate,
    'pub(super) fn lower_compiler_field_load_source',
    'lower_aggregate.rs',
);
assertContains(
    lowerAggregateProjection,
    'pub(super) fn aggregate_field_projection_by_name',
    'lower_aggregate_projection.rs',
);
assertContains(
    lowerAggregateSelector,
    'pub(super) fn aggregate_field_selector',
    'lower_aggregate_selector.rs',
);
assertUsesResourceModuleSymbol(
    borrowSummary,
    'borrow_check',
    'ResourceBorrowCheckEngine',
    'borrow_summary.rs',
);
assertUsesResourceModuleSymbol(
    ownerConsumption,
    'owner_check',
    'ResourceOwnerCheckEngine',
    'owner_consumption.rs',
);
assertUsesResourceModuleSymbol(
    ownerSummary,
    'owner_check',
    'ResourceOwnerCheckEngine',
    'owner_summary.rs',
);
assertUsesResourceModuleSymbol(
    ownerReturn,
    'owner_check',
    'ResourceOwnerCheckEngine',
    'owner_return.rs',
);
assertUsesResourceModuleSymbol(
    ownerReturnApply,
    'owner_check',
    'ResourceOwnerCheckEngine',
    'owner_return_apply.rs',
);
assertUsesResourceModuleSymbol(
    ownerReturnUnknown,
    'owner_check',
    'ResourceOwnerCheckEngine',
    'owner_return_unknown.rs',
);
assertUsesResourceModuleSymbol(
    ownerReturnView,
    'owner_check',
    'ResourceOwnerCheckEngine',
    'owner_return_view.rs',
);
assertContains(
    ownerReturnUnknown,
    'apply_unknown_indirect_call_return_owner',
    'owner_return_unknown.rs',
);
assertNotContains(
    ownerReturn,
    'fn apply_unknown_indirect_call_return_owner',
    'owner_return.rs',
);
assertUsesResourceModuleSymbol(
    effectSummary,
    'effect_check',
    'ResourceEffectBoundaryEngine',
    'effect_summary.rs',
);

const maxLines = new Map([
    ['effect.rs', 160],
    ['effect_counts.rs', 80],
    ['effect_counts_host.rs', 220],
    ['effect_counts_raw.rs', 80],
    ['initialized.rs', 750],
    ['borrow_call.rs', 120],
    ['borrow_check.rs', 550],
    ['borrow_scope.rs', 100],
    ['borrow_summary.rs', 120],
    ['borrow_usage.rs', 260],
    ['cell_state_raw_range.rs', 140],
    ['cell_state_raw_range_append.rs', 120],
    ['cell_state_raw_range_cover.rs', 140],
    ['cell_state_raw_range_cover_tests.rs', 80],
    ['cell_state_raw_range_count.rs', 90],
    ['cell_state_raw_range_merge.rs', 120],
    ['cell_state_raw_range_model.rs', 80],
    ['cell_state_raw_range_value.rs', 80],
    ['cell_state_raw_range_value_alias.rs', 80],
    ['owner_check.rs', 800],
    ['owner_consumption.rs', 80],
    ['owner_flow.rs', 620],
    ['owner_raw_view.rs', 180],
    ['owner_raw_view_table.rs', 160],
    ['owner_summary.rs', 380],
    ['owner_summary_variant_build.rs', 360],
    ['owner_summary_variant_conditions.rs', 260],
    ['owner_summary_variant_construct.rs', 140],
    ['owner_summary_variant_match.rs', 140],
    ['owner_summary_variant_paths.rs', 380],
    ['owner_summary_variant_return.rs', 220],
    ['owner_summary_update.rs', 100],
    ['owner_summary_leaf.rs', 260],
    ['owner_summary_raw_consumption.rs', 140],
    ['owner_summary_record.rs', 260],
    ['owner_summary_variant_leaf.rs', 80],
    ['owner_return.rs', 220],
    ['owner_return_apply.rs', 280],
    ['owner_return_unknown.rs', 180],
    ['owner_return_view.rs', 80],
    ['summary_dependency.rs', 220],
    ['summary_worklist.rs', 100],
    ['timing.rs', 80],
    ['effect_check.rs', 700],
    ['summary.rs', 300],
    ['effect_summary.rs', 250],
    ['coverage.rs', 280],
    ['coverage_hir.rs', 240],
    ['coverage_hir_place.rs', 120],
    ['coverage_hir_projection.rs', 280],
    ['coverage_hir_projection_aggregate.rs', 180],
    ['coverage_hir_raw.rs', 80],
    ['coverage_hir_scope.rs', 100],
    ['coverage_resource.rs', 520],
    ['drop_elaboration.rs', 220],
    ['drop_elaboration_bindings.rs', 140],
    ['drop_elaboration_validate.rs', 120],
    ['drop_model.rs', 80],
    ['drop_plan.rs', 160],
    ['drop_plan_assignment.rs', 80],
    ['drop_point_path.rs', 80],
    ['drop_point_resolve.rs', 220],
    ['drop_point_resolve_assignment.rs', 80],
    ['drop_requirement.rs', 220],
    ['lower.rs', 1150],
    ['lower_aggregate.rs', 320],
    ['lower_aggregate_projection.rs', 180],
    ['lower_aggregate_selector.rs', 100],
    ['lower_condition.rs', 140],
    ['lower_raw_address.rs', 620],
    ['lower_raw_address_place.rs', 180],
    ['lower_raw_address_return.rs', 430],
    ['lower_raw_memory.rs', 120],
    ['initialized_alias.rs', 520],
    ['initialized_alias_difference.rs', 80],
    ['initialized_alias_difference_flow.rs', 120],
    ['initialized_alias_flow.rs', 550],
    ['initialized_alias_i32_facts.rs', 180],
    ['initialized_alias_i32.rs', 80],
    ['initialized_alias_origin.rs', 160],
    ['initialized_alias_rank.rs', 120],
    ['initialized_alias_relation.rs', 100],
    ['initialized_alias_relation_flow.rs', 100],
    ['initialized_alias_relation_op.rs', 80],
    ['initialized_alias_scalar.rs', 180],
    ['initialized_alias_scale.rs', 140],
    ['initialized_alias_tests.rs', 120],
    ['initialized_availability.rs', 120],
    ['initialized_drop_assignment.rs', 100],
    ['initialized_drop_scope.rs', 80],
    ['initialized_direct_call_scalar.rs', 150],
    ['initialized_external_io.rs', 140],
    ['initialized_external_io_effect.rs', 90],
    ['initialized_external_io_input.rs', 80],
    ['initialized_external_io_iov.rs', 130],
    ['initialized_external_io_iov_layout.rs', 120],
    ['initialized_external_io_payload.rs', 90],
    ['initialized_raw_fill.rs', 120],
    ['initialized_raw_memory.rs', 300],
    ['initialized_rekey.rs', 160],
    ['initialized_summary.rs', 80],
    ['initialized_alias_flow_apply.rs', 180],
    ['initialized_alias_flow_projection.rs', 120],
    ['initialized_alias_flow_raw.rs', 320],
    ['initialized_alias_flow_value_projection.rs', 520],
    ['initialized_summary_apply.rs', 130],
    ['initialized_summary_apply_param.rs', 100],
    ['initialized_summary_apply_return.rs', 120],
    ['initialized_summary_build.rs', 260],
    ['initialized_summary_byte_range_model.rs', 80],
    ['initialized_summary_cells.rs', 140],
    ['initialized_summary_condition.rs', 80],
    ['initialized_summary_indirect_release.rs', 120],
    ['initialized_summary_param_byte_range_count.rs', 100],
    ['initialized_summary_param_byte_ranges.rs', 140],
    ['initialized_summary_param_cells.rs', 120],
    ['initialized_summary_raw_release.rs', 80],
    ['initialized_summary_release.rs', 100],
    ['initialized_summary_release_build.rs', 420],
    ['initialized_summary_release_model.rs', 80],
    ['initialized_summary_return_byte_range_count.rs', 100],
    ['initialized_summary_return_byte_ranges.rs', 140],
    ['initialized_summary_variant_build.rs', 260],
    ['initialized_summary_variant_condition.rs', 140],
    ['initialized_summary_variant_requirement.rs', 120],
    ['initialized_summary_variant_unique.rs', 80],
]);

for (const [name, limit] of maxLines) {
    const lines = lineCount(readResource(name));
    assert(lines <= limit, `${name} has ${lines} lines; responsibility split limit is ${limit}`);
}

console.log('resource checker responsibility ok');
