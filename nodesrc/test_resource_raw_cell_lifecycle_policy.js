#!/usr/bin/env node
"use strict";

const fs = require("fs");
const path = require("path");

const repoRoot = path.resolve(__dirname, "..");
const resourceRoot = path.join(repoRoot, "nepl-core", "src", "resource");

const lifecycle = read("raw_cell_lifecycle.rs");
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
    "BulkCopyInitializedCopyCells",
    "FillBytes",
    "FillCopyElements",
];

for (const event of requiredEvents) {
    assertIncludes(lifecycle, event, `RawCellLifecycleEvent must include ${event}`);
}

assertIncludes(
    lifecycle,
    "match event",
    "raw cell lifecycle must dispatch through an exhaustive enum match",
);

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
