#!/usr/bin/env node
"use strict";

const fs = require("fs");
const path = require("path");

const repoRoot = path.resolve(__dirname, "..");
const resourceRoot = path.join(repoRoot, "nepl-core", "src", "resource");

const lifecycle = read("raw_cell_lifecycle.rs");
const resourceIrTests = readTest("resource_ir.rs");
const rawMemoryFiles = [
    "initialized_raw_memory.rs",
    "initialized_raw_memory_access.rs",
    "initialized_raw_fill.rs",
    "initialized_control.rs",
];

const requiredEvents = [
    "MoveOutLoadedCell",
    "StoreValue",
    "DiscardCellsUnderAddress",
    "ReleaseStorage",
    "ReallocSuccessTransfer",
    "BulkCopyInitializedRawState",
    "FillBytes",
    "FillCopyElements",
];

for (const event of requiredEvents) {
    assertIncludes(lifecycle, event, `RawCellLifecycleEvent must include ${event}`);
}

assertIncludes(
    lifecycle,
    "match event",
    "raw cell lifecycle policy is an architecture smoke check and must dispatch through an exhaustive enum match",
);

for (const proofBoundary of [
    "struct CopyRawElementType",
    "CopyRawElementType::new",
    "types.is_copy(ty).then",
    "copy_initialized_copy_raw_cells_covered_by_count",
    "copy_initialized_raw_byte_ranges_for_bulk_copy",
]) {
    assertIncludes(
        lifecycle,
        proofBoundary,
        `raw cell lifecycle must keep typed proof boundary ${proofBoundary}`,
    );
}

for (const regression of [
    "copy_raw_element_type_requires_copy_evidence",
    "resource_ir_cell_check_moves_non_copy_raw_load_cell",
    "resource_ir_cell_check_store_reinitializes_moved_raw_cell",
    "resource_ir_cell_check_raw_fill_with_non_copy_value_does_not_initialize_range",
    "resource_ir_cell_check_bulk_copy_transfers_initialized_byte_ranges",
    "resource_ir_cell_check_bulk_move_transfers_initialized_element_ranges",
    "resource_ir_cell_check_bulk_copy_does_not_transfer_uncovered_byte_ranges",
    "resource_ir_cell_check_realloc_transfers_initialized_byte_ranges",
    "resource_ir_cell_check_realloc_transfers_initialized_element_ranges",
    "resource_ir_cell_check_allows_dealloc_after_non_copy_raw_load",
    "resource_ir_cell_check_aggregate_assignment_clears_stale_byte_range",
]) {
    const source = regression === "copy_raw_element_type_requires_copy_evidence"
        ? lifecycle
        : resourceIrTests;
    assertIncludes(
        source,
        regression,
        `raw cell lifecycle source policy must be backed by semantic regression ${regression}`,
    );
}

for (const file of rawMemoryFiles) {
    const source = read(file);
    assertIncludes(
        source,
        "apply_raw_cell_lifecycle_event",
        `${file} must use the typed raw cell lifecycle boundary`,
    );
    for (const forbidden of [
        "mark_raw_cell_moved(",
        "clear_raw_cells_overwritten_by_store(",
        "release_owned_raw_storage_under(",
        "copy_initialized_copy_raw_cells(",
        "copy_initialized_copy_raw_cells_covered_by_count(",
        "copy_initialized_raw_byte_ranges_under(",
        "copy_initialized_raw_byte_ranges_for_bulk_copy(",
        "extend_initialized_raw_byte_ranges(",
        "mark_initialized_raw_byte_range(",
        "mark_initialized_raw_byte_range_extending_appended_difference(",
    ]) {
        assertNotIncludes(source, forbidden, `${file} must not bypass raw_cell_lifecycle.rs`);
    }
    if (file !== "initialized_raw_fill.rs") {
        assertNotIncludes(
            source,
            "clear_raw_cells_under(",
            `${file} must not bypass raw_cell_lifecycle.rs`,
        );
    }
}

console.log("resource raw cell lifecycle policy: passed");

function read(file) {
    return fs
        .readFileSync(path.join(resourceRoot, file), "utf8")
        .replace(/\r\n/g, "\n");
}

function readTest(file) {
    return fs
        .readFileSync(path.join(repoRoot, "nepl-core", "tests", file), "utf8")
        .replace(/\r\n/g, "\n");
}

function assertIncludes(text, needle, message) {
    if (!text.includes(needle)) {
        throw new Error(message);
    }
}

function assertNotIncludes(text, needle, message) {
    if (text.includes(needle)) {
        throw new Error(`${message}: found ${needle}`);
    }
}
