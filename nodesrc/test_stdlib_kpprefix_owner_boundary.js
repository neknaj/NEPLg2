#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");

const repoRoot = path.resolve(__dirname, "..");
const relPath = "stdlib/kp/kpprefix.nepl";
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
        "kpprefix must not be a raw-memory-boundary module; raw storage belongs behind Vec APIs",
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
        "kpprefix must not manipulate raw prefix storage directly",
    );
}

for (const name of ["prefix_build_i32", "prefix_range_sum_i32"]) {
    assert.doesNotMatch(
        implementation,
        new RegExp(`\\b(?:pub\\s+)?fn\\s+${name}\\b`),
        `kpprefix.${name} must not reintroduce raw i32 storage-address APIs`,
    );
}

assert.match(
    implementation,
    /pub\s+struct\s+PrefixI32:\s*\n\s+data\s+<Vec<i32>>/,
    "PrefixI32 must own a typed Vec<i32> prefix buffer instead of a raw pointer and length pair",
);
assert.doesNotMatch(
    implementation,
    /\bimpl\s+Copy\s+for\s+PrefixI32\b|\bimpl\s+Clone\s+for\s+PrefixI32\b/,
    "PrefixI32 owns storage and must not be Copy or Clone",
);
assert.match(
    implementation,
    /pub\s+fn\s+prefix_build_vec_i32\s+<\(Vec<i32>\)\*>Result<PrefixI32,\s*Diag>>/,
    "prefix_build_vec_i32 must return Result<PrefixI32, Diag>",
);
assert.match(
    implementation,
    /pub\s+fn\s+prefix_sum_i32\s+<\(&PrefixI32,i32,i32\)\*>Result<i32,\s*Diag>>/,
    "prefix_sum_i32 must borrow PrefixI32 and return Result<i32, Diag>",
);

console.log("kpprefix owner-boundary regression passed");
