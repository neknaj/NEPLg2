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

for (const name of [
    "lower_bound_i32",
    "upper_bound_i32",
    "contains_i32",
    "count_equal_range_i32",
    "unique_sorted_i32",
]) {
    assert.doesNotMatch(
        implementation,
        new RegExp(`pub\\s+fn\\s+${name}\\b`),
        `kpsearch.${name} must stay private because it accepts raw i32 storage addresses`,
    );
    assert.match(
        implementation,
        new RegExp(`\\bfn\\s+${name}\\b`),
        `kpsearch.${name} internal helper must remain available for safe Vec wrappers`,
    );
}

for (const name of [
    "lower_bound_vec_i32",
    "upper_bound_vec_i32",
    "contains_vec_i32",
    "count_equal_range_vec_i32",
    "unique_sorted_vec_i32",
]) {
    assert.match(
        implementation,
        new RegExp(`pub\\s+fn\\s+${name}\\b`),
        `kpsearch.${name} must be the public owner-based API`,
    );
}

assert.doesNotMatch(implementation, /\bdata_ptr\s*</, "kpsearch must not depend on the removed Vec.data_ptr raw address observer");
assert.doesNotMatch(source, /\balloc_raw\b|\bdealloc_raw\b/, "kpsearch documentation examples must not teach ordinary callers to allocate raw buffers");

console.log("kpsearch raw pointer boundary regression passed");
