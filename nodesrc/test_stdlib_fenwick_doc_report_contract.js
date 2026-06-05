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

const root = source("stdlib/alloc/collections/fenwick.nepl");
const types = source("stdlib/alloc/collections/fenwick/types.nepl");
const observer = source("stdlib/alloc/collections/fenwick/api/observer.nepl");
const update = source("stdlib/alloc/collections/fenwick/api/update.nepl");
const apiQuery = source("stdlib/alloc/collections/fenwick/api/query.nepl");
const diagnostic = source("stdlib/alloc/collections/fenwick/api/diagnostic.nepl");
const storage = source("stdlib/alloc/collections/fenwick/storage.nepl");
const query = source("stdlib/alloc/collections/fenwick/query.nepl");
const mutation = source("stdlib/alloc/collections/fenwick/mutation.nepl");

for (const [code, reportName] of [
    [diagnostic, "fenwick_diag_len_doc"],
    [diagnostic, "fenwick_diag_index_doc"],
    [diagnostic, "fenwick_diag_range_doc"],
    [storage, "fenwick_load_owned_doc"],
    [storage, "fenwick_store_owned_doc"],
    [storage, "fenwick_alloc_bit_doc"],
    [storage, "fenwick_free_bit_doc"],
    [query, "fenwick_sum_prefix_storage_doc"],
    [mutation, "fenwick_add_storage_doc"],
]) {
    assertReport(code, reportName);
}

for (const snippet of [
    "n >= 0",
    "n + 1",
    "typed `Vec i32` owner",
    "sentinel / 未使用 cell",
    "1-indexed traversal",
    "lowbit interval",
]) {
    assertIncludes(types, snippet, `Fenwick type docs must keep storage invariant detail: ${snippet}`);
}

for (const snippet of [
    "StdErrorKind::CapacityExceeded",
    "StdErrorKind::IndexOutOfBounds",
    "typed error data",
    "FenwickAddError",
    "enum kind",
    "storage missing cell",
]) {
    assertIncludes(diagnostic, snippet, `Fenwick diagnostic docs must keep enum error contract: ${snippet}`);
}

for (const snippet of [
    "typed `Vec i32`",
    "Option::Some value",
    "Option::None",
    "raw pointer",
    "sentinel value",
    "Vec.get",
    "Vec.replace",
    "Vec.filled",
    "diag_out_of_memory",
    "Vec.free",
    "O(1)",
    "O(bit_len)",
]) {
    assertIncludes(storage, snippet, `Fenwick storage docs must keep typed storage boundary: ${snippet}`);
}

for (const snippet of [
    "Option::Some acc",
    "Option::None",
    "storage invariant failure",
    "typed `Diag`",
    "i & -i",
    "lowbit",
    "O(log n)",
]) {
    assertIncludes(query, snippet, `Fenwick query docs must keep prefix traversal contract: ${snippet}`);
}

for (const snippet of [
    "true",
    "false",
    "storage invariant failure",
    "owner-preserving `FenwickAddError`",
    "rollback は行いません",
    "i & -i",
    "O(log n)",
]) {
    assertIncludes(mutation, snippet, `Fenwick mutation docs must keep update traversal contract: ${snippet}`);
}

for (const snippet of [
    "borrowed observer API",
    "owner を消費しない",
    "不要になったら `free`",
]) {
    assertIncludes(observer, snippet, `Fenwick observer docs must keep borrowed observer contract: ${snippet}`);
}

for (const snippet of [
    "Result::Ok",
    "Result::Err",
    "FenwickAddError",
    "add_error_tree",
    "storage update 前に拒否",
    "cleanup 用 owner",
    "rollback は契約しません",
]) {
    assertIncludes(update, snippet, `Fenwick update docs must keep owner recovery contract: ${snippet}`);
}

assertIncludes(root, "FenwickAddError", "Fenwick facade docs must mention owner-returning update errors");
assertIncludes(apiQuery, "range validation", "Fenwick query API docs must keep range validation boundary");

assert.doesNotMatch(
    [root, types, observer, update, apiQuery, diagnostic, storage, query, mutation].join("\n"),
    /rollback を(?:行います|保証します)|rollback is guaranteed/u,
    "Fenwick docs must not claim rollback for storage invariant failure",
);
assert.doesNotMatch(
    [diagnostic, storage, query, mutation].join("\n"),
    /message(?:文字列)?を.*契約/u,
    "Fenwick docs must keep enum kind as the error contract, not message strings",
);

console.log("fenwick doc report contract passed");
