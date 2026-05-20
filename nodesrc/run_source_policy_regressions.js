#!/usr/bin/env node
"use strict";

const fs = require("fs");
const path = require("path");
const { spawnSync } = require("child_process");

const warnOnly = process.argv.includes("--warn-only");

const checks = [
    "nodesrc/test_stdlib_match_decision_trees.js",
    "nodesrc/test_stdlib_sha256_no_unsafe_unwraps.js",
    "nodesrc/test_stdlib_fs_report_contract.js",
    "nodesrc/test_run_test_wasi_tmp_dir.js",
    "nodesrc/test_run_test_wasix_missing_wasmer_fallback.js",
    "nodesrc/test_run_test_timing_metadata.js",
    "nodesrc/test_stdlib_sort_merge_no_unsafe_unwraps.js",
    "nodesrc/test_stdlib_vec_sort_module_split.js",
    "nodesrc/test_stdlib_btree_insert_no_unsafe_grow_unwraps.js",
    "nodesrc/test_stdlib_bytebuf_utf8_boundary.js",
    "nodesrc/test_stdlib_fs_no_unsafe_unwraps.js",
    "nodesrc/test_stdlib_cliarg_no_unsafe_unwraps.js",
    "nodesrc/test_stdlib_streamio_no_unsafe_unwraps.js",
    "nodesrc/test_stdlib_no_unsafe_helpers.js",
    "nodesrc/test_stdlib_math_module_split.js",
    "nodesrc/test_stdlib_cast_doc_no_boilerplate.js",
    "nodesrc/test_stdlib_json_doc_no_boilerplate.js",
    "nodesrc/test_stdlib_nm_parser_doc_no_boilerplate.js",
    "nodesrc/test_stdlib_nm_json_escape_boundary.js",
    "nodesrc/test_stdlib_nm_parser_document_boundary.js",
    "nodesrc/test_stdlib_nm_parser_scanner_boundary.js",
    "nodesrc/test_stdlib_nm_parser_json_inline_boundary.js",
    "nodesrc/test_stdlib_nm_parser_json_section_boundary.js",
    "nodesrc/test_stdlib_nm_html_escape_boundary.js",
    "nodesrc/test_stdlib_nm_html_heading_boundary.js",
    "nodesrc/test_stdlib_nm_html_inline_boundary.js",
    "nodesrc/test_stdlib_nm_html_section_boundary.js",
    "nodesrc/test_stdlib_string_doc_no_boilerplate.js",
    "nodesrc/test_stdlib_documentation_contract.js",
    "nodesrc/test_tutorial_getting_started_current_style.js",
    "nodesrc/test_tutorial_vec_basics_report_contract.js",
    "nodesrc/test_features_tui_report_contract.js",
    "nodesrc/test_stdlib_nm_parser_no_inline_unwraps.js",
    "nodesrc/test_stdlib_nm_parser_no_block_unwraps.js",
    "nodesrc/test_stdlib_diag_error_no_unsafe_unwraps.js",
    "nodesrc/test_stdlib_std_test_no_unsafe_unwraps.js",
    "nodesrc/test_stdlib_kpgraph_no_unsafe_unwraps.js",
    "nodesrc/test_stdlib_kpsearch_raw_pointer_boundary.js",
    "nodesrc/test_stdlib_wasix_tui_no_unsafe_unwraps.js",
    "nodesrc/test_stdlib_binary_heap_no_unsafe_unwraps.js",
    "nodesrc/test_stdlib_queue_deque_no_unsafe_unwraps.js",
    "nodesrc/test_stdlib_btree_borrowed_observers.js",
    "nodesrc/test_stdlib_bitset_no_unsafe_unwraps.js",
    "nodesrc/test_stdlib_bitset_borrowed_observers.js",
    "nodesrc/test_stdlib_bitset_update_error_owner.js",
    "nodesrc/test_stdlib_adjacency_matrix_no_unsafe_unwraps.js",
    "nodesrc/test_stdlib_adjacency_matrix_borrowed_observers.js",
    "nodesrc/test_stdlib_adjacency_matrix_update_error_owner.js",
    "nodesrc/test_stdlib_adjacency_matrix_doc_report_contract.js",
    "nodesrc/test_stdlib_bloom_filter_no_unsafe_unwraps.js",
    "nodesrc/test_stdlib_bloom_filter_borrowed_observers.js",
    "nodesrc/test_stdlib_counting_bloom_filter_no_unsafe_unwraps.js",
    "nodesrc/test_stdlib_counting_bloom_filter_borrowed_observers.js",
    "nodesrc/test_stdlib_fenwick_no_unsafe_unwraps.js",
    "nodesrc/test_stdlib_fenwick_borrowed_queries.js",
    "nodesrc/test_stdlib_fenwick_add_error_owner.js",
    "nodesrc/test_stdlib_ringbuffer_no_unsafe_unwraps.js",
    "nodesrc/test_stdlib_ringbuffer_borrowed_observers.js",
    "nodesrc/test_stdlib_disjoint_set_no_unsafe_unwraps.js",
    "nodesrc/test_stdlib_disjoint_set_borrowed_observers.js",
    "nodesrc/test_stdlib_disjoint_set_union_error_owner.js",
    "nodesrc/test_stdlib_list_no_unsafe_unwraps.js",
    "nodesrc/test_stdlib_sparse_set_no_unsafe_unwraps.js",
    "nodesrc/test_stdlib_sparse_set_borrowed_observers.js",
    "nodesrc/test_stdlib_sparse_set_update_error_owner.js",
    "nodesrc/test_stdlib_segment_tree_no_unsafe_unwraps.js",
    "nodesrc/test_stdlib_segment_tree_borrowed_observers.js",
    "nodesrc/test_stdlib_segment_tree_update_error_owner.js",
    "nodesrc/test_stdlib_stack_no_unsafe_unwraps.js",
    "nodesrc/test_stdlib_collection_cleanup_contract.js",
    "nodesrc/test_stdlib_hashmap_storage_contract.js",
    "nodesrc/test_stdlib_hashset_storage_contract.js",
    "nodesrc/test_stdlib_hash_string_access_boundary.js",
    "nodesrc/test_ci_examples_doctest_job.js",
    "nodesrc/test_examples_string_direct_imports.js",
    "nodesrc/test_stdlib_vec_no_unsafe_unwraps.js",
    "nodesrc/test_stdlib_vec_borrowed_observers.js",
    "nodesrc/test_stdlib_memptr_owner_field_policy.js",
    "nodesrc/test_stdlib_mem_internal_region_new_docs.js",
    "nodesrc/test_stdlib_core_mem_boundary.js",
    "nodesrc/test_typekind_doc_free_policy.js",
    "nodesrc/test_static_check_boundary_responsibility.js",
    "nodesrc/test_abstraction_static_verification_policy.js",
    "nodesrc/test_resource_checker_responsibility.js",
    "nodesrc/test_parser_backend_responsibility_policy.js",
    "nodesrc/test_resource_gate_order.js",
    "nodesrc/test_monomorphize_unresolved_api_policy.js",
    "nodesrc/test_resource_ir_test_harness_policy.js",
    "nodesrc/test_selfhost_outcome_no_raw_result_cell.js",
    "nodesrc/test_selfhost_cli_args_types_split.js",
    "nodesrc/test_selfhost_cli_args_doc_report_contract.js",
    "nodesrc/test_selfhost_cli_args_no_owner_field_reads.js",
    "nodesrc/test_selfhost_cli_driver_boundary.js",
    "nodesrc/test_selfhost_cli_driver_report_contract.js",
    "nodesrc/test_selfhost_cli_file_io_boundary.js",
    "nodesrc/test_selfhost_cli_file_io_report_contract.js",
    "nodesrc/test_selfhost_req_report_contract.js",
    "nodesrc/test_selfhost_cli_reporter_boundary.js",
    "nodesrc/test_selfhost_cli_reporter_report_contract.js",
    "nodesrc/test_selfhost_lexer_report_contract.js",
    "nodesrc/test_selfhost_lexer_split_contract.js",
    "nodesrc/test_selfhost_parser_report_contract.js",
    "nodesrc/test_selfhost_checker_report_contract.js",
    "nodesrc/test_selfhost_module_loader_report_contract.js",
    "nodesrc/test_selfhost_diag_outcome_report_contract.js",
    "nodesrc/test_selfhost_import_spec_report_contract.js",
    "nodesrc/test_selfhost_module_graph_report_contract.js",
    "nodesrc/test_selfhost_stdlib_map_report_contract.js",
    "nodesrc/test_selfhost_diag_code_enum.js",
    "nodesrc/test_selfhost_model_no_numeric_kind_tags.js",
    "nodesrc/test_selfhost_builtin_signature_payload.js",
    "nodesrc/test_selfhost_type_record_payload.js",
    "nodesrc/test_selfhost_type_arena_report_contract.js",
    "nodesrc/test_selfhost_hir_range_payload.js",
    "nodesrc/test_selfhost_mono_instance_absence.js",
    "nodesrc/test_selfhost_hir_expr_id_absence.js",
    "nodesrc/test_selfhost_def_id_absence.js",
    "nodesrc/test_selfhost_hir_expr_payload.js",
    "nodesrc/test_selfhost_hir_report_contract.js",
    "nodesrc/test_selfhost_name_resolver_report_contract.js",
    "nodesrc/test_selfhost_lexer_raw_mode_directive_enum.js",
    "nodesrc/test_selfhost_source_text_report_contract.js",
    "nodesrc/test_selfhost_source_text_no_recursive_line_map.js",
    "nodesrc/test_selfhost_string_helpers_boundary.js",
    "nodesrc/test_selfhost_parser_tokenkind_match.js",
    "nodesrc/test_stdlib_byte_scanner_helpers_boundary.js",
    "nodesrc/test_stdlib_nm_no_raw_aggregate_detours.js",
    "nodesrc/test_diagnostic_code_first_boundary.js",
    "nodesrc/test_editor_diagnostic_code_contract.js",
    "nodesrc/test_doctest_diag_code_metadata.js",
    "nodesrc/test_doctest_exit_code_metadata.js",
    "nodesrc/test_doctest_std_test_assertion_report_contract.js",
    "nodesrc/test_doctest_assertion_ret_policy.js",
    "nodesrc/test_nmd_report_metadata_policy.js",
    "nodesrc/test_nepl_doc_report_metadata_policy.js",
    "nodesrc/test_core_char_doc_report_contract.js",
    "nodesrc/test_core_result_doc_report_contract.js",
    "nodesrc/test_core_traits_doc_report_contract.js",
    "nodesrc/test_stdlib_traits_order_report_contract.js",
    "nodesrc/test_stdlib_traits_hash_report_contract.js",
    "nodesrc/test_stdlib_io_nmd_report_contract.js",
    "nodesrc/test_stdlib_pipe_collections_report_contract.js",
    "nodesrc/test_stdlib_traits_serde_report_contract.js",
    "nodesrc/test_stdlib_btree_search_doc_report_contract.js",
    "nodesrc/test_string_trim_doc_report_contract.js",
    "nodesrc/test_alloc_string_doc_report_contract.js",
    "nodesrc/test_stdlib_string_report_contract.js",
    "nodesrc/test_stdlib_string_nmd_report_contract.js",
    "nodesrc/test_stdlib_btreemap_report_contract.js",
    "nodesrc/test_stdlib_btreeset_report_contract.js",
    "nodesrc/test_stdlib_cliarg_report_contract.js",
    "nodesrc/test_stdlib_collections_diag_report_contract.js",
    "nodesrc/test_stdlib_diag_nmd_report_contract.js",
    "nodesrc/test_stdlib_rand_report_contract.js",
    "nodesrc/test_stdlib_fs_nmd_report_contract.js",
    "nodesrc/test_stdlib_json_nmd_report_contract.js",
    "nodesrc/test_stdlib_error_nmd_report_contract.js",
    "nodesrc/test_stdlib_hash_nmd_report_contract.js",
    "nodesrc/test_stdlib_vec_pop_doc_report_contract.js",
    "nodesrc/test_llvm_runner_return_value.js",
    "nodesrc/test_stdlib_builder_owner_boundary.js",
    "nodesrc/test_stdlib_io_bytebuf_owner_boundary.js",
    "nodesrc/test_stdlib_string_no_unsafe_unwraps.js",
    "nodesrc/test_stdlib_string_utf8_boundary.js",
    "nodesrc/test_stdlib_string_storage_boundary.js",
    "nodesrc/test_stdlib_string_access_boundary.js",
    "nodesrc/test_stdlib_string_char_report_contract.js",
    "nodesrc/test_stdlib_string_search_boundary.js",
    "nodesrc/test_stdlib_string_slice_boundary.js",
    "nodesrc/test_stdlib_string_split_boundary.js",
    "nodesrc/test_stdlib_string_integer_boundary.js",
    "nodesrc/test_stdlib_string_float_boundary.js",
    "nodesrc/test_stdlib_string_facade_boundary.js",
    "nodesrc/test_stdlib_text_boundary.js",
    "nodesrc/test_stdlib_utf8_validation_doc_report_contract.js",
    "nodesrc/test_stdlib_text_utf8_report_contract.js",
    "nodesrc/test_stdlib_stdio_print_i32_boundary.js",
    "nodesrc/test_stdlib_stdio_ansi_boundary.js",
    "nodesrc/test_stdlib_stdio_debug_boundary.js",
    "nodesrc/test_stdlib_stdio_read_boundary.js",
    "nodesrc/test_stdlib_streamio_scanner_boundary.js",
    "nodesrc/test_stdlib_streamio_writer_boundary.js",
    "nodesrc/test_zed_extension_no_tracked_target.js",
];

