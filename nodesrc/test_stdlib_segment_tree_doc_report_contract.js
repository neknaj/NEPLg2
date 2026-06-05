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

function assertReport(code, reportName) {
    assertIncludes(
        code,
        "neplg2:test[stdio, normalize_newlines]",
        `${reportName} must use stdio report doctest metadata`,
    );
    assertIncludes(code, `test_report_new "${reportName}"`, `${reportName} report doctest is missing`);
}

const root = source("stdlib/alloc/collections/segment_tree.nepl");
const types = source("stdlib/alloc/collections/segment_tree/types.nepl");
const update = source("stdlib/alloc/collections/segment_tree/api/update.nepl");
const diagnostic = source("stdlib/alloc/collections/segment_tree/api/diagnostic.nepl");
const layout = source("stdlib/alloc/collections/segment_tree/layout.nepl");
const storage = source("stdlib/alloc/collections/segment_tree/storage.nepl");
const range = source("stdlib/alloc/collections/segment_tree/range.nepl");
const mutation = source("stdlib/alloc/collections/segment_tree/mutation.nepl");

for (const [code, reportName] of [
    [diagnostic, "segment_tree_diag_len_doc"],
    [diagnostic, "segment_tree_diag_index_doc"],
    [diagnostic, "segment_tree_diag_range_doc"],
    [layout, "segment_tree_next_pow2_doc"],
    [layout, "segment_tree_expected_cells_doc"],
    [layout, "segment_tree_storage_expected_len_doc"],
    [storage, "segment_tree_load_owned_doc"],
    [storage, "segment_tree_store_owned_doc"],
    [storage, "segment_tree_pair_sum_doc"],
    [range, "segment_tree_sum_range_storage_doc"],
    [mutation, "segment_tree_rebuild_parents_doc"],
    [mutation, "segment_tree_replace_storage_doc"],
    [mutation, "segment_tree_add_storage_doc"],
]) {
    assertReport(code, reportName);
}

for (const snippet of [
    "i32` の[加算/かさん] monoid",
    "typed `Vec i32` owner",
    "n == 0` でも 1",
    "2 * base",
    "`sum_range` は `[l, r)`",
    "SegmentTreeUpdateError",
]) {
    assertIncludes(root, snippet, `SegmentTree facade docs must keep lifecycle and storage contract: ${snippet}`);
}

for (const snippet of [
    "n >= 0",
    "n <= base",
    "最小の 2 冪",
    "2 * base",
    "unused / neutral cell",
    "i32` 加算 monoid",
]) {
    assertIncludes(types, snippet, `SegmentTree type docs must keep storage invariant detail: ${snippet}`);
}

for (const snippet of [
    "StdErrorKind::CapacityExceeded",
    "StdErrorKind::IndexOutOfBounds",
    "typed error data",
    "SegmentTreeUpdateError",
    "enum kind",
]) {
    assertIncludes(diagnostic, snippet, `SegmentTree diagnostic docs must keep enum error contract: ${snippet}`);
}

for (const snippet of [
    "n <= 1",
    "2 冪",
    "2 * base",
    "Vec.len",
    "storage invariant",
    "O(log n)",
    "O(1)",
]) {
    assertIncludes(layout, snippet, `SegmentTree layout docs must keep base/storage invariant contract: ${snippet}`);
}

for (const snippet of [
    "typed `Vec i32`",
    "Option::Some value",
    "Option::None",
    "raw pointer",
    "sentinel value",
    "Vec.get",
    "Vec.replace",
    "storage boundary",
    "O(1)",
]) {
    assertIncludes(storage, snippet, `SegmentTree storage docs must keep typed storage boundary: ${snippet}`);
}

for (const snippet of [
    "[l, r)",
    "Option::Some total",
    "Option::None",
    "storage invariant failure",
    "typed `Diag`",
    "iterative traversal",
    "O(log n)",
]) {
    assertIncludes(range, snippet, `SegmentTree range docs must keep traversal contract: ${snippet}`);
}

for (const snippet of [
    "true",
    "false",
    "storage invariant failure",
    "owner-preserving `SegmentTreeUpdateError`",
    "rollback は契約しません",
    "seg_pair_sum",
    "O(log n)",
]) {
    assertIncludes(mutation, snippet, `SegmentTree mutation docs must keep update traversal contract: ${snippet}`);
}

for (const snippet of [
    "Result::Ok",
    "Result::Err",
    "SegmentTreeUpdateError",
    "cleanup 用 owner",
    "storage update 前に拒否",
    "rollback は契約しません",
]) {
    assertIncludes(update, snippet, `SegmentTree update API docs must keep owner recovery contract: ${snippet}`);
}

assert.doesNotMatch(
    [root, types, update, diagnostic, layout, storage, range, mutation].join("\n"),
    /rollback を(?:行います|保証します)|rollback is guaranteed/u,
    "SegmentTree docs must not claim rollback for storage invariant failure",
);
assert.doesNotMatch(
    diagnostic,
    /message(?:文字列)?を.*契約/u,
    "SegmentTree docs must keep enum kind as the error contract, not message strings",
);

console.log("segment tree doc report contract passed");
