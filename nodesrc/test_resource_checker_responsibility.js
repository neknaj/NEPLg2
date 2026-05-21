#!/usr/bin/env node

const fs = require('node:fs');
const path = require('node:path');

const ROOT = path.resolve(__dirname, '..');
const CORE_SRC_DIR = path.join(ROOT, 'nepl-core', 'src');
const RESOURCE_DIR = path.join(ROOT, 'nepl-core', 'src', 'resource');

function readResource(name) {
    return fs.readFileSync(path.join(RESOURCE_DIR, name), 'utf8').replace(/\r\n/g, '\n');
}

function readCoreSrc(name) {
    return fs.readFileSync(path.join(CORE_SRC_DIR, name), 'utf8').replace(/\r\n/g, '\n');
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

function assertMatches(text, pattern, source) {
    assert(pattern.test(text), `${source} must match ${pattern}`);
}

function braceBlock(text, signature, source) {
    const start = text.indexOf(signature);
    assert(start >= 0, `${source} must contain ${signature}`);
    const open = text.indexOf('{', start);
    assert(open >= 0, `${source} must contain ${signature} body`);
    let depth = 0;
    for (let i = open; i < text.length; i += 1) {
        const ch = text[i];
        if (ch === '{') {
            depth += 1;
        } else if (ch === '}') {
            depth -= 1;
            if (depth === 0) {
                return text.slice(open + 1, i);
            }
        }
    }
    throw new Error(`${source} has unterminated body for ${signature}`);
}

function escapeRegExp(text) {
    return text.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
}

function moduleNameFromFile(name) {
    return name.replace(/\.rs$/, '');
}

function assertModuleDeclared(modText, fileName) {
    const moduleName = escapeRegExp(moduleNameFromFile(fileName));
    const declaration = new RegExp(`(^|\\n)(#\\[cfg\\(test\\)\\]\\n)?mod ${moduleName};`);
    const pathDeclaration = fs
        .readdirSync(RESOURCE_DIR)
        .filter((resourceFileName) => resourceFileName.endsWith('.rs'))
        .some((resourceFileName) =>
            readResource(resourceFileName).includes(`#[path = "${fileName}"]`),
        );
    assert(
        declaration.test(modText) || pathDeclaration,
        `resource/mod.rs or a resource test module must declare ${fileName}`,
    );
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
const resourcePrimitivesCompilerMemory = readCoreSrc(
    path.join('resource_primitives', 'compiler_memory.rs'),
);
const typeCtxSource = readCoreSrc('types.rs');
const typecheckDriver = readCoreSrc(path.join('typecheck', 'driver.rs'));
assertMissing('check.rs');

for (const moduleName of [
    'address_projection.rs',
    'initialized.rs',
    'borrow_check.rs',
    'borrow_summary.rs',
    'borrow_state.rs',
    'cell_state.rs',
    'cell_state_raw_copy.rs',
    'cell_state_raw_range.rs',
    'cell_state_raw_range_append.rs',
    'cell_state_raw_range_cover.rs',
    'cell_state_raw_range_cover_tests.rs',
    'cell_state_raw_range_count.rs',
    'cell_state_raw_range_merge.rs',
    'cell_state_raw_range_model.rs',
    'cell_state_raw_range_copy.rs',
    'cell_state_raw_range_offset.rs',
    'cell_state_raw_range_value.rs',
    'cell_state_raw_range_value_alias.rs',
    'cell_state_raw_range_value_alias_tests.rs',
    'cell_state_tests.rs',
    'collection_slot_drop_traversal_certified.rs',
    'collection_slot_drop_proof.rs',
    'collection_slot_drop_traversal.rs',
    'collection_slot_drop_traversal_range.rs',
    'collection_slot_drop_traversal_summary.rs',
    'collection_slot_lifecycle.rs',
    'collection_slot_lifecycle_model.rs',
    'collection_slot_lifecycle_storage_tests.rs',
    'collection_slot_lifecycle_tests.rs',
    'collection_slot_lifecycle_transition.rs',
    'collection_slot_lifecycle_type_tests.rs',
    'collection_slot_owner_transfer.rs',
    'collection_slot_owner_transfer_proof.rs',
    'collection_slot_state_identity.rs',
    'collection_slot_summary_apply.rs',
    'collection_slot_summary_apply_return_path.rs',
    'collection_slot_summary_build.rs',
    'collection_slot_summary_build_drop_traversal.rs',
    'collection_slot_summary_build_event.rs',
    'collection_slot_summary_build_ops.rs',
    'collection_slot_summary_build_state.rs',
    'collection_slot_summary_event_apply_proof.rs',
    'collection_slot_summary_event_proof.rs',
    'collection_slot_summary_match_state.rs',
    'collection_slot_summary_model.rs',
    'collection_slot_summary_replay.rs',
    'collection_slot_summary_replay_drop_traversal.rs',
    'collection_slot_summary_return.rs',
    'collection_slot_summary_return_build.rs',
    'collection_slot_summary_return_call.rs',
    'collection_slot_summary_return_collect.rs',
    'collection_slot_summary_return_model.rs',
    'collection_slot_summary_return_path.rs',
    'collection_slot_summary_return_path_call.rs',
    'collection_slot_summary_return_path_control.rs',
    'collection_slot_summary_return_path_model.rs',
    'collection_slot_summary_return_path_slots.rs',
    'collection_slot_summary_return_path_state.rs',
    'collection_slot_summary_return_path_value.rs',
    'collection_slot_summary_return_state.rs',
    'collection_slot_summary_target.rs',
    'collection_slot_summary_translate.rs',
    'collection_slot_summary_translate_drop.rs',
    'collection_slot_summary_return_unique.rs',
    'collection_slot_summary_return_value.rs',
    'collection_slot_storage_release_proof.rs',
    'collection_slot_state_merge.rs',
    'collection_slot_state_merge_tests.rs',
    'collection_slot_state_release.rs',
    'collection_slot_state_release_tests.rs',
    'collection_slot_state_relocate.rs',
    'collection_slot_state_relocate_tests.rs',
    'collection_slot_state_return.rs',
    'collection_slot_state_table.rs',
    'collection_slot_state_table_tests.rs',
    'collection_slot_state_transfer.rs',
    'collection_slot_state_transfer_tests.rs',
    'owner_check.rs',
    'owner_check_utils.rs',
    'owner_consumption.rs',
    'owner_consumption_extent.rs',
    'owner_drop.rs',
    'owner_expr.rs',
    'owner_external_io.rs',
    'owner_host_direct_span.rs',
    'owner_host_memory_span.rs',
    'owner_host_memory_summary.rs',
    'owner_extent.rs',
    'owner_extent_check.rs',
    'owner_extent_compare.rs',
    'owner_extent_coverage.rs',
    'owner_extent_coverage_place.rs',
    'owner_extent_expected.rs',
    'owner_extent_summary.rs',
    'owner_external_io_payload.rs',
    'owner_flow.rs',
    'owner_host_dependent_span.rs',
    'owner_host_iov_descriptor.rs',
    'owner_host_payload_extent.rs',
    'owner_host_size_outputs.rs',
    'owner_match_payload.rs',
    'owner_raw_view.rs',
    'owner_raw_memory.rs',
    'owner_raw_memory_cell.rs',
    'owner_raw_memory_span.rs',
    'owner_raw_view_model.rs',
    'owner_raw_view_table.rs',
    'owner_summary.rs',
    'owner_summary_canonicalize.rs',
    'owner_summary_consumed.rs',
    'owner_summary_host_size_return.rs',
    'owner_summary_i32_condition_leaf.rs',
    'owner_summary_i32_leaf.rs',
    'owner_summary_owner_token_leaf.rs',
    'owner_summary_owner_token_leaf_tests.rs',
    'owner_summary_owner_token_type.rs',
    'owner_summary_parameters.rs',
    'owner_summary_raw_alias.rs',
    'owner_summary_raw_alias_branch.rs',
    'owner_summary_raw_alias_walk.rs',
    'owner_summary_raw_i32_leaf.rs',
    'owner_summary_type_size_return.rs',
    'owner_summary_type_params.rs',
    'owner_summary_variant_build.rs',
    'owner_summary_variant_conditions.rs',
    'owner_summary_variant_construct.rs',
    'owner_summary_variant_i32_conditions.rs',
    'owner_summary_variant_match.rs',
    'owner_summary_variant_path_conditions.rs',
    'owner_summary_variant_payload_conditions.rs',
    'owner_summary_variant_paths.rs',
    'owner_summary_variant_return.rs',
    'owner_summary_variant_return_sources.rs',
    'owner_summary_update.rs',
    'owner_summary_leaf.rs',
    'owner_summary_raw_consumption.rs',
    'owner_summary_raw_transfer_tests.rs',
    'owner_summary_raw_use.rs',
    'owner_summary_raw_use_branch.rs',
    'owner_summary_raw_use_call.rs',
    'owner_summary_raw_use_return.rs',
    'owner_summary_raw_use_walk.rs',
    'owner_summary_raw_view_return.rs',
    'owner_summary_update_tests.rs',
    'owner_summary_record.rs',
    'owner_summary_seed_leaf.rs',
    'owner_summary_size_return.rs',
    'owner_summary_storage_origin.rs',
    'owner_summary_variant_leaf.rs',
    'owner_summary_variant_projection.rs',
    'owner_return.rs',
    'owner_return_apply.rs',
    'owner_return_apply_consumption.rs',
    'owner_return_apply_extent.rs',
    'owner_return_unknown.rs',
    'owner_return_view.rs',
    'owner_variant.rs',
    'owner_variant_apply.rs',
    'owner_variant_condition_truth.rs',
    'owner_variant_lifecycle.rs',
    'owner_variant_record.rs',
    'owner_variant_source_list.rs',
    'owner_variant_unreachable.rs',
    'owner_variant_utils.rs',
    'owner_variant_value_condition.rs',
    'result_variant.rs',
    'variant_name.rs',
    'summary.rs',
    'summary_dependency.rs',
    'summary_index.rs',
    'summary_worklist.rs',
    'summary_worklist_order.rs',
    'summary_worklist_tests.rs',
    'timing.rs',
    'effect.rs',
    'effect_checked_memptr.rs',
    'effect_check.rs',
    'effect_counts.rs',
    'effect_counts_host.rs',
    'effect_counts_raw.rs',
    'effect_diagnostic.rs',
    'effect_summary.rs',
    'effect_summary_identity.rs',
    'effect_summary_identity_seed.rs',
    'effect_summary_identity_replay_tests.rs',
    'effect_summary_identity_tests.rs',
    'effect_summary_pointer.rs',
    'effect_summary_pointer_filter.rs',
    'effect_summary_pointer_seed.rs',
    'effect_summary_projection.rs',
    'effect_summary_seed.rs',
    'effect_summary_seed_alias.rs',
    'effect_summary_seed_walk.rs',
    'effect_identity.rs',
    'effect_match.rs',
    'effect_place_prefix.rs',
    'effect_pointer_alias.rs',
    'effect_raw_provenance.rs',
    'effect_raw_memory_identity.rs',
    'effect_return_escape.rs',
    'effect_return_escape_tests.rs',
    'effect_return_identity.rs',
    'effect_return_owner_type.rs',
    'effect_return_pointer.rs',
    'effect_return_protection.rs',
    'effect_return_summary_filter.rs',
    'effect_return_summary_filter_tests.rs',
    'coverage.rs',
    'coverage_hir.rs',
    'coverage_hir_projection.rs',
    'coverage_hir_projection_aggregate.rs',
    'coverage_hir_raw.rs',
    'coverage_hir_scope.rs',
    'coverage_hir_transparent.rs',
    'coverage_kind.rs',
    'coverage_operation.rs',
    'coverage_resource.rs',
    'coverage_resource_collection_slot.rs',
    'coverage_resource_place.rs',
    'dump.rs',
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
    'lower_collection_slot.rs',
    'lower_collection_slot_relocate_tests.rs',
    'lower_collection_slot_tests.rs',
    'lower_raw_address.rs',
    'lower_raw_address_offset.rs',
    'lower_raw_address_place.rs',
    'lower_raw_address_return.rs',
    'lower_raw_address_return_util.rs',
    'lower_raw_memory.rs',
    'lower_temporary_scope.rs',
    'lower_temporary_scope_op.rs',
    'report.rs',
    'shadow.rs',
    'initialized_alias.rs',
    'initialized_alias_difference.rs',
    'initialized_alias_difference_flow.rs',
    'initialized_alias_flow.rs',
    'initialized_alias_i32.rs',
    'initialized_alias_i32_bounds.rs',
    'initialized_alias_i32_condition.rs',
    'initialized_alias_i32_condition_context.rs',
    'initialized_alias_i32_condition_tests.rs',
    'initialized_alias_i32_facts.rs',
    'initialized_alias_offset.rs',
    'initialized_alias_i32_relation_condition.rs',
    'initialized_alias_origin.rs',
    'initialized_alias_origin_tests.rs',
    'initialized_alias_rank.rs',
    'initialized_alias_relation.rs',
    'initialized_alias_relation_flow.rs',
    'initialized_alias_relation_op.rs',
    'initialized_alias_scalar.rs',
    'initialized_alias_scalar_copy.rs',
    'initialized_alias_scale.rs',
    'initialized_alias_test_support.rs',
    'initialized_alias_utils.rs',
    'initialized_alias_tests.rs',
    'initialized_alias_raw_view_tests.rs',
    'initialized_call_args.rs',
    'initialized_call_effect.rs',
    'initialized_collection_slot_alias.rs',
    'initialized_collection_slot_apply.rs',
    'initialized_collection_slot_dispatch.rs',
    'initialized_collection_slot_proof.rs',
    'initialized_collection_slot_relocate.rs',
    'initialized_collection_slot_tests.rs',
    'initialized_collection_slot_transfer.rs',
    'initialized_control.rs',
    'initialized_control_slot_transfer.rs',
    'initialized_availability.rs',
    'initialized_drop_assignment.rs',
    'initialized_drop_requirement.rs',
    'initialized_drop_scope.rs',
    'i32_call_facts.rs',
    'i32_call_facts_scale.rs',
    'i32_call_facts_scale_tests.rs',
    'i32_call_facts_tests.rs',
    'i32_extent_proof.rs',
    'initialized_external_io.rs',
    'external_io_iov_layout.rs',
    'host_dependent_length.rs',
    'host_memory_address.rs',
    'host_memory_contract.rs',
    'host_memory_contract_tests.rs',
    'host_size_contract.rs',
    'initialized_external_io_effect.rs',
    'initialized_external_io_input.rs',
    'initialized_external_io_iov.rs',
    'initialized_external_io_payload.rs',
    'initialized_host_dependent.rs',
    'initialized_path_state.rs',
    'initialized_raw_fill.rs',
    'initialized_raw_memory.rs',
    'initialized_raw_memory_access.rs',
    'initialized_raw_memory_bulk.rs',
    'raw_cell_lifecycle.rs',
    'raw_cell_value_flow.rs',
    'raw_cell_value_flow_alias.rs',
    'raw_cell_value_flow_cell.rs',
    'raw_cell_value_flow_proof.rs',
    'raw_cell_value_flow_alias_tests.rs',
    'raw_cell_value_flow_tests.rs',
    'initialized_rekey.rs',
    'initialized_scalar_flow.rs',
    'initialized_scalar_flow_ops.rs',
    'initialized_str_layout.rs',
    'initialized_summary.rs',
    'initialized_alias_flow_apply.rs',
    'initialized_alias_flow_projection.rs',
    'initialized_alias_flow_raw.rs',
    'initialized_alias_flow_value_projection.rs',
    'initialized_alias_host_size.rs',
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
    'initialized_summary_release_build_tests.rs',
    'initialized_summary_release_model.rs',
    'initialized_summary_seed.rs',
    'initialized_summary_seed_tests.rs',
    'initialized_summary_return_byte_range_count.rs',
    'initialized_summary_return_byte_ranges.rs',
    'initialized_summary_variant_build.rs',
    'initialized_summary_variant_build_tests.rs',
    'initialized_summary_variant_condition.rs',
    'initialized_summary_variant_requirement.rs',
    'initialized_summary_variant_type.rs',
    'initialized_summary_variant_unique.rs',
    'initialized_variant.rs',
    'compiler_memory_place.rs',
    'compiler_memory_place_tests.rs',
    'lower_call.rs',
    'lower_aggregate.rs',
    'lower_aggregate_projection.rs',
    'lower_aggregate_selector.rs',
    'lower_condition.rs',
    'lower_layout_intrinsic.rs',
    'model.rs',
    'scalar_primitive.rs',
    'owner_control.rs',
    'owner_drop_scope.rs',
    'owner_state.rs',
    'place_utils.rs',
    'raw_pointer_type.rs',
    'storage_origin.rs',
    'type_var.rs',
]) {
    assertFile(moduleName);
}

for (const moduleDecl of [
    'mod address_projection;',
    'mod initialized;',
    'mod borrow_check;',
    'mod borrow_summary;',
    'mod borrow_state;',
    'mod cell_state;',
    'mod cell_state_raw_copy;',
    'mod cell_state_raw_range;',
    'mod cell_state_raw_range_append;',
    'mod cell_state_raw_range_cover;',
    'mod cell_state_raw_range_cover_tests;',
    'mod cell_state_raw_range_count;',
    'mod cell_state_raw_range_merge;',
    'mod cell_state_raw_range_model;',
    'mod cell_state_raw_range_copy;',
    'mod cell_state_raw_range_offset;',
    'mod cell_state_raw_range_value;',
    'mod cell_state_raw_range_value_alias;',
    'mod cell_state_raw_range_value_alias_tests;',
    'mod cell_state_tests;',
    'mod collection_slot_drop_traversal_certified;',
    'mod collection_slot_drop_proof;',
    'mod collection_slot_drop_traversal;',
    'mod collection_slot_drop_traversal_range;',
    'mod collection_slot_drop_traversal_summary;',
    'mod collection_slot_lifecycle;',
    'mod collection_slot_lifecycle_model;',
    'mod collection_slot_lifecycle_storage_tests;',
    'mod collection_slot_lifecycle_tests;',
    'mod collection_slot_lifecycle_transition;',
    'mod collection_slot_lifecycle_type_tests;',
    'mod collection_slot_owner_transfer;',
    'mod collection_slot_owner_transfer_proof;',
    'mod collection_slot_state_identity;',
    'mod collection_slot_summary_apply;',
    'mod collection_slot_summary_apply_return_path;',
    'mod collection_slot_summary_build;',
    'mod collection_slot_summary_build_drop_traversal;',
    'mod collection_slot_summary_build_event;',
    'mod collection_slot_summary_build_ops;',
    'mod collection_slot_summary_build_state;',
    'mod collection_slot_summary_event_apply_proof;',
    'mod collection_slot_summary_event_proof;',
    'mod collection_slot_summary_match_state;',
    'mod collection_slot_summary_model;',
    'mod collection_slot_summary_replay;',
    'mod collection_slot_summary_replay_drop_traversal;',
    'mod collection_slot_summary_return;',
    'mod collection_slot_summary_return_build;',
    'mod collection_slot_summary_return_call;',
    'mod collection_slot_summary_return_collect;',
    'mod collection_slot_summary_return_model;',
    'mod collection_slot_summary_return_path;',
    'mod collection_slot_summary_return_path_call;',
    'mod collection_slot_summary_return_path_control;',
    'mod collection_slot_summary_return_path_model;',
    'mod collection_slot_summary_return_path_slots;',
    'mod collection_slot_summary_return_path_state;',
    'mod collection_slot_summary_return_path_value;',
    'mod collection_slot_summary_return_state;',
    'mod collection_slot_summary_target;',
    'mod collection_slot_summary_translate;',
    'mod collection_slot_summary_translate_drop;',
    'mod collection_slot_summary_return_unique;',
    'mod collection_slot_summary_return_value;',
    'mod collection_slot_storage_release_proof;',
    'mod collection_slot_state_merge;',
    'mod collection_slot_state_merge_tests;',
    'mod collection_slot_state_release;',
    'mod collection_slot_state_release_tests;',
    'mod collection_slot_state_relocate;',
    'mod collection_slot_state_relocate_tests;',
    'mod collection_slot_state_return;',
    'mod collection_slot_state_table;',
    'mod collection_slot_state_table_tests;',
    'mod collection_slot_state_transfer;',
    'mod collection_slot_state_transfer_tests;',
    'mod owner_check;',
    'mod owner_check_utils;',
    'mod owner_consumption;',
    'mod owner_consumption_extent;',
    'mod owner_drop;',
    'mod owner_expr;',
    'mod owner_external_io;',
    'mod owner_host_direct_span;',
    'mod owner_host_memory_span;',
    'mod owner_host_memory_summary;',
    'mod owner_extent;',
    'mod owner_extent_check;',
    'mod owner_extent_compare;',
    'mod owner_extent_coverage;',
    'mod owner_extent_coverage_place;',
    'mod owner_extent_expected;',
    'mod owner_extent_summary;',
    'mod owner_external_io_payload;',
    'mod owner_flow;',
    'mod owner_host_dependent_span;',
    'mod owner_host_iov_descriptor;',
    'mod owner_host_payload_extent;',
    'mod owner_host_size_outputs;',
    'mod owner_match_payload;',
    'mod owner_raw_memory;',
    'mod owner_raw_memory_cell;',
    'mod owner_raw_memory_span;',
    'mod owner_raw_view;',
    'mod owner_raw_view_model;',
    'mod owner_raw_view_table;',
    'mod owner_summary;',
    'mod owner_summary_canonicalize;',
    'mod owner_summary_consumed;',
    'mod owner_summary_host_size_return;',
    'mod owner_summary_i32_condition_leaf;',
    'mod owner_summary_i32_leaf;',
    'mod owner_summary_owner_token_leaf;',
    'mod owner_summary_owner_token_type;',
    'mod owner_summary_parameters;',
    'mod owner_summary_raw_alias;',
    'mod owner_summary_raw_alias_branch;',
    'mod owner_summary_raw_alias_walk;',
    'mod owner_summary_raw_i32_leaf;',
    'mod owner_summary_type_size_return;',
    'mod owner_summary_type_params;',
    'mod owner_summary_update;',
    'mod owner_summary_variant_build;',
    'mod owner_summary_raw_consumption;',
    'mod owner_summary_raw_use;',
    'mod owner_summary_raw_use_branch;',
    'mod owner_summary_raw_use_call;',
    'mod owner_summary_raw_use_return;',
    'mod owner_summary_raw_use_walk;',
    'mod owner_summary_raw_view_return;',
    'mod owner_summary_variant_conditions;',
    'mod owner_summary_variant_construct;',
    'mod owner_summary_variant_i32_conditions;',
    'mod owner_summary_variant_leaf;',
    'mod owner_summary_variant_match;',
    'mod owner_summary_variant_path_conditions;',
    'mod owner_summary_variant_payload_conditions;',
    'mod owner_summary_variant_paths;',
    'mod owner_summary_variant_return;',
    'mod owner_summary_leaf;',
    'mod owner_summary_record;',
    'mod owner_summary_seed_leaf;',
    'mod owner_summary_size_return;',
    'mod owner_summary_storage_origin;',
    'mod owner_summary_variant_projection;',
    'mod owner_return;',
    'mod owner_return_apply;',
    'mod owner_return_apply_consumption;',
    'mod owner_return_unknown;',
    'mod owner_return_view;',
    'mod owner_variant;',
    'mod owner_variant_lifecycle;',
    'mod owner_variant_record;',
    'mod owner_variant_source_list;',
    'mod owner_variant_unreachable;',
    'mod owner_variant_utils;',
    'mod owner_variant_value_condition;',
    'mod result_variant;',
    'mod variant_name;',
    'mod summary;',
    'mod effect;',
    'mod effect_checked_memptr;',
    'mod effect_check;',
    'mod effect_counts;',
    'mod effect_counts_host;',
    'mod effect_counts_raw;',
    'mod effect_diagnostic;',
    'mod effect_summary;',
    'mod effect_summary_identity;',
    'mod effect_summary_identity_seed;',
    'mod effect_summary_identity_replay_tests;',
    'mod effect_summary_identity_tests;',
    'mod effect_summary_pointer;',
    'mod effect_summary_pointer_filter;',
    'mod effect_summary_pointer_seed;',
    'mod effect_summary_projection;',
    'mod effect_summary_seed;',
    'mod effect_summary_seed_alias;',
    'mod effect_summary_seed_walk;',
    'mod effect_identity;',
    'mod effect_match;',
    'mod effect_place_prefix;',
    'mod effect_pointer_alias;',
    'mod effect_raw_provenance;',
    'mod effect_raw_memory_identity;',
    'mod effect_return_escape;',
    'mod effect_return_escape_tests;',
    'mod effect_return_identity;',
    'mod effect_return_owner_type;',
    'mod effect_return_pointer;',
    'mod effect_return_protection;',
    'mod effect_return_summary_filter;',
    'mod coverage;',
    'mod coverage_hir;',
    'mod coverage_hir_projection;',
    'mod coverage_hir_projection_aggregate;',
    'mod coverage_hir_raw;',
    'mod coverage_hir_scope;',
    'mod coverage_hir_transparent;',
    'mod coverage_kind;',
    'mod coverage_operation;',
    'mod coverage_resource;',
    'mod coverage_resource_collection_slot;',
    'mod coverage_resource_place;',
    'mod dump;',
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
    'mod lower_collection_slot;',
    'mod lower_collection_slot_relocate_tests;',
    'mod lower_collection_slot_tests;',
    'mod lower_raw_address;',
    'mod lower_raw_address_offset;',
    'mod lower_raw_address_place;',
    'mod lower_raw_address_return;',
    'mod lower_raw_address_return_util;',
    'mod lower_raw_memory;',
    'mod lower_temporary_scope;',
    'mod lower_temporary_scope_op;',
    'mod report;',
    'mod shadow;',
    'mod initialized_alias;',
    'mod initialized_alias_difference;',
    'mod initialized_alias_difference_flow;',
    'mod initialized_alias_flow;',
    'mod initialized_alias_i32;',
    'mod initialized_alias_i32_bounds;',
    'mod initialized_alias_i32_condition;',
    'mod initialized_alias_i32_condition_context;',
    'mod initialized_alias_i32_condition_tests;',
    'mod initialized_alias_i32_facts;',
    'mod initialized_alias_offset;',
    'mod initialized_alias_i32_relation_condition;',
    'mod initialized_alias_origin;',
    'mod initialized_alias_origin_tests;',
    'mod initialized_alias_rank;',
    'mod initialized_alias_relation;',
    'mod initialized_alias_relation_flow;',
    'mod initialized_alias_relation_op;',
    'mod initialized_alias_scalar;',
    'mod initialized_alias_scalar_copy;',
    'mod initialized_alias_scale;',
    'mod initialized_alias_test_support;',
    'mod initialized_alias_utils;',
    'mod initialized_alias_tests;',
    'mod initialized_alias_raw_view_tests;',
    'mod initialized_collection_slot_dispatch;',
    'mod initialized_collection_slot_transfer;',
    'mod initialized_control;',
    'mod initialized_control_slot_transfer;',
    'mod initialized_availability;',
    'mod initialized_drop_assignment;',
    'mod initialized_drop_requirement;',
    'mod initialized_drop_scope;',
    'mod i32_call_facts;',
    'mod i32_call_facts_scale;',
    'mod i32_call_facts_scale_tests;',
    'mod i32_call_facts_tests;',
    'mod i32_extent_proof;',
    'mod initialized_external_io;',
    'mod external_io_iov_layout;',
    'mod host_dependent_length;',
    'mod host_memory_address;',
    'mod host_memory_contract;',
    'mod host_memory_contract_tests;',
    'mod host_size_contract;',
    'mod initialized_external_io_input;',
    'mod initialized_external_io_effect;',
    'mod initialized_external_io_iov;',
    'mod initialized_external_io_payload;',
    'mod initialized_host_dependent;',
    'mod initialized_path_state;',
    'mod initialized_raw_fill;',
    'mod initialized_raw_memory;',
    'mod initialized_raw_memory_access;',
    'mod initialized_raw_memory_bulk;',
    'mod raw_cell_lifecycle;',
    'mod raw_cell_value_flow;',
    'mod raw_cell_value_flow_alias;',
    'mod raw_cell_value_flow_cell;',
    'mod raw_cell_value_flow_proof;',
    'mod initialized_rekey;',
    'mod initialized_scalar_flow;',
    'mod initialized_scalar_flow_ops;',
    'mod initialized_str_layout;',
    'mod initialized_summary;',
    'mod initialized_alias_flow_apply;',
    'mod initialized_alias_flow_projection;',
    'mod initialized_alias_flow_raw;',
    'mod initialized_alias_flow_value_projection;',
    'mod initialized_alias_host_size;',
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
    'mod initialized_summary_release_build_tests;',
    'mod initialized_summary_release_model;',
    'mod initialized_summary_seed;',
    'mod initialized_summary_seed_tests;',
    'mod initialized_summary_return_byte_range_count;',
    'mod initialized_summary_return_byte_ranges;',
    'mod initialized_summary_variant_build;',
    'mod initialized_summary_variant_build_tests;',
    'mod initialized_summary_variant_condition;',
    'mod initialized_summary_variant_requirement;',
    'mod initialized_summary_variant_type;',
    'mod initialized_summary_variant_unique;',
    'mod initialized_variant;',
    'mod summary_dependency;',
    'mod summary_index;',
    'mod summary_worklist;',
    'mod summary_worklist_order;',
    'mod timing;',
    'mod compiler_memory_place;',
    'mod lower_call;',
    'mod lower_aggregate;',
    'mod lower_aggregate_projection;',
    'mod lower_aggregate_selector;',
    'mod lower_condition;',
    'mod lower_layout_intrinsic;',
    'mod model;',
    'mod scalar_primitive;',
    'mod owner_control;',
    'mod owner_drop_scope;',
    'mod owner_state;',
    'mod place_utils;',
    'mod raw_pointer_type;',
    'mod storage_origin;',
    'mod type_var;',
]) {
    assertContains(mod, moduleDecl, 'resource/mod.rs');
}

assertNotContains(mod, 'mod check;', 'resource/mod.rs');

const initialized = readResource('initialized.rs');
const borrowCheck = readResource('borrow_check.rs');
const borrowSummary = readResource('borrow_summary.rs');
const ownerCheck = readResource('owner_check.rs');
const ownerConsumption = readResource('owner_consumption.rs');
const ownerDrop = readResource('owner_drop.rs');
const ownerExpr = readResource('owner_expr.rs');
const ownerFlow = readResource('owner_flow.rs');
const ownerRawViewModel = readResource('owner_raw_view_model.rs');
const ownerSummary = readResource('owner_summary.rs');
const ownerSummaryRawTransfer = readResource('owner_summary_raw_transfer.rs');
const ownerSummaryRawViewReturn = readResource('owner_summary_raw_view_return.rs');
const ownerReturn = readResource('owner_return.rs');
const ownerReturnApply = readResource('owner_return_apply.rs');
const ownerReturnUnknown = readResource('owner_return_unknown.rs');
const ownerReturnView = readResource('owner_return_view.rs');
const compilerMemoryPlace = readResource('compiler_memory_place.rs');
const placeUtils = readResource('place_utils.rs');
const summary = readResource('summary.rs');
const effect = readResource('effect.rs');
const effectCheck = readResource('effect_check.rs');
const effectReturnEscape = readResource('effect_return_escape.rs');
const effectReturnSummaryFilter = readResource('effect_return_summary_filter.rs');
const effectReturnSummaryFilterTests = readResource('effect_return_summary_filter_tests.rs');
const effectSummary = readResource('effect_summary.rs');
const effectSummaryIdentity = readResource('effect_summary_identity.rs');
const rawPointerType = readResource('raw_pointer_type.rs');
const resourceDump = readResource('dump.rs');
const addressProjection = readResource('address_projection.rs');
const scalarPrimitive = readResource('scalar_primitive.rs');
const scalarPrimitives = readCoreSrc('scalar_primitives.rs');
const coverage = readResource('coverage.rs');
const coverageHir = readResource('coverage_hir.rs');
const coverageHirPlace = readResource('coverage_hir_place.rs');
const coverageHirProjection = readResource('coverage_hir_projection.rs');
const coverageHirProjectionAggregate = readResource('coverage_hir_projection_aggregate.rs');
const coverageHirRaw = readResource('coverage_hir_raw.rs');
const coverageHirScope = readResource('coverage_hir_scope.rs');
const coverageHirTransparent = readResource('coverage_hir_transparent.rs');
const coverageKind = readResource('coverage_kind.rs');
const coverageOperation = readResource('coverage_operation.rs');
const coverageResource = readResource('coverage_resource.rs');
const coverageResourcePlace = readResource('coverage_resource_place.rs');
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
const lowerCall = readResource('lower_call.rs');
const lowerAggregate = readResource('lower_aggregate.rs');
const lowerAggregateProjection = readResource('lower_aggregate_projection.rs');
const lowerAggregateSelector = readResource('lower_aggregate_selector.rs');
const lowerCondition = readResource('lower_condition.rs');
const lowerLayoutIntrinsic = readResource('lower_layout_intrinsic.rs');
const lowerRawAddress = readResource('lower_raw_address.rs');
const lowerRawAddressOffset = readResource('lower_raw_address_offset.rs');
const lowerRawAddressPlace = readResource('lower_raw_address_place.rs');
const lowerRawAddressReturn = readResource('lower_raw_address_return.rs');
const lowerRawAddressReturnUtil = readResource('lower_raw_address_return_util.rs');
const lowerTests = readResource('lower_tests.rs');
const resultVariant = readResource('result_variant.rs');
const lowerRawMemory = readResource('lower_raw_memory.rs');
const lowerTemporaryScope = readResource('lower_temporary_scope.rs');
const initializedDropRequirement = readResource('initialized_drop_requirement.rs');
const initializedAliasOrigin = readResource('initialized_alias_origin.rs');
const initializedAliasOriginTests = readResource('initialized_alias_origin_tests.rs');
const initializedAliasRelation = readResource('initialized_alias_relation.rs');
const initializedAliasRelationFlow = readResource('initialized_alias_relation_flow.rs');
const initializedAliasRelationOp = readResource('initialized_alias_relation_op.rs');
const initializedAliasScalar = readResource('initialized_alias_scalar.rs');
const initializedAliasOffset = readResource('initialized_alias_offset.rs');
const initializedScalarFlow = readResource('initialized_scalar_flow.rs');
const initializedScalarFlowOps = readResource('initialized_scalar_flow_ops.rs');
const initializedStrLayout = readResource('initialized_str_layout.rs');
const i32CallFacts = readResource('i32_call_facts.rs');
const initializedAliasFlowValueProjection = readResource(
    'initialized_alias_flow_value_projection.rs',
);
const ownerSummaryRawI32Leaf = readResource('owner_summary_raw_i32_leaf.rs');
const ownerSummaryOwnerTokenLeaf = readResource('owner_summary_owner_token_leaf.rs');

assertContains(initialized, 'struct ResourceCheckEngine', 'initialized.rs');
assertContains(borrowCheck, 'struct ResourceBorrowCheckEngine', 'borrow_check.rs');
assertContains(ownerCheck, 'struct ResourceOwnerCheckEngine', 'owner_check.rs');
assertContains(effectCheck, 'struct ResourceEffectBoundaryEngine', 'effect_check.rs');
assertContains(
    ownerSummaryOwnerTokenLeaf,
    'compiler_memory_type_field_index(',
    'owner_summary_owner_token_leaf.rs must derive owner-token raw identity from shared compiler memory field specs',
);
assertContains(
    ownerSummaryOwnerTokenLeaf,
    'CompilerMemoryType::OwnerToken',
    'owner_summary_owner_token_leaf.rs must use the owner-token memory type proof for raw owner leaves',
);
assertContains(
    ownerSummaryOwnerTokenLeaf,
    'CompilerMemoryFieldSpec::RawI32',
    'owner_summary_owner_token_leaf.rs must use the typed raw field spec for raw owner leaves',
);
assertContains(
    ownerSummaryOwnerTokenLeaf,
    'type_is_owner_token(types, ty)',
    'owner_summary_owner_token_leaf.rs must reject same-shape structs without owner-token identity proof',
);
assertNotContains(
    ownerSummaryOwnerTokenLeaf,
    'field_name == "raw"',
    'owner_summary_owner_token_leaf.rs must not duplicate compiler memory field spelling',
);
assertContains(
    ownerSummaryRawI32Leaf,
    'RawI32OwnerLeafMode::OwnerTokenOnly',
    'owner_summary_raw_i32_leaf.rs must keep owner-token-backed metadata out of raw owner candidates',
);

assertNotContains(effect, 'struct ResourceEffectBoundaryEngine', 'effect.rs');
assertContains(effect, 'pub fn check_resource_effect_boundaries', 'effect.rs');
assertNotContains(resourceDump, 'fn dump_raw_memory_op', 'dump.rs');
assertContains(
    lowerRawMemory,
    'pub(super) fn raw_memory_call_uses_direct_raw_address',
    'lower_raw_memory.rs',
);
assertContains(
    lowerRawMemory,
    'raw_memory_intrinsic_op_from_name(name)',
    'lower_raw_memory.rs must consume the typed raw-memory intrinsic classifier directly',
);
assertNotContains(
    lowerRawMemory,
    'intrinsic_is_raw_memory_effect',
    'lower_raw_memory.rs must not combine a boolean intrinsic gate with helper-name reclassification',
);
assertContains(lowerCall, 'pub(super) fn call_effect_skeleton', 'lower_call.rs');
assertContains(lowerCall, 'pub(super) fn lower_call_target', 'lower_call.rs');
assertContains(lowerCall, 'pub(super) fn func_ref_base_name', 'lower_call.rs');
assertNotContains(lower, 'fn call_effect_skeleton', 'lower.rs');
assertNotContains(lower, 'fn lower_call_target', 'lower.rs');
assertNotContains(lower, 'fn should_lower_raw_memory_call', 'lower.rs');
assertNotContains(
    lower,
    'FieldAccessorKind::from_call_base_name',
    'lower.rs must not classify field access from ordinary direct call spelling',
);
assertContains(
    lowerTests,
    'ordinary_get_direct_call_is_not_field_projection',
    'lower_tests.rs must keep the regression that ordinary get calls remain ordinary calls',
);
assertContains(
    lowerTests,
    'transparent_raw_address_return_ignores_ordinary_get_call',
    'lower_tests.rs must keep the regression that transparent raw-address proof ignores ordinary get calls',
);
assertNotContains(
    lower,
    'Some("get") | Some("get_ref") | Some("get_field") | Some("get_field_ref")',
    'lower.rs must not duplicate field accessor spelling for recursive lowering',
);
assertContains(
    initializedDropRequirement,
    'pub(super) fn partial_drop_requirement_for_initialized_descendants',
    'initialized_drop_requirement.rs',
);
assertNotContains(
    readResource('initialized_drop_scope.rs'),
    'fn partial_drop_requirement_inner',
    'initialized_drop_scope.rs',
);
assertNotContains(coverageHirRaw, 'is_named_struct_type', 'coverage_hir_raw.rs');
assertContains(addressProjection, 'enum AddressProjectionPrimitive', 'address_projection.rs');
assertContains(addressProjection, 'I32ArithmeticPrimitive::from_symbol', 'address_projection.rs');
assertContains(
    addressProjection,
    'I32ArithmeticPrimitive::from_base_name',
    'address_projection.rs',
);
assertContains(
    addressProjection,
    'pub(super) fn compiler_field_address_base_and_offset',
    'address_projection.rs',
);
assertContains(
    addressProjection,
    'pub(super) fn non_negative_i32_literal',
    'address_projection.rs',
);
assertContains(
    addressProjection,
    'pub(super) fn intrinsic_is_address_projection',
    'address_projection.rs',
);
assertContains(
    addressProjection,
    'pub(super) fn storage_offset_base_and_offset',
    'address_projection.rs',
);
assertContains(
    scalarPrimitive,
    'pub(super) use crate::scalar_primitives',
    'scalar_primitive.rs',
);
assertContains(
    scalarPrimitives,
    'pub(crate) enum I32ArithmeticPrimitive',
    'scalar_primitives.rs',
);
assertContains(
    scalarPrimitives,
    'pub(crate) enum I32ComparisonPrimitive',
    'scalar_primitives.rs',
);
assertContains(
    scalarPrimitives,
    'pub(crate) enum BooleanPrimitive',
    'scalar_primitives.rs',
);
assertContains(
    scalarPrimitive,
    'pub(super) fn from_resource_call_target',
    'scalar_primitive.rs',
);
assertContains(
    scalarPrimitive,
    'pub(super) const fn relation_op',
    'scalar_primitive.rs',
);
assertContains(
    lowerLayoutIntrinsic,
    'CoreIntrinsicKind::from_intrinsic_name',
    'lower_layout_intrinsic.rs must use shared core intrinsic classification',
);
assertContains(
    lowerLayoutIntrinsic,
    'kind.layout_i32_value',
    'lower_layout_intrinsic.rs must evaluate layout constants through CoreIntrinsicKind',
);
assertNotContains(
    lowerLayoutIntrinsic,
    '"size_of" =>',
    'lower_layout_intrinsic.rs must not duplicate size_of spelling outside CoreIntrinsicKind',
);
assertNotContains(
    lowerLayoutIntrinsic,
    '"align_of" =>',
    'lower_layout_intrinsic.rs must not duplicate align_of spelling outside CoreIntrinsicKind',
);
assertContains(coverage, 'pub fn compare_hir_resource_lowering_typed', 'coverage.rs');
assertContains(coverageHir, 'pub(super) fn hir_function_coverage', 'coverage_hir.rs');
assertContains(
    coverageHirPlace,
    'intrinsic_is_address_projection(name)',
    'coverage_hir_place.rs must use shared address projection classification',
);
assertNotContains(
    coverageHirPlace,
    'name == "add"',
    'coverage_hir_place.rs must not classify address add locally',
);
assertContains(
    coverageHirProjection,
    'pub(super) fn get_field_intrinsic_owner',
    'coverage_hir_projection.rs must classify get_field coverage from intrinsic evidence only',
);
assertContains(
    coverageHirProjection,
    'pub(super) fn get_field_ref_intrinsic_owner',
    'coverage_hir_projection.rs must classify get_field_ref coverage from intrinsic evidence only',
);
assertNotContains(
    coverageHirProjection,
    'FieldAccessorKind::from_core_field_member_name',
    'coverage_hir_projection.rs must not classify coverage from ordinary core/field method names',
);
assertContains(
    coverageHirProjection,
    'raw_memory_op_from_intrinsic(name),\n                Some(RawMemoryOp::Load | RawMemoryOp::LoadU8)',
    'coverage_hir_projection.rs raw load coverage classifier',
);
assertContains(
    coverageHirProjection,
    'raw_memory_op_from_callee(callee),\n                Some(RawMemoryOp::Load | RawMemoryOp::LoadU8)',
    'coverage_hir_projection.rs raw load coverage classifier',
);
assertContains(
    coverageHirProjection,
    'super::address_projection::{',
    'coverage_hir_projection.rs must share field address projection classifier with lowering',
);
assertContains(
    coverageHirProjection,
    'compiler_field_address_base_and_offset, AddressProjectionPrimitive',
    'coverage_hir_projection.rs must share field address projection classifier with lowering',
);
assertContains(
    coverageHirProjection,
    'AddressProjectionPrimitive::from_base_name',
    'coverage_hir_projection.rs must use typed address projection classification',
);
assertNotContains(
    coverageHirProjection,
    'Some("add" | "sub")',
    'coverage_hir_projection.rs must not classify reference address arithmetic locally',
);
assertNotContains(
    coverageHirProjection,
    'matches!(name, Some("add"))',
    'coverage_hir_projection.rs must not classify reference field arithmetic locally',
);
assertNotContains(
    coverageHirProjection,
    'name == "load"',
    'coverage_hir_projection.rs must not classify raw load coverage by literal helper spelling',
);
assertNotContains(
    coverageHirProjection,
    'starts_with("load_")',
    'coverage_hir_projection.rs must not classify raw load coverage by helper name prefix',
);
assertContains(
    coverageHirProjectionAggregate,
    'pub(super) fn aggregate_field_exists',
    'coverage_hir_projection_aggregate.rs',
);
assertNotContains(
    coverageHirProjectionAggregate,
    'name == "add"',
    'coverage_hir_projection_aggregate.rs must not classify field address add locally',
);
assertNotContains(
    coverageHirProjectionAggregate,
    'callee_base_name',
    'coverage_hir_projection_aggregate.rs must not reimplement call-head address projection classification',
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
    coverageHirTransparent,
    'transparent_raw_address_return_deref_projection_count',
    'coverage_hir_transparent.rs',
);
assertContains(
    coverageHirTransparent,
    'TRANSPARENT_RAW_ADDRESS_COVERAGE_DEPTH_LIMIT',
    'coverage_hir_transparent.rs',
);
assertContains(coverageKind, 'pub enum ResourceCoverageKind', 'coverage_kind.rs');
assertContains(
    coverageOperation,
    'pub enum ResourceCoveragePlaceOperation',
    'coverage_operation.rs',
);
assertContains(
    coverageOperation,
    'pub fn as_str(self)',
    'coverage_operation.rs',
);
assertContains(
    coverageResource,
    'pub(super) fn resource_function_coverage',
    'coverage_resource.rs',
);
assertContains(
    coverageResourcePlace,
    'pub(super) fn resource_place_coverage',
    'coverage_resource_place.rs',
);
assertContains(
    coverageResourcePlace,
    'pub(super) fn resource_alias_place_coverage',
    'coverage_resource_place.rs',
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
assertMatches(
    lowerRawAddress,
    /Some\(MemoryHelperPrimitive::MemPtrAddr\)\s*=>\s*\{[\s\S]*push_raw_address_op\([\s\S]*Some\(RawAddressViewKind::NonOwningProjection\)[\s\S]*\}\s*Some\(MemoryHelperPrimitive::MemPtrAdd\)/,
    'lower_raw_address.rs mem_ptr_addr non-owning projection policy',
);
assertMatches(
    lowerRawAddress,
    /Some\(MemoryHelperPrimitive::MemPtrAdd\)\s*=>\s*\{[\s\S]*RawAddressViewKind::MemPtrOffset[\s\S]*push_raw_address_op/,
    'lower_raw_address.rs mem_ptr_add offset boundary policy',
);
assertContains(
    lowerRawAddress,
    'ResultVariant::Ok.payload_place',
    'lower_raw_address.rs must consume typed Result success projection',
);
assertNotContains(
    lowerRawAddress,
    'enum_payload_type(env.types, output.ty, "Ok")',
    'lower_raw_address.rs must not hardcode Result success payload spelling',
);
assertNotContains(
    lowerRawAddress,
    'variant: String::from("Ok")',
    'lower_raw_address.rs must not construct Result success payload projection by raw string',
);
assertContains(
    resultVariant,
    'pub(super) enum ResultVariant',
    'result_variant.rs must own Resource IR Result payload variant spelling',
);
assertContains(
    resultVariant,
    'pub(super) fn payload_place',
    'result_variant.rs must own Result payload projection construction',
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
assertMatches(
    lowerRawAddressReturn,
    /fn function_has_dedicated_raw_address_lowering[\s\S]*MemoryHelperPrimitive::from_symbol\(name\)[\s\S]*MemoryHelperPrimitive::has_dedicated_raw_address_lowering/,
    'lower_raw_address_return.rs dedicated wrapper policy',
);
assertContains(
    lowerRawAddressReturnUtil,
    'pub(super) fn raw_address_offset_from_return_expr',
    'lower_raw_address_return_util.rs',
);
assertContains(
    lowerRawMemory,
    'pub(super) fn raw_memory_op_from_name',
    'lower_raw_memory.rs',
);
assertNotContains(
    initializedAliasFlowValueProjection,
    'type_is_result_enum',
    'initialized_alias_flow_value_projection.rs',
);
assertNotContains(
    initializedAliasFlowValueProjection,
    'name == "Result"',
    'initialized_alias_flow_value_projection.rs',
);
assertContains(
    typeCtxSource,
    'compiler_memory_types: Vec<(TypeId, CompilerMemoryType)>',
    'types.rs compiler memory type identity store',
);
assertContains(
    typeCtxSource,
    'pub fn mark_compiler_memory_type',
    'types.rs compiler memory type proof registration',
);
assertContains(
    typeCtxSource,
    'pub fn compiler_memory_type',
    'types.rs compiler memory type proof query',
);
assertContains(
    typecheckDriver,
    'ctx.mark_compiler_memory_type(ty, memory_type)',
    'typecheck driver must attach proven compiler memory type identity to TypeCtx',
);
assertContains(
    resourcePrimitivesCompilerMemory,
    'TypeKind::Struct { .. } => types.compiler_memory_type(resolved)',
    'resource_primitives/compiler_memory.rs must query proven TypeCtx identity for compiler memory structs',
);
assertContains(
    resourcePrimitivesCompilerMemory,
    'types.compiler_memory_type(base)',
    'resource_primitives/compiler_memory.rs must preserve proven compiler memory identity through type application',
);
assertNotContains(
    lowerRawAddressReturn,
    'compiler_memory_type_from_constructor_name',
    'lower_raw_address_return.rs must not classify compiler memory constructs by constructor name',
);
assertContains(
    lowerRawAddressReturn,
    'type_is_raw_pointer(env.types, expr.ty)',
    'lower_raw_address_return.rs must query proven TypeCtx identity for constructed raw pointer returns',
);
assertNotContains(
    ownerFlow,
    'compiler_memory_type_from_constructor_name',
    'owner_flow.rs must not classify compiler memory constructs by constructor name',
);
assertContains(
    ownerFlow,
    'type_is_owner_token(self.types, output.ty)',
    'owner_flow.rs must query proven TypeCtx identity for owner-token construct extent handling',
);
assertNotContains(
    resourcePrimitivesCompilerMemory,
    'TypeKind::Struct { name, .. } => compiler_memory_type_from_constructor_name(name)',
    'resource_primitives/compiler_memory.rs must not infer compiler memory type identity from struct names',
);
assertContains(
    lowerTemporaryScope,
    'pub(super) fn push_line_copy_state_only_temporary_scope',
    'lower_temporary_scope.rs',
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
    initializedAliasOffset,
    'pub(super) struct I32OffsetFact',
    'initialized_alias_offset.rs',
);
assertContains(
    initializedAliasOffset,
    'pub(super) struct I32OffsetFacts',
    'initialized_alias_offset.rs',
);
assertContains(
    initializedAliasOffset,
    'pub(super) fn facts_with_replaced_prefix',
    'initialized_alias_offset.rs',
);
assertContains(
    initializedAliasOffset,
    'pub(super) fn merge_paths',
    'initialized_alias_offset.rs',
);
assertContains(
    initializedAliasOrigin,
    'pub(super) struct RawValueOrigins',
    'initialized_alias_origin.rs',
);
assertContains(
    initializedAliasOriginTests,
    'copy_stable_origin_follows_temporary_source_origin',
    'initialized_alias_origin_tests.rs',
);
assertContains(
    initializedScalarFlow,
    'pub(super) fn compute_i32_scalar_return_summaries',
    'initialized_scalar_flow.rs',
);
assertContains(
    initializedScalarFlow,
    'pub(super) fn apply_direct_call_i32_scalar_summary',
    'initialized_scalar_flow.rs',
);
assertContains(
    initializedScalarFlowOps,
    'pub(super) fn propagate_i32_scalar_ops',
    'initialized_scalar_flow_ops.rs',
);
assertContains(
    initializedStrLayout,
    'pub(super) fn seed_str_storage_layout',
    'initialized_str_layout.rs',
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
    lowerAggregate,
    'super::address_projection::{',
    'lower_aggregate.rs must use the shared address projection classifier',
);
assertContains(
    lowerAggregate,
    'AddressProjectionPrimitive::from_symbol',
    'lower_aggregate.rs must use typed address projection classification',
);
assertNotContains(
    lowerAggregate,
    'helper_base_name(name) != "add"',
    'lower_aggregate.rs must not classify reference field arithmetic locally',
);
assertContains(
    lower,
    'storage_offset_base_and_offset(expr)',
    'lower.rs place skeleton must use the shared address projection classifier',
);
assertNotContains(
    lower,
    'name == "add"',
    'lower.rs must not classify address add locally',
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
assertContains(
    lowerAggregateSelector,
    'super::address_projection::non_negative_i32_literal',
    'lower_aggregate_selector.rs must share selector literal handling',
);
assertNotContains(
    lowerAggregateSelector,
    'name == "add"',
    'lower_aggregate_selector.rs must not classify field address add locally',
);
assertNotContains(
    lowerAggregateSelector,
    'callee_base_name',
    'lower_aggregate_selector.rs must not reimplement call-head address projection classification',
);
assertContains(
    lowerRawAddress,
    'I32ArithmeticPrimitive::from_symbol',
    'lower_raw_address.rs must use typed scalar arithmetic classification',
);
assertContains(
    lowerRawAddressReturn,
    'I32ArithmeticPrimitive::from_symbol',
    'lower_raw_address_return.rs must use typed scalar arithmetic classification',
);
assertContains(
    lowerRawAddressReturnUtil,
    'I32ArithmeticPrimitive::from_symbol',
    'lower_raw_address_return_util.rs must use typed scalar arithmetic classification',
);
assertContains(
    lowerRawAddress,
    'layout_intrinsic_i64_value_from_callee',
    'lower_raw_address.rs must evaluate call layout constants through shared core intrinsic classification',
);
assertContains(
    lowerRawAddress,
    'layout_intrinsic_i64_value(name, type_args, env)',
    'lower_raw_address.rs must evaluate intrinsic layout constants through shared core intrinsic classification',
);
assertContains(
    lowerRawAddressReturnUtil,
    'layout_intrinsic_i64_value_from_callee',
    'lower_raw_address_return_util.rs must evaluate return call layout constants through shared core intrinsic classification',
);
assertContains(
    lowerRawAddressReturnUtil,
    'layout_intrinsic_i64_value(name, type_args, env)',
    'lower_raw_address_return_util.rs must evaluate return intrinsic layout constants through shared core intrinsic classification',
);
assertNotContains(
    lowerRawAddress,
    'match helper_base_name(name) {',
    'lower_raw_address.rs must not classify scalar arithmetic by local helper-base string match',
);
assertNotContains(
    lowerRawAddressReturn,
    'match helper_base_name(name) {',
    'lower_raw_address_return.rs must not classify scalar arithmetic by local helper-base string match',
);
assertNotContains(
    lowerRawAddressReturnUtil,
    'match helper_base_name(name) {',
    'lower_raw_address_return_util.rs must not classify scalar arithmetic by local helper-base string match',
);
assertNotContains(
    lowerRawAddress,
    'helper_base_name(name) == "size_of"',
    'lower_raw_address.rs must not duplicate size_of spelling outside CoreIntrinsicKind',
);
assertNotContains(
    lowerRawAddressReturnUtil,
    'helper_base_name(name) == "size_of"',
    'lower_raw_address_return_util.rs must not duplicate size_of spelling outside CoreIntrinsicKind',
);
assertContains(
    i32CallFacts,
    'I32ArithmeticPrimitive::from_resource_call_target',
    'i32_call_facts.rs must use typed scalar arithmetic classification',
);
assertNotContains(
    i32CallFacts,
    'resource_call_target_base_name',
    'i32_call_facts.rs must not classify scalar facts by local call-target strings',
);
assertContains(
    lowerCondition,
    'BooleanPrimitive::from_base_name',
    'lower_condition.rs must use typed boolean primitive classification',
);
assertContains(
    lowerCondition,
    'I32ComparisonPrimitive::from_base_name',
    'lower_condition.rs must use typed comparison primitive classification',
);
for (const conditionHelperBranch of ['"or" =>', '"and" =>', '"eq" =>', '"ne" =>', '"lt" =>']) {
    assertNotContains(
        lowerCondition,
        conditionHelperBranch,
        'lower_condition.rs must not classify condition helpers by local string match',
    );
}
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
    ownerDrop,
    'owner_check',
    'ResourceOwnerCheckEngine',
    'owner_drop.rs',
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
assertUsesResourceModuleSymbol(
    ownerExpr,
    'owner_check',
    'ResourceOwnerCheckEngine',
    'owner_expr.rs',
);
assertContains(ownerExpr, 'fn check_expr', 'owner_expr.rs');
assertNotContains(ownerCheck, 'fn check_expr', 'owner_check.rs');
assertContains(
    ownerDrop,
    'fn auto_drop_scope_owner_obligations',
    'owner_drop.rs',
);
assertContains(ownerDrop, 'fn drop_owner_obligation', 'owner_drop.rs');
assertContains(
    ownerRawViewModel,
    'enum RawAddressViewOwnership',
    'owner_raw_view_model.rs',
);
const rawOwnerAliasPolicy = braceBlock(
    ownerSummaryRawTransfer,
    'fn raw_owner_alias_transfer_kind',
    'owner_summary_raw_transfer.rs',
);
assertContains(
    ownerSummaryRawTransfer,
    'enum RawOwnerAliasTransferKind',
    'owner_summary_raw_transfer.rs',
);
const rawOwnerAliasTransfer = braceBlock(
    ownerSummaryRawTransfer,
    'pub(super) fn push_transferred_raw_owner_view_aliases',
    'owner_summary_raw_transfer.rs',
);
assertContains(
    rawOwnerAliasTransfer,
    'match raw_owner_alias_transfer_kind(raw_views, source, kind) {',
    'owner_summary_raw_transfer.rs raw owner alias policy',
);
assertContains(
    rawOwnerAliasTransfer,
    'RawOwnerAliasTransferKind::NonOwningProjection => {',
    'owner_summary_raw_transfer.rs raw owner alias policy',
);
assertContains(
    rawOwnerAliasTransfer,
    'raw_views.mark_non_owning_projection(target);',
    'owner_summary_raw_transfer.rs raw owner alias policy',
);
assertContains(
    rawOwnerAliasTransfer,
    'RawOwnerAliasTransferKind::NonOwning => {',
    'owner_summary_raw_transfer.rs raw owner alias policy',
);
assertContains(
    rawOwnerAliasTransfer,
    'raw_views.mark_non_owning(target);',
    'owner_summary_raw_transfer.rs raw owner alias policy',
);
assertContains(
    rawOwnerAliasTransfer,
    'RawOwnerAliasTransferKind::OwnerAlias => {',
    'owner_summary_raw_transfer.rs raw owner alias policy',
);
assertContains(
    rawOwnerAliasTransfer,
    'push_transferred_aliases(aliases, source, target)',
    'owner_summary_raw_transfer.rs raw owner alias policy',
);
assertContains(
    rawOwnerAliasPolicy,
    'RawAddressViewKind::NonOwningProjection | RawAddressViewKind::InternalHelper => {\n            RawOwnerAliasTransferKind::NonOwningProjection',
    'owner_summary_raw_transfer.rs raw owner alias policy',
);
assertMatches(
    rawOwnerAliasPolicy,
    /RawAddressViewKind::Offset\s*\|\s*RawAddressViewKind::MemPtrOffset\s+if raw_views\.contains_non_owning_projection\(source\) =>/,
    'owner_summary_raw_transfer.rs raw owner alias policy',
);
assertMatches(
    rawOwnerAliasPolicy,
    /RawAddressViewKind::Offset\s*\|\s*RawAddressViewKind::MemPtrOffset\s+if raw_views\.contains_non_owning\(source\) =>/,
    'owner_summary_raw_transfer.rs raw owner alias policy',
);
assertContains(
    rawOwnerAliasPolicy,
    'RawAddressViewKind::Offset | RawAddressViewKind::MemPtrOffset => {',
    'owner_summary_raw_transfer.rs raw owner alias policy',
);
assertNotContains(
    rawOwnerAliasPolicy,
    '_ =>',
    'owner_summary_raw_transfer.rs raw owner alias policy',
);
const rawViewReturnKindPolicy = braceBlock(
    ownerSummaryRawViewReturn,
    'fn non_owning_raw_view_return_kind',
    'owner_summary_raw_view_return.rs',
);
assertContains(
    rawViewReturnKindPolicy,
    'RawAddressViewOwnership::NonOwning => OwnerNonOwningRawViewKind::AliasView',
    'owner_summary_raw_view_return.rs raw view return policy',
);
assertContains(
    rawViewReturnKindPolicy,
    'RawAddressViewOwnership::NonOwningProjection => OwnerNonOwningRawViewKind::ProjectionView',
    'owner_summary_raw_view_return.rs raw view return policy',
);
assertContains(
    rawViewReturnKindPolicy,
    'RawAddressViewOwnership::AddressView => OwnerNonOwningRawViewKind::AliasView',
    'owner_summary_raw_view_return.rs raw view return policy',
);
assertNotContains(
    rawViewReturnKindPolicy,
    '_ =>',
    'owner_summary_raw_view_return.rs raw view return policy',
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
    effectSummaryIdentity,
    'effect_check',
    'ResourceEffectBoundaryEngine',
    'effect_summary_identity.rs',
);
assertUsesResourceModuleSymbol(
    effectReturnEscape,
    'effect_return_protection',
    'raw_identity_projection_has_owner_protection',
    'effect_return_escape.rs',
);
assertUsesResourceModuleSymbol(
    effectSummaryIdentity,
    'effect_return_summary_filter',
    'raw_identity_return_projection_requires_summary',
    'effect_summary_identity.rs',
);
assertContains(
    effectReturnSummaryFilter,
    'pub(super) fn raw_identity_return_projection_requires_summary',
    'effect_return_summary_filter.rs',
);
assertContains(
    effectReturnSummaryFilter,
    'fn raw_identity_projection_has_summary_owner_carrier_protection',
    'effect_return_summary_filter.rs must keep internal provenance summary filtering separate from public escape filtering',
);
assertNotContains(
    effectReturnSummaryFilter,
    'raw_identity_projection_has_owner_protection',
    'effect_return_summary_filter.rs must not re-couple internal provenance summaries to public escape filtering',
);
assertContains(
    effectReturnSummaryFilter,
    'resolved == types.str()\n        || (!type_is_owner_token(types, resolved)\n            && !raw_identity_type_is_enum_like(types, resolved)\n            && raw_identity_type_is_structural_owner_carrier(types, resolved))',
    'effect_return_summary_filter.rs must keep enum payload owner provenance while suppressing opaque str and structural owner carriers',
);
assertContains(
    effectReturnSummaryFilter,
    'raw_identity_type_is_structural_owner_carrier',
    'effect_return_summary_filter.rs must suppress structural owner carriers without stdlib allowlists',
);
assertContains(
    effectReturnSummaryFilterTests,
    'summary_filter_keeps_direct_owner_token_internal_provenance',
    'effect_return_summary_filter.rs must keep direct owner-token internal provenance summaries',
);
assertContains(
    effectReturnSummaryFilterTests,
    'summary_filter_hides_owner_token_inside_aggregate',
    'effect_return_summary_filter.rs must hide structural owner-carrier summaries even when the aggregate has plain i32 fields',
);
assertContains(
    rawPointerType,
    'raw_identity_type_is_structural_owner_carrier',
    'raw_pointer_type.rs must suppress structural owner carriers without stdlib allowlists',
);
assertContains(
    readResource('raw_pointer_owner_carrier_tests.rs'),
    'summary_carrier_excludes_owner_token_carriers',
    'raw_pointer_owner_carrier_tests.rs must cover owner-token carriers with plain i32 metadata fields',
);
assertContains(
    readResource('raw_pointer_owner_carrier_tests.rs'),
    'summary_carrier_excludes_enum_backed_owner_storage_carriers',
    'raw_pointer_owner_carrier_tests.rs must cover enum-backed owner storage carriers',
);
assertContains(
    effectSummaryIdentity,
    'raw_identity_return_projection_requires_summary(types, place, &suffix, ty)',
    'effect_summary_identity.rs must filter protected raw identities before recording summaries',
);
assertContains(
    effect,
    'compute_raw_identity_return_summaries(module, &pointer_summaries, types)',
    'effect.rs must pass TypeCtx into raw identity summary construction',
);
assertUsesResourceModuleSymbol(
    effectSummaryIdentity,
    'summary_worklist',
    'SummaryWorklist',
    'effect_summary_identity.rs',
);
assertUsesResourceModuleSymbol(
    readResource('effect_summary_pointer.rs'),
    'summary_worklist',
    'SummaryWorklist',
    'effect_summary_pointer.rs',
);
assertNotContains(
    effectSummaryIdentity,
    'for _ in 0..=module.functions.len()',
    'effect_summary_identity.rs must not reintroduce full-module fixed-point sweeps',
);
assertNotContains(
    readResource('effect_summary_pointer.rs'),
    'for _ in 0..=module.functions.len()',
    'effect_summary_pointer.rs must not reintroduce full-module fixed-point sweeps',
);
assertNotContains(
    lowerRawAddressReturn,
    'FieldAccessorKind::from_call_base_name',
    'lower_raw_address_return.rs must not classify transparent field projection proof from ordinary callee spelling',
);
assertContains(
    lowerRawAddressReturn,
    'enum RawAddressReturnCalleeEvidence',
    'lower_raw_address_return.rs must keep explicit callee evidence for transparent raw-address proof',
);
assertContains(
    lowerRawAddressReturn,
    'Self::OrdinaryCall => None',
    'lower_raw_address_return.rs must reject ordinary direct calls as field accessor proof evidence',
);
assertContains(
    lowerRawAddressReturn,
    'Self::Intrinsic => FieldAccessorKind::from_intrinsic_name(helper_base_name(name))',
    'lower_raw_address_return.rs must restrict field accessor proof evidence to typed intrinsic classification',
);
assertContains(
    lowerRawAddressOffset,
    'pub(super) enum RawAddressOffset',
    'lower_raw_address_offset.rs',
);
assertContains(
    lowerRawAddressOffset,
    'pub(super) fn symbolic',
    'lower_raw_address_offset.rs',
);
assertContains(
    lowerRawAddressReturn,
    'CompilerMemoryFieldSpec::RawI32.name()',
    'lower_raw_address_return.rs must derive raw field spelling from CompilerMemoryFieldSpec',
);
assertNotContains(
    lowerRawAddressReturn,
    'matches!(base_name, "get" | "get_field")',
    'lower_raw_address_return.rs must not duplicate field accessor spelling',
);
assertNotContains(
    lowerRawAddressReturn,
    'field_name == "raw"',
    'lower_raw_address_return.rs must not duplicate raw field spelling',
);
assertNotContains(
    lowerRawAddressReturn,
    'Some("raw")',
    'lower_raw_address_return.rs must not duplicate raw field spelling',
);
assertUsesResourceModuleSymbol(
    placeUtils,
    'variant_name',
    'normalize_variant_name',
    'place_utils.rs',
);
assertUsesResourceModuleSymbol(
    placeUtils,
    'variant_name',
    'match_pattern_variant_name',
    'place_utils.rs',
);
assertUsesResourceModuleSymbol(
    placeUtils,
    'variant_name',
    'variant_names_match',
    'place_utils.rs',
);
assertContains(
    readResource('variant_name.rs'),
    'crate::qualified_name::member_tail',
    'variant_name.rs',
);
assertNotContains(
    placeUtils,
    'fn canonical_variant_name',
    'place_utils.rs',
);
for (const resourceFileName of fs.readdirSync(RESOURCE_DIR)) {
    if (!resourceFileName.endsWith('.rs') || resourceFileName === 'variant_name.rs') {
        continue;
    }
    assertNotContains(
        readResource(resourceFileName),
        'rsplit("::")',
        resourceFileName,
    );
}
assertContains(
    compilerMemoryPlace,
    'compiler_memory_type_field_index(memory_type, field)',
    'compiler_memory_place.rs must derive compiler memory field index from shared field specs',
);
assertContains(
    compilerMemoryPlace,
    'compiler_memory_type_field_offset_bytes(memory_type, field)',
    'compiler_memory_place.rs must derive compiler memory field offsets from shared field specs',
);
assertContains(
    compilerMemoryPlace,
    'type_is_compiler_memory_type(types, place.ty, memory_type)',
    'compiler_memory_place.rs must require TypeCtx compiler memory identity before building field places',
);
assertNotContains(
    readResource('lower_raw_address_place.rs'),
    'PlaceProjection::Field {\n            index: 0,\n            offset_bytes: 0,',
    'lower_raw_address_place.rs must not hand-code compiler memory field projections',
);
assertNotContains(
    readResource('initialized_summary_indirect_release.rs'),
    'PlaceProjection::Field {\n            index: 0,\n            offset_bytes: 0,',
    'initialized_summary_indirect_release.rs must not duplicate compiler memory field projections',
);

const maxLines = new Map([
    ['address_projection.rs', 80],
    ['effect.rs', 160],
    ['effect_checked_memptr.rs', 120],
    ['effect_counts.rs', 80],
    ['effect_counts_host.rs', 220],
    ['effect_counts_raw.rs', 80],
    ['effect_diagnostic.rs', 110],
    ['effect_identity.rs', 420],
    ['effect_place_prefix.rs', 80],
    ['effect_pointer_alias.rs', 180],
    ['effect_raw_provenance.rs', 140],
    ['effect_raw_memory_identity.rs', 140],
    ['initialized.rs', 760],
    ['borrow_call.rs', 120],
    ['borrow_check.rs', 550],
    ['borrow_scope.rs', 100],
    ['borrow_summary.rs', 120],
    ['borrow_state.rs', 320],
    ['borrow_usage.rs', 270],
    ['cell_state.rs', 760],
    ['cell_state_raw_copy.rs', 120],
    ['cell_state_raw_range.rs', 140],
    ['cell_state_raw_range_append.rs', 120],
    ['cell_state_raw_range_cover.rs', 180],
    ['cell_state_raw_range_cover_tests.rs', 180],
    ['cell_state_raw_range_count.rs', 90],
    ['cell_state_raw_range_merge.rs', 120],
    ['cell_state_raw_range_model.rs', 80],
    ['cell_state_raw_range_copy.rs', 180],
    ['cell_state_raw_range_offset.rs', 80],
    ['cell_state_raw_range_value.rs', 80],
    ['cell_state_raw_range_value_alias.rs', 80],
    ['cell_state_raw_range_value_alias_tests.rs', 120],
    ['cell_state_tests.rs', 140],
    ['collection_slot_drop_traversal_certified.rs', 130],
    ['collection_slot_drop_proof.rs', 190],
    ['collection_slot_drop_traversal.rs', 220],
    ['collection_slot_drop_traversal_range.rs', 90],
    ['collection_slot_drop_traversal_summary.rs', 140],
    ['collection_slot_lifecycle.rs', 40],
    ['collection_slot_lifecycle_model.rs', 120],
    ['collection_slot_lifecycle_storage_tests.rs', 80],
    ['collection_slot_lifecycle_tests.rs', 160],
    ['collection_slot_lifecycle_transition.rs', 180],
    ['collection_slot_lifecycle_type_tests.rs', 60],
    ['collection_slot_owner_transfer.rs', 210],
    ['collection_slot_owner_transfer_proof.rs', 250],
    ['collection_slot_state_identity.rs', 40],
    ['collection_slot_summary_apply.rs', 180],
    ['collection_slot_summary_apply_return_path.rs', 140],
    ['collection_slot_summary_build.rs', 180],
    ['collection_slot_summary_build_drop_traversal.rs', 70],
    ['collection_slot_summary_build_event.rs', 80],
    ['collection_slot_summary_build_ops.rs', 260],
    ['collection_slot_summary_build_state.rs', 80],
    ['collection_slot_summary_event_apply_proof.rs', 80],
    ['collection_slot_summary_event_proof.rs', 100],
    ['collection_slot_summary_match_state.rs', 80],
    ['collection_slot_summary_model.rs', 120],
    ['collection_slot_summary_replay.rs', 180],
    ['collection_slot_summary_replay_drop_traversal.rs', 80],
    ['collection_slot_summary_return.rs', 60],
    ['collection_slot_summary_return_build.rs', 80],
    ['collection_slot_summary_return_call.rs', 100],
    ['collection_slot_summary_return_collect.rs', 100],
    ['collection_slot_summary_return_model.rs', 40],
    ['collection_slot_summary_return_path.rs', 60],
    ['collection_slot_summary_return_path_call.rs', 220],
    ['collection_slot_summary_return_path_control.rs', 220],
    ['collection_slot_summary_return_path_model.rs', 40],
    ['collection_slot_summary_return_path_slots.rs', 100],
    ['collection_slot_summary_return_path_state.rs', 260],
    ['collection_slot_summary_return_path_value.rs', 340],
    ['collection_slot_summary_return_state.rs', 60],
    ['collection_slot_summary_target.rs', 60],
    ['collection_slot_summary_translate.rs', 180],
    ['collection_slot_summary_translate_drop.rs', 90],
    ['collection_slot_summary_return_unique.rs', 60],
    ['collection_slot_summary_return_value.rs', 320],
    ['collection_slot_storage_release_proof.rs', 80],
    ['collection_slot_state_merge.rs', 180],
    ['collection_slot_state_merge_tests.rs', 220],
    ['collection_slot_state_release.rs', 160],
    ['collection_slot_state_release_tests.rs', 120],
    ['collection_slot_state_relocate.rs', 170],
    ['collection_slot_state_relocate_tests.rs', 150],
    ['collection_slot_state_return.rs', 40],
    ['collection_slot_state_table.rs', 170],
    ['collection_slot_state_table_tests.rs', 140],
    ['collection_slot_state_transfer.rs', 160],
    ['collection_slot_state_transfer_tests.rs', 100],
    ['condition_fact.rs', 180],
    ['owner_check.rs', 800],
    ['owner_entry.rs', 80],
    ['owner_check_utils.rs', 80],
    ['owner_alias.rs', 180],
    ['owner_consumption.rs', 80],
    ['owner_consumption_extent.rs', 80],
    ['owner_drop.rs', 180],
    ['owner_expr.rs', 80],
    ['owner_external_io.rs', 80],
    ['owner_external_io_payload.rs', 220],
    ['owner_host_direct_span.rs', 80],
    ['owner_host_memory_span.rs', 80],
    ['owner_host_memory_summary.rs', 360],
    ['owner_extent.rs', 240],
    ['owner_extent_check.rs', 100],
    ['owner_extent_compare.rs', 80],
    ['owner_extent_coverage.rs', 80],
    ['owner_extent_coverage_place.rs', 80],
    ['owner_extent_expected.rs', 80],
    ['owner_extent_summary.rs', 280],
    ['owner_flow.rs', 620],
    ['owner_host_dependent_span.rs', 120],
    ['owner_host_iov_descriptor.rs', 80],
    ['owner_host_payload_extent.rs', 260],
    ['owner_host_size_outputs.rs', 80],
    ['owner_match_payload.rs', 80],
    ['owner_raw_address.rs', 40],
    ['owner_raw_memory.rs', 260],
    ['owner_raw_memory_cell.rs', 220],
    ['owner_raw_memory_span.rs', 90],
    ['owner_raw_view.rs', 180],
    ['owner_raw_view_model.rs', 60],
    ['owner_raw_view_table.rs', 160],
    ['owner_release.rs', 170],
    ['owner_summary.rs', 480],
    ['owner_summary_canonicalize.rs', 240],
    ['owner_summary_consumed.rs', 80],
    ['owner_summary_host_size_return.rs', 80],
    ['owner_summary_i32_condition_leaf.rs', 180],
    ['owner_summary_i32_leaf.rs', 220],
    ['owner_summary_owner_token_leaf.rs', 100],
    ['owner_summary_owner_token_leaf_tests.rs', 80],
    ['owner_summary_owner_token_type.rs', 120],
    ['owner_summary_parameters.rs', 100],
    ['owner_summary_raw_alias.rs', 140],
    ['owner_summary_raw_alias_branch.rs', 80],
    ['owner_summary_raw_alias_walk.rs', 180],
    ['owner_summary_raw_i32_leaf.rs', 240],
    ['owner_summary_type_size_return.rs', 80],
    ['owner_summary_type_params.rs', 100],
    ['owner_summary_raw_transfer.rs', 150],
    ['owner_summary_variant_build.rs', 360],
    ['owner_summary_resolved_variant.rs', 260],
    ['owner_summary_size_return.rs', 80],
    ['owner_summary_variant_ambiguous.rs', 80],
    ['owner_summary_variant_conditions.rs', 260],
    ['owner_summary_variant_construct.rs', 140],
    ['owner_summary_variant_i32_conditions.rs', 40],
    ['owner_summary_variant_match.rs', 140],
    ['owner_summary_variant_path_conditions.rs', 120],
    ['owner_summary_variant_payload_conditions.rs', 160],
    ['owner_summary_variant_paths.rs', 440],
    ['owner_summary_variant_return.rs', 280],
    ['owner_summary_variant_return_sources.rs', 180],
    ['owner_summary_update.rs', 100],
    ['owner_summary_leaf.rs', 260],
    ['owner_summary_seed_leaf.rs', 100],
    ['owner_summary_raw_consumption.rs', 140],
    ['owner_summary_raw_transfer_tests.rs', 120],
    ['owner_summary_raw_use.rs', 160],
    ['owner_summary_raw_use_branch.rs', 80],
    ['owner_summary_raw_use_call.rs', 90],
    ['owner_summary_raw_use_return.rs', 100],
    ['owner_summary_raw_use_walk.rs', 240],
    ['owner_summary_raw_view_return.rs', 90],
    ['owner_summary_record.rs', 260],
    ['owner_summary_storage_origin.rs', 60],
    ['owner_summary_update_tests.rs', 140],
    ['owner_summary_variant_leaf.rs', 80],
    ['owner_summary_variant_projection.rs', 100],
    ['owner_return.rs', 220],
    ['owner_return_apply.rs', 410],
    ['owner_return_apply_consumption.rs', 100],
    ['owner_return_apply_extent.rs', 120],
    ['owner_return_apply_place.rs', 80],
    ['owner_return_apply_projection.rs', 220],
    ['owner_return_apply_source.rs', 180],
    ['owner_return_unknown.rs', 180],
    ['owner_return_view.rs', 80],
    ['owner_transfer.rs', 120],
    ['owner_variant.rs', 840],
    ['owner_variant_apply.rs', 260],
    ['owner_variant_condition_truth.rs', 80],
    ['owner_variant_lifecycle.rs', 280],
    ['owner_variant_record.rs', 220],
    ['owner_variant_source_list.rs', 80],
    ['owner_variant_unreachable.rs', 80],
    ['owner_variant_utils.rs', 220],
    ['owner_variant_value_condition.rs', 220],
    ['result_variant.rs', 100],
    ['variant_name.rs', 80],
    ['summary_dependency.rs', 220],
    ['summary_index.rs', 80],
    ['summary_worklist.rs', 100],
    ['summary_worklist_order.rs', 80],
    ['summary_worklist_tests.rs', 120],
    ['timing.rs', 80],
    ['trait_identity.rs', 80],
    ['type_var.rs', 100],
    ['type_pattern.rs', 120],
    ['effect_check.rs', 700],
    ['summary.rs', 300],
    ['effect_summary.rs', 250],
    ['effect_summary_identity.rs', 380],
    ['effect_summary_identity_seed.rs', 140],
    ['effect_summary_identity_replay_tests.rs', 140],
    ['effect_summary_identity_tests.rs', 120],
    ['effect_summary_pointer.rs', 220],
    ['effect_summary_pointer_filter.rs', 80],
    ['effect_summary_pointer_seed.rs', 40],
    ['effect_summary_projection.rs', 40],
    ['effect_summary_seed.rs', 180],
    ['effect_summary_seed_alias.rs', 120],
    ['effect_summary_seed_walk.rs', 220],
    ['effect_match.rs', 80],
    ['effect_return_escape.rs', 120],
    ['effect_return_escape_tests.rs', 180],
    ['effect_return_identity.rs', 140],
    ['effect_return_owner_type.rs', 180],
    ['effect_return_pointer.rs', 120],
    ['effect_return_protection.rs', 80],
    ['effect_return_summary_filter.rs', 260],
    ['effect_return_summary_filter_tests.rs', 180],
    ['function_alias.rs', 140],
    ['coverage.rs', 300],
    ['coverage_hir.rs', 260],
    ['coverage_hir_match.rs', 80],
    ['coverage_hir_place.rs', 120],
    ['coverage_hir_projection.rs', 280],
    ['coverage_hir_projection_aggregate.rs', 180],
    ['coverage_hir_raw.rs', 80],
    ['coverage_hir_scope.rs', 100],
    ['coverage_hir_transparent.rs', 240],
    ['coverage_kind.rs', 80],
    ['coverage_operation.rs', 110],
    ['coverage_resource.rs', 540],
    ['coverage_resource_collection_slot.rs', 90],
    ['coverage_resource_place.rs', 100],
    ['dump.rs', 740],
    ['drop_elaboration.rs', 220],
    ['drop_elaboration_bindings.rs', 140],
    ['drop_elaboration_hir_bridge.rs', 260],
    ['drop_elaboration_validate.rs', 120],
    ['drop_model.rs', 80],
    ['drop_plan.rs', 170],
    ['drop_plan_assignment.rs', 80],
    ['drop_point_path.rs', 80],
    ['drop_point_resolve.rs', 220],
    ['drop_point_resolve_assignment.rs', 80],
    ['drop_requirement.rs', 220],
    ['lower.rs', 1150],
    ['lower_call.rs', 120],
    ['lower_collection_slot.rs', 190],
    ['lower_collection_slot_relocate_tests.rs', 160],
    ['lower_collection_slot_tests.rs', 260],
    ['lower_aggregate.rs', 320],
    ['lower_aggregate_projection.rs', 180],
    ['lower_aggregate_selector.rs', 100],
    ['lower_condition.rs', 150],
    ['lower_layout_intrinsic.rs', 80],
    ['lower_match.rs', 100],
    ['lower_raw_address.rs', 620],
    ['lower_raw_address_offset.rs', 90],
    ['lower_raw_address_place.rs', 180],
    ['lower_raw_address_return.rs', 480],
    ['lower_raw_address_return_util.rs', 160],
    ['lower_raw_address_source.rs', 140],
    ['lower_raw_memory.rs', 120],
    ['lower_temporary_scope.rs', 100],
    ['lower_temporary_scope_op.rs', 100],
    ['lower_tests.rs', 340],
    ['raw_realloc.rs', 270],
    ['report.rs', 380],
    ['report_collection_slot.rs', 80],
    ['shadow.rs', 60],
    ['initialized_alias.rs', 580],
    ['initialized_alias_difference.rs', 80],
    ['initialized_alias_difference_flow.rs', 120],
    ['initialized_alias_flow.rs', 550],
    ['initialized_alias_flow_tests.rs', 240],
    ['initialized_alias_i32_condition.rs', 200],
    ['initialized_alias_i32_condition_context.rs', 120],
    ['initialized_alias_i32_condition_tests.rs', 80],
    ['initialized_alias_i32_bounds.rs', 80],
    ['initialized_alias_i32_facts.rs', 180],
    ['initialized_alias_offset.rs', 130],
    ['initialized_alias_i32_relation_condition.rs', 120],
    ['initialized_alias_i32.rs', 80],
    ['initialized_alias_origin.rs', 140],
    ['initialized_alias_origin_tests.rs', 80],
    ['initialized_alias_rank.rs', 120],
    ['initialized_alias_raw_view.rs', 40],
    ['initialized_alias_raw_view_tests.rs', 80],
    ['initialized_alias_relation.rs', 100],
    ['initialized_alias_relation_flow.rs', 100],
    ['initialized_alias_relation_op.rs', 80],
    ['initialized_alias_scalar.rs', 180],
    ['initialized_alias_scalar_copy.rs', 100],
    ['initialized_alias_scale.rs', 140],
    ['initialized_alias_test_support.rs', 40],
    ['initialized_alias_utils.rs', 80],
    ['initialized_alias_tests.rs', 120],
    ['initialized_availability.rs', 120],
    ['initialized_call.rs', 150],
    ['initialized_call_args.rs', 40],
    ['initialized_call_effect.rs', 40],
    ['initialized_collection_slot.rs', 80],
    ['initialized_collection_slot_alias.rs', 80],
    ['initialized_collection_slot_apply.rs', 110],
    ['initialized_collection_slot_dispatch.rs', 100],
    ['initialized_collection_slot_proof.rs', 160],
    ['initialized_collection_slot_relocate.rs', 120],
    ['initialized_collection_slot_tests.rs', 460],
    ['initialized_collection_slot_transfer.rs', 80],
    ['initialized_control.rs', 620],
    ['initialized_control_slot_transfer.rs', 40],
    ['initialized_drop_assignment.rs', 100],
    ['initialized_drop_requirement.rs', 220],
    ['initialized_drop_scope.rs', 80],
    ['initialized_external_seed.rs', 80],
    ['i32_call_facts.rs', 120],
    ['i32_call_facts_scale.rs', 100],
    ['i32_call_facts_scale_tests.rs', 90],
    ['i32_call_facts_tests.rs', 140],
    ['i32_extent_proof.rs', 120],
    ['initialized_external_io.rs', 140],
    ['external_io_iov_layout.rs', 120],
    ['host_dependent_length.rs', 60],
    ['host_memory_address.rs', 40],
    ['host_memory_contract.rs', 320],
    ['host_memory_contract_tests.rs', 210],
    ['host_size_contract.rs', 140],
    ['initialized_external_io_effect.rs', 90],
    ['initialized_external_io_input.rs', 80],
    ['initialized_external_io_iov.rs', 130],
    ['initialized_external_io_payload.rs', 90],
    ['initialized_host_dependent.rs', 90],
    ['initialized_path_state.rs', 180],
    ['initialized_raw_fill.rs', 140],
    ['initialized_raw_memory.rs', 190],
    ['initialized_raw_memory_access.rs', 180],
    ['initialized_raw_memory_bulk.rs', 100],
    ['raw_cell_lifecycle.rs', 240],
    ['raw_cell_value_flow.rs', 300],
    ['raw_cell_value_flow_alias.rs', 180],
    ['raw_cell_value_flow_cell.rs', 130],
    ['raw_cell_value_flow_proof.rs', 80],
    ['raw_cell_value_flow_alias_tests.rs', 120],
    ['raw_cell_value_flow_tests.rs', 180],
    ['initialized_raw_view.rs', 60],
    ['initialized_rekey.rs', 160],
    ['initialized_scalar_flow.rs', 300],
    ['initialized_scalar_flow_ops.rs', 330],
    ['initialized_str_layout.rs', 80],
    ['initialized_summary.rs', 80],
    ['initialized_alias_flow_apply.rs', 180],
    ['initialized_alias_flow_projection.rs', 120],
    ['initialized_alias_flow_raw.rs', 320],
    ['initialized_alias_flow_value_projection.rs', 520],
    ['initialized_alias_host_size.rs', 140],
    ['initialized_alias_type_size.rs', 260],
    ['initialized_summary_apply.rs', 130],
    ['initialized_summary_apply_param.rs', 100],
    ['initialized_summary_apply_return.rs', 120],
    ['initialized_summary_build.rs', 260],
    ['initialized_summary_byte_range_model.rs', 80],
    ['initialized_summary_cells.rs', 140],
    ['initialized_summary_condition.rs', 80],
    ['initialized_summary_engine.rs', 40],
    ['initialized_summary_indirect_release.rs', 120],
    ['initialized_summary_param_byte_range_count.rs', 100],
    ['initialized_summary_param_byte_ranges.rs', 140],
    ['initialized_summary_param_cells.rs', 120],
    ['initialized_summary_raw_release.rs', 80],
    ['initialized_summary_release.rs', 100],
    ['initialized_summary_release_build.rs', 420],
    ['initialized_summary_release_build_tests.rs', 180],
    ['initialized_summary_release_model.rs', 80],
    ['initialized_summary_return_byte_range_count.rs', 100],
    ['initialized_summary_return_byte_ranges.rs', 140],
    ['initialized_summary_seed.rs', 60],
    ['initialized_summary_seed_tests.rs', 120],
    ['initialized_summary_variant_build.rs', 280],
    ['initialized_summary_variant_build_tests.rs', 120],
    ['initialized_summary_variant_condition.rs', 140],
    ['initialized_summary_variant_requirement.rs', 120],
    ['initialized_summary_variant_type.rs', 80],
    ['initialized_summary_variant_unique.rs', 80],
    ['initialized_variant.rs', 500],
    ['compiler_memory_place.rs', 120],
    ['compiler_memory_place_tests.rs', 140],
    ['model.rs', 640],
    ['scalar_primitive.rs', 80],
    ['owner_control.rs', 680],
    ['owner_drop_scope.rs', 260],
    ['owner_state.rs', 400],
    ['place_utils.rs', 460],
    ['raw_pointer_owner_carrier_tests.rs', 100],
    ['raw_pointer_type.rs', 120],
    ['raw_pointer_type_tests.rs', 80],
    ['storage_origin.rs', 320],
]);

const monitoredResourceFiles = new Set(maxLines.keys());
for (const resourceFileName of fs.readdirSync(RESOURCE_DIR)) {
    if (!resourceFileName.endsWith('.rs') || resourceFileName === 'mod.rs') {
        continue;
    }
    assert(
        monitoredResourceFiles.has(resourceFileName),
        `${resourceFileName} must be monitored by resource responsibility line limits`,
    );
}

for (const [name, limit] of maxLines) {
    assertModuleDeclared(mod, name);
    const lines = lineCount(readResource(name));
    assert(lines <= limit, `${name} has ${lines} lines; responsibility split limit is ${limit}`);
}

console.log('resource checker responsibility ok');