let failures = 0;

for (const check of checks) {
    group(check);
    const result = spawnSync(process.execPath, [check], {
        cwd: process.cwd(),
        stdio: "inherit",
        env: process.env,
    });
    endGroup();

    const status = result.status ?? 1;
    if (status === 0) {
        continue;
    }

    failures += 1;
    const message = `${check} failed with exit code ${status}`;
    if (warnOnly) {
        warn(message);
        appendSummary(`- warning: \`${check}\` failed with exit code ${status}\n`);
    } else {
        console.error(message);
        process.exit(status);
    }
}

if (failures > 0) {
    const message = `${failures} source policy regression(s) failed; downstream CI jobs continue because --warn-only is enabled.`;
    warn(message);
    appendSummary(`\n${message}\n`);
}

process.exit(0);

function group(name) {
    if (process.env.GITHUB_ACTIONS === "true") {
        console.log(`::group::${escapeCommand(name)}`);
    } else {
        console.log(`\n[source-policy] ${name}`);
    }
}

function endGroup() {
    if (process.env.GITHUB_ACTIONS === "true") {
        console.log("::endgroup::");
    }
}

function warn(message) {
    if (process.env.GITHUB_ACTIONS === "true") {
        console.log(`::warning title=Source policy regression::${escapeCommand(message)}`);
    } else {
        console.warn(`[source-policy warning] ${message}`);
    }
}

function appendSummary(text) {
    const summaryPath = process.env.GITHUB_STEP_SUMMARY;
    if (!summaryPath) {
        return;
    }
    fs.appendFileSync(path.resolve(summaryPath), text);
}

function escapeCommand(value) {
    return String(value)
        .replace(/%/g, "%25")
        .replace(/\r/g, "%0D")
        .replace(/\n/g, "%0A");
}
