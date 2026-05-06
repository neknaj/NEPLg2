#!/usr/bin/env node
"use strict";

const fs = require("fs");
const path = require("path");
const { spawnSync } = require("child_process");

const warnOnly = process.argv.includes("--warn-only");

const checks = [
    "nodesrc/test_stdlib_match_decision_trees.js",
    "nodesrc/test_stdlib_sha256_no_unsafe_unwraps.js",
    "nodesrc/test_run_test_wasi_tmp_dir.js",
    "nodesrc/test_run_test_wasix_missing_wasmer_fallback.js",
    "nodesrc/test_stdlib_sort_merge_no_unsafe_unwraps.js",
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
    "nodesrc/test_stdlib_nm_parser_scanner_boundary.js",
    "nodesrc/test_stdlib_string_doc_no_boilerplate.js",
    "nodesrc/test_tutorial_getting_started_current_style.js",
    "nodesrc/test_stdlib_nm_parser_no_inline_unwraps.js",
    "nodesrc/test_stdlib_nm_parser_no_block_unwraps.js",
    "nodesrc/test_stdlib_diag_error_no_unsafe_unwraps.js",
    "nodesrc/test_stdlib_std_test_no_unsafe_unwraps.js",
    "nodesrc/test_stdlib_kpgraph_no_unsafe_unwraps.js",
    "nodesrc/test_stdlib_wasix_tui_no_unsafe_unwraps.js",
    "nodesrc/test_stdlib_binary_heap_no_unsafe_unwraps.js",
    "nodesrc/test_stdlib_queue_deque_no_unsafe_unwraps.js",
    "nodesrc/test_stdlib_bitset_no_unsafe_unwraps.js",
    "nodesrc/test_stdlib_bitset_borrowed_observers.js",
    "nodesrc/test_stdlib_bitset_update_error_owner.js",
    "nodesrc/test_stdlib_adjacency_matrix_no_unsafe_unwraps.js",
    "nodesrc/test_stdlib_adjacency_matrix_borrowed_observers.js",
    "nodesrc/test_stdlib_adjacency_matrix_update_error_owner.js",
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
    "nodesrc/test_stdlib_segment_tree_no_unsafe_unwraps.js",
    "nodesrc/test_stdlib_segment_tree_borrowed_observers.js",
    "nodesrc/test_stdlib_segment_tree_update_error_owner.js",
    "nodesrc/test_stdlib_stack_no_unsafe_unwraps.js",
    "nodesrc/test_stdlib_hashmap_storage_contract.js",
    "nodesrc/test_stdlib_hashset_storage_contract.js",
    "nodesrc/test_stdlib_vec_no_unsafe_unwraps.js",
    "nodesrc/test_stdlib_vec_borrowed_observers.js",
    "nodesrc/test_static_check_boundary_responsibility.js",
    "nodesrc/test_resource_checker_responsibility.js",
    "nodesrc/test_resource_gate_order.js",
    "nodesrc/test_resource_ir_test_harness_policy.js",
    "nodesrc/test_selfhost_outcome_no_raw_result_cell.js",
    "nodesrc/test_selfhost_cli_args_types_split.js",
    "nodesrc/test_selfhost_cli_args_no_owner_field_reads.js",
    "nodesrc/test_selfhost_cli_driver_boundary.js",
    "nodesrc/test_selfhost_cli_file_io_boundary.js",
    "nodesrc/test_selfhost_cli_reporter_boundary.js",
    "nodesrc/test_selfhost_diag_code_enum.js",
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
    "nodesrc/test_llvm_runner_return_value.js",
    "nodesrc/test_stdlib_builder_owner_boundary.js",
    "nodesrc/test_stdlib_io_bytebuf_owner_boundary.js",
    "nodesrc/test_stdlib_string_no_unsafe_unwraps.js",
    "nodesrc/test_stdlib_stdio_print_i32_boundary.js",
    "nodesrc/test_stdlib_stdio_ansi_boundary.js",
    "nodesrc/test_stdlib_stdio_debug_boundary.js",
    "nodesrc/test_stdlib_stdio_read_boundary.js",
    "nodesrc/test_stdlib_streamio_scanner_boundary.js",
    "nodesrc/test_stdlib_streamio_writer_boundary.js",
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
