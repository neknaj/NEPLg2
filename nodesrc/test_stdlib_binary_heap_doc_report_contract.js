#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");

const repoRoot = path.resolve(__dirname, "..");

function source(relPath) {
    return fs.readFileSync(path.join(repoRoot, relPath), "utf8").replace(/\r\n/g, "\n");
}

function assertIncludes(code, needle, message) {
    assert.ok(code.includes(needle), message);
}

function docText(code) {
    return code
        .split("\n")
        .filter((line) => line.startsWith("//:"))
        .join("\n");
}

const root = source("stdlib/alloc/collections/binary_heap.nepl");
const types = source("stdlib/alloc/collections/binary_heap/types.nepl");
const storage = source("stdlib/alloc/collections/binary_heap/storage.nepl");
const order = source("stdlib/alloc/collections/binary_heap/order.nepl");
const api = source("stdlib/alloc/collections/binary_heap/api.nepl");
const observer = source("stdlib/alloc/collections/binary_heap/api/observer.nepl");
const pop = source("stdlib/alloc/collections/binary_heap/api/pop.nepl");

for (const [code, reportName] of [
    [root, "binary_heap_facade_lifecycle_doc"],
    [api, "binary_heap_api_facade_doc"],
    [types, "binary_heap_type_invariant_doc"],
    [types, "binary_heap_pop_type_doc"],
    [observer, "binary_heap_len_doc"],
    [observer, "binary_heap_cap_doc"],
    [observer, "binary_heap_is_empty_doc"],
    [storage, "binary_heap_normalize_capacity_doc"],
    [storage, "binary_heap_item_at_doc"],
    [storage, "binary_heap_store_slot_doc"],
    [storage, "binary_heap_alloc_slots_doc"],
    [storage, "binary_heap_copy_live_slots_doc"],
    [order, "binary_heap_parent_doc"],
    [order, "binary_heap_left_doc"],
    [order, "binary_heap_right_doc"],
    [order, "binary_heap_swap_slots_doc"],
    [order, "binary_heap_sift_up_doc"],
    [order, "binary_heap_sift_down_doc"],
    [pop, "binary_heap_pop_max_doc"],
    [pop, "binary_heap_pop_item_doc"],
    [pop, "binary_heap_pop_heap_doc"],
]) {
    assertIncludes(
        code,
        "neplg2:test[stdio, normalize_newlines]",
        `${reportName} must use stdio report doctest metadata`,
    );
    assertIncludes(code, `test_report_new "${reportName}"`, `${reportName} report doctest is missing`);
}

for (const snippet of [
    "### [契約/けいやく]",
    "時間計算量",
    "Option::None",
    "StdErrorKind::OutOfMemory",
    "Vec Option .T",
    "BinaryHeapPop",
    "owner",
    "O(log n)",
]) {
    assertIncludes(
        [types, storage, order].join("\n"),
        snippet,
        `BinaryHeap docs must keep contract detail: ${snippet}`,
    );
}

assertIncludes(
    storage,
    "sentinel 数値は使いません",
    "BinaryHeap storage docs must keep Option slot state distinct from numeric sentinels",
);
assertIncludes(
    order,
    "missing slot",
    "BinaryHeap order docs must document fail-closed stopping on storage invariant gaps",
);
assertIncludes(
    types,
    "binary_heap_pop_heap",
    "BinaryHeapPop docs must direct callers through the owner accessor",
);
assert.doesNotMatch(
    [docText(types), docText(pop)].join("\n"),
    /field::get(?:_ref)?\s+&?[A-Za-z_][A-Za-z0-9_]*\s+"(?:heap|item)"/,
    "BinaryHeapPop docs must not teach direct field projection",
);

console.log("binary heap doc report contract passed");
