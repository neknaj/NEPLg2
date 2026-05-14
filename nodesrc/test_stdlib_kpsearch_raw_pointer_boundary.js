#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");

const repoRoot = path.resolve(__dirname, "..");
const relPath = "stdlib/kp/kpsearch.nepl";
const source = fs.readFileSync(path.join(repoRoot, relPath), "utf8");
const implementation = source
    .split(/\r?\n/)
    .filter((line) => !/^\s*\/\//.test(line))
    .join("\n");

for (const rawImport of [
    /#import\s+"core\/mem"/,
    /#import\s+"core\/mem\/internal"/,
    /#import\s+"core\/mem\/allocator"/,
    /#import\s+"core\/mem\/raw"/,
]) {
    assert.doesNotMatch(
        implementation,
        rawImport,
        "kpsearch must be implemented through typed Vec APIs, not raw-memory-boundary imports",
    );
}

for (const rawHelper of [
    /\balloc_raw\b/,
    /\bdealloc_raw\b/,
    /\bload_i32\b/,
    /\bstore_i32\b/,
    /\bmem_ptr_addr\b/,
    /\bdata_mem_ptr\b/,
]) {
    assert.doesNotMatch(
        implementation,
        rawHelper,
        "kpsearch must not manipulate Vec storage through raw addresses",
    );
}

for (const name of [
    "lower_bound_i32",
    "upper_bound_i32",
    "contains_i32",
    "count_equal_range_i32",
    "unique_sorted_i32",
]) {
    assert.doesNotMatch(
        implementation,
        new RegExp(`\\b(?:pub\\s+)?fn\\s+${name}\\b`),
        `kpsearch.${name} raw pointer helper must not be reintroduced`,
    );
}

for (const [name, signature] of [
    ["lower_bound_vec_i32", /pub\s+fn\s+lower_bound_vec_i32\s+<\(&Vec<i32>,i32\)->i32>/],
    ["upper_bound_vec_i32", /pub\s+fn\s+upper_bound_vec_i32\s+<\(&Vec<i32>,i32\)->i32>/],
    ["contains_vec_i32", /pub\s+fn\s+contains_vec_i32\s+<\(&Vec<i32>,i32\)->bool>/],
    ["count_equal_range_vec_i32", /pub\s+fn\s+count_equal_range_vec_i32\s+<\(&Vec<i32>,i32\)->i32>/],
    ["unique_sorted_vec_i32", /pub\s+fn\s+unique_sorted_vec_i32\s+<\(Vec<i32>\)\*>UniqueSortedVecI32>/],
]) {
    assert.match(
        implementation,
        signature,
        `kpsearch.${name} must expose the typed Vec boundary signature`,
    );
}

console.log("kpsearch Vec API boundary regression passed");
