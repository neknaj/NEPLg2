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

const root = source("stdlib/alloc/collections/sparse_set.nepl");
const types = source("stdlib/alloc/collections/sparse_set/types.nepl");
const update = source("stdlib/alloc/collections/sparse_set/api/update.nepl");
const diagnostic = source("stdlib/alloc/collections/sparse_set/api/diagnostic.nepl");
const storage = source("stdlib/alloc/collections/sparse_set/storage.nepl");
const membership = source("stdlib/alloc/collections/sparse_set/membership.nepl");
const mutation = source("stdlib/alloc/collections/sparse_set/mutation.nepl");

for (const [code, reportName] of [
    [diagnostic, "sparse_set_diag_len_doc"],
    [diagnostic, "sparse_set_diag_index_doc"],
    [storage, "sparse_set_load_owned_doc"],
    [storage, "sparse_set_store_owned_doc"],
    [storage, "sparse_set_free_arrays_doc"],
    [storage, "sparse_set_alloc_array_doc"],
    [membership, "sparse_set_valid_index_doc"],
    [membership, "sparse_set_contains_raw_doc"],
    [mutation, "sparse_set_insert_storage_doc"],
    [mutation, "sparse_set_remove_storage_doc"],
]) {
    assertReport(code, reportName);
}

for (const snippet of [
    "domain は `[0, n)`",
    "new 0` は valid empty set",
    "0 <= len0 <= n",
    "typed `Vec i32` owner",
    "null pointer / sentinel owner state は使いません",
    "SparseSetUpdateError",
]) {
    assertIncludes(root, snippet, `SparseSet facade docs must keep lifecycle and owner contract: ${snippet}`);
}

for (const snippet of [
    "n >= 0",
    "0 <= len0 <= n",
    "長さ `n`",
    "dense[i] = v",
    "sparse[v] = i",
    "stale",
    "membership は `len0` と dense 照合",
]) {
    assertIncludes(types, snippet, `SparseSet type docs must keep dense/sparse invariant detail: ${snippet}`);
}

for (const snippet of [
    "StdErrorKind::CapacityExceeded",
    "StdErrorKind::IndexOutOfBounds",
    "typed error data",
    "SparseSetUpdateError",
]) {
    assertIncludes(diagnostic, snippet, `SparseSet diagnostic docs must keep enum error contract: ${snippet}`);
}

for (const snippet of [
    "typed `Vec i32`",
    "Option::Some value",
    "Option::None",
    "raw pointer",
    "null sentinel",
    "Vec.get",
    "Vec.replace",
    "Vec.filled",
    "Vec.free",
    "diag_out_of_memory",
    "O(1)",
    "O(n)",
]) {
    assertIncludes(storage, snippet, `SparseSet storage docs must keep typed storage boundary: ${snippet}`);
}

for (const snippet of [
    "0 <= idx < n",
    "domain validation",
    "dense[idx] == value",
    "storage 欠損",
    "`false` に畳みます",
    "Vec.get",
    "O(1)",
]) {
    assertIncludes(membership, snippet, `SparseSet membership docs must keep validation and fail-closed contract: ${snippet}`);
}

for (const snippet of [
    "Option::Some len0",
    "Option::Some len0 + 1",
    "Option::Some len0 - 1",
    "Option::None",
    "storage invariant failure",
    "owner-preserving `SparseSetUpdateError`",
    "rollback は契約しません",
    "old last dense slot",
    "stale",
    "O(1)",
]) {
    assertIncludes(mutation, snippet, `SparseSet mutation docs must keep update traversal contract: ${snippet}`);
}

for (const snippet of [
    "Result::Ok",
    "Result::Err",
    "SparseSetUpdateError",
    "cleanup 用 owner",
    "storage update 前に拒否",
    "rollback は契約しません",
]) {
    assertIncludes(update, snippet, `SparseSet update API docs must keep owner recovery contract: ${snippet}`);
}

assert.doesNotMatch(
    [root, types, update, diagnostic, storage, membership, mutation].join("\n"),
    /rollback を(?:行います|保証します)|rollback is guaranteed/u,
    "SparseSet docs must not claim rollback for storage invariant failure",
);
assert.doesNotMatch(
    diagnostic,
    /message(?:文字列)?を.*契約/u,
    "SparseSet docs must keep enum kind as the error contract, not message strings",
);

console.log("sparse set doc report contract passed");
