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

const root = source("stdlib/alloc/collections/disjoint_set.nepl");
const types = source("stdlib/alloc/collections/disjoint_set/types.nepl");
const observer = source("stdlib/alloc/collections/disjoint_set/api/observer.nepl");
const mutation = source("stdlib/alloc/collections/disjoint_set/api/mutation.nepl");
const diagnostic = source("stdlib/alloc/collections/disjoint_set/api/diagnostic.nepl");
const storage = source("stdlib/alloc/collections/disjoint_set/storage.nepl");
const query = source("stdlib/alloc/collections/disjoint_set/query.nepl");

for (const [code, reportName] of [
    [diagnostic, "disjoint_set_diag_len_doc"],
    [diagnostic, "disjoint_set_diag_index_doc"],
    [storage, "disjoint_set_load_owned_doc"],
    [storage, "disjoint_set_store_owned_doc"],
    [query, "disjoint_set_root_storage_doc"],
    [query, "disjoint_set_size_storage_doc"],
]) {
    assertReport(code, reportName);
}

for (const snippet of [
    "n >= 0",
    "parent` と `sizes`",
    "長/なが]さ `n` の `Vec i32`",
    "0 <= parent[i] < n",
    "parent[root] == root",
    "component size",
    "non-root",
]) {
    assertIncludes(types, snippet, `DisjointSet type docs must keep storage invariant detail: ${snippet}`);
}

for (const snippet of [
    "StdErrorKind::CapacityExceeded",
    "StdErrorKind::IndexOutOfBounds",
    "typed error data",
    "DisjointSetUpdateError",
    "storage invariant failure",
]) {
    assertIncludes(diagnostic, snippet, `DisjointSet diagnostic docs must keep enum error contract: ${snippet}`);
}

for (const snippet of [
    "typed `Vec i32`",
    "Option::Some value",
    "Option::None",
    "raw pointer",
    "sentinel value",
    "Vec.get",
    "Vec.replace",
    "caller",
    "O(1)",
]) {
    assertIncludes(storage, snippet, `DisjointSet storage docs must keep typed storage boundary: ${snippet}`);
}

for (const snippet of [
    "Option::Some root",
    "Option::None",
    "parent.len",
    "cycle / storage invariant failure",
    "path compression は行いません",
    "union-by-size invariant",
    "O(n)",
    "O(1)",
]) {
    assertIncludes(query, snippet, `DisjointSet query docs must keep observer contract: ${snippet}`);
}

for (const snippet of [
    "borrowed observer API",
    "owner を消費しない",
    "不要になったら `free`",
    "path compression は行わず",
]) {
    assertIncludes(observer, snippet, `DisjointSet observer docs must keep borrowed observer contract: ${snippet}`);
}

for (const snippet of [
    "union-by-size",
    "Result::Ok",
    "Result::Err",
    "DisjointSetUpdateError",
    "disjoint_set_update_error_owner",
    "typed `Diag`",
    "storage invariant failure",
    "path compression を行いません",
    "O(log n)",
]) {
    assertIncludes(mutation, snippet, `DisjointSet mutation docs must keep owner recovery contract: ${snippet}`);
}

assertIncludes(root, "DisjointSetUpdateError", "DisjointSet facade docs must mention owner-returning update errors");
assertIncludes(root, "path compression を行わず", "DisjointSet facade docs must mention pure query traversal");

assert.doesNotMatch(
    [root, types, observer, mutation, diagnostic, storage, query].join("\n"),
    /union`?\s+を\s+`?Result\s+DisjointSet\s+Diag`?/u,
    "DisjointSet docs must not describe union as losing owner recovery",
);
assert.doesNotMatch(
    [root, types, observer, mutation, diagnostic, storage, query].join("\n"),
    /path compression を行う/u,
    "DisjointSet docs must not claim path compression is performed",
);

console.log("disjoint set doc report contract passed");
