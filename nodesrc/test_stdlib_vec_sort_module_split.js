#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");
const { implementationLineCount } = require("./source_policy/stdlib_builder_owner");

const repoRoot = path.resolve(__dirname, "..");

function read(relPath) {
    return fs.readFileSync(path.join(repoRoot, relPath), "utf8");
}

function implementation(src) {
    return src
        .split(/\r?\n/)
        .filter((line) => !/^\s*\/\//.test(line))
        .join("\n");
}

function assertOwns(moduleSource, name, relPath) {
    assert.match(moduleSource, new RegExp(`fn\\s+${name}\\b`), `${relPath} must own ${name}`);
}

function assertNotOwns(moduleSource, name, relPath) {
    assert.doesNotMatch(moduleSource, new RegExp(`fn\\s+${name}\\b`), `${relPath} must not own ${name}`);
}

const files = {
    facade: "stdlib/alloc/collections/vec/sort.nepl",
    common: "stdlib/alloc/collections/vec/sort/common.nepl",
    simple: "stdlib/alloc/collections/vec/sort/simple.nepl",
    simpleInsertion: "stdlib/alloc/collections/vec/sort/simple/insertion.nepl",
    simpleSelection: "stdlib/alloc/collections/vec/sort/simple/selection.nepl",
    simpleExchange: "stdlib/alloc/collections/vec/sort/simple/exchange.nepl",
    simpleGap: "stdlib/alloc/collections/vec/sort/simple/gap.nepl",
    raw: "stdlib/alloc/collections/vec/sort/raw.nepl",
    rawAccess: "stdlib/alloc/collections/vec/sort/raw/access.nepl",
    rawQuick: "stdlib/alloc/collections/vec/sort/raw/quick.nepl",
    rawHeap: "stdlib/alloc/collections/vec/sort/raw/heap.nepl",
    quick: "stdlib/alloc/collections/vec/sort/quick.nepl",
    heap: "stdlib/alloc/collections/vec/sort/heap.nepl",
    merge: "stdlib/alloc/collections/vec/sort/merge.nepl",
    mergeBuffer: "stdlib/alloc/collections/vec/sort/merge/buffer.nepl",
    mergeRange: "stdlib/alloc/collections/vec/sort/merge/range.nepl",
    mergeApi: "stdlib/alloc/collections/vec/sort/merge/api.nepl",
};

const sources = Object.fromEntries(Object.entries(files).map(([key, relPath]) => [key, read(relPath)]));
const impl = Object.fromEntries(Object.entries(sources).map(([key, src]) => [key, implementation(src)]));

for (const moduleName of ["common", "simple", "quick", "heap", "merge"]) {
    const pattern = new RegExp(`pub\\s+#import\\s+"\\./sort/${moduleName}"\\s+as\\s+\\*`);
    assert.match(sources.facade, pattern, `vec/sort facade must re-export ${moduleName}`);
}

assert.doesNotMatch(impl.facade, /\bfn\s+/, "vec/sort facade must not own sort implementations");
assert.doesNotMatch(impl.simple, /\bfn\s+/, "vec/sort/simple facade must not own sort implementations");
assert.doesNotMatch(impl.raw, /\bfn\s+/, "vec/sort/raw facade must not own raw sort implementations");
assert.doesNotMatch(impl.merge, /\bfn\s+/, "vec/sort/merge facade must not own merge sort implementations");

for (const moduleName of ["insertion", "selection", "exchange", "gap"]) {
    const pattern = new RegExp(`pub\\s+#import\\s+"\\./simple/${moduleName}"\\s+as\\s+\\*`);
    assert.match(sources.simple, pattern, `vec/sort/simple facade must re-export simple/${moduleName}`);
}

for (const moduleName of ["access", "quick", "heap"]) {
    const pattern = new RegExp(`pub\\s+#import\\s+"\\./raw/${moduleName}"\\s+as\\s+\\*`);
    assert.match(sources.raw, pattern, `vec/sort/raw facade must re-export raw/${moduleName}`);
}
assert.match(sources.merge, /pub\s+#import\s+"\.\/merge\/api"\s+as\s+\*/, "vec/sort/merge facade must re-export only the public merge API");
assert.doesNotMatch(sources.merge, /pub\s+#import\s+"\.\/merge\/(?:buffer|range)"\s+as\s+\*/, "vec/sort/merge facade must not re-export raw merge buffer/range helpers");

for (const [key, limit] of [
    ["facade", 90],
    ["common", 240],
    ["simple", 80],
    ["simpleInsertion", 130],
    ["simpleSelection", 80],
    ["simpleExchange", 150],
    ["simpleGap", 150],
    ["raw", 40],
    ["rawAccess", 110],
    ["rawQuick", 90],
    ["rawHeap", 100],
    ["quick", 150],
    ["heap", 140],
    ["merge", 80],
    ["mergeBuffer", 80],
    ["mergeRange", 140],
    ["mergeApi", 140],
]) {
    assert.ok(implementationLineCount(sources[key]) <= limit, `${files[key]} must stay below ${limit} implementation lines`);
}

for (const name of [
    "sort_lt",
    "sort_le",
    "sort_gt",
    "sort_ge",
    "sort_is_sorted",
]) {
    assertOwns(impl.common, name, files.common);
    assertNotOwns(impl.facade, name, files.facade);
}

for (const name of [
    "sort_get_unchecked",
    "sort_set_unchecked",
    "sort_get_unchecked_data",
    "sort_set_unchecked_data",
    "sort_swap_data",
    "sort_swap",
]) {
    assertOwns(impl.rawAccess, name, files.rawAccess);
    assertNotOwns(impl.common, name, files.common);
    assertNotOwns(impl.facade, name, files.facade);
}

for (const [moduleKey, names] of [
    ["simpleInsertion", ["sort_insertion", "sort_gnome"]],
    ["simpleSelection", ["sort_selection"]],
    ["simpleExchange", ["sort_bubble", "sort_cocktail"]],
    ["simpleGap", ["sort_shell", "sort_comb"]],
]) {
    for (const name of names) {
        assertOwns(impl[moduleKey], name, files[moduleKey]);
        assertNotOwns(impl.simple, name, files.simple);
        assertNotOwns(impl.facade, name, files.facade);
    }
}

for (const name of [
    "sort_quick",
    "sort_quick_ret",
    "sort",
]) {
    assertOwns(impl.quick, name, files.quick);
    assertNotOwns(impl.facade, name, files.facade);
}
for (const name of ["sort_quick_partition_data", "sort_quick_range_data", "sort_slice_quick"]) {
    assertOwns(impl.rawQuick, name, files.rawQuick);
    assertNotOwns(impl.quick, name, files.quick);
    assertNotOwns(impl.facade, name, files.facade);
}
for (const [key, source] of Object.entries(impl)) {
    assert.doesNotMatch(source, /\bsort_i32\b/, `${files[key]} must not expose the removed raw sort_i32 adapter`);
}

for (const name of ["sort_heap", "sort_heap_ret"]) {
    assertOwns(impl.heap, name, files.heap);
    assertNotOwns(impl.facade, name, files.facade);
}
assertOwns(impl.rawHeap, "sort_heap_sift_down_data", files.rawHeap);
assertNotOwns(impl.heap, "sort_heap_sift_down_data", files.heap);
assertNotOwns(impl.facade, "sort_heap_sift_down_data", files.facade);

for (const name of ["sort_buf_get", "sort_buf_set"]) {
    assertOwns(impl.mergeBuffer, name, files.mergeBuffer);
    assertNotOwns(impl.merge, name, files.merge);
    assertNotOwns(impl.facade, name, files.facade);
}
assertOwns(impl.mergeRange, "sort_merge_range_data", files.mergeRange);
assertNotOwns(impl.merge, "sort_merge_range_data", files.merge);
assertNotOwns(impl.facade, "sort_merge_range_data", files.facade);
for (const name of ["sort_merge", "sort_merge_ret"]) {
    assertOwns(impl.mergeApi, name, files.mergeApi);
    assertNotOwns(impl.merge, name, files.merge);
    assertNotOwns(impl.facade, name, files.facade);
}

console.log("vec sort module split regression passed");
