#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");

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

function lineCount(src) {
    return src.trimEnd().split(/\r?\n/).length;
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
    quick: "stdlib/alloc/collections/vec/sort/quick.nepl",
    heap: "stdlib/alloc/collections/vec/sort/heap.nepl",
    merge: "stdlib/alloc/collections/vec/sort/merge.nepl",
};

const sources = Object.fromEntries(Object.entries(files).map(([key, relPath]) => [key, read(relPath)]));
const impl = Object.fromEntries(Object.entries(sources).map(([key, src]) => [key, implementation(src)]));

for (const moduleName of ["common", "simple", "quick", "heap", "merge"]) {
    const pattern = new RegExp(`pub\\s+#import\\s+"\\./sort/${moduleName}"\\s+as\\s+\\*`);
    assert.match(sources.facade, pattern, `vec/sort facade must re-export ${moduleName}`);
}

assert.doesNotMatch(impl.facade, /\bfn\s+/, "vec/sort facade must not own sort implementations");
assert.doesNotMatch(impl.simple, /\bfn\s+/, "vec/sort/simple facade must not own sort implementations");

for (const moduleName of ["insertion", "selection", "exchange", "gap"]) {
    const pattern = new RegExp(`pub\\s+#import\\s+"\\./simple/${moduleName}"\\s+as\\s+\\*`);
    assert.match(sources.simple, pattern, `vec/sort/simple facade must re-export simple/${moduleName}`);
}

for (const [key, limit] of [
    ["facade", 90],
    ["common", 240],
    ["simple", 80],
    ["simpleInsertion", 130],
    ["simpleSelection", 80],
    ["simpleExchange", 150],
    ["simpleGap", 150],
    ["quick", 240],
    ["heap", 190],
    ["merge", 280],
]) {
    assert.ok(lineCount(sources[key]) <= limit, `${files[key]} must stay below ${limit} lines`);
}

for (const name of [
    "sort_lt",
    "sort_le",
    "sort_gt",
    "sort_ge",
    "sort_get_unchecked",
    "sort_set_unchecked",
    "sort_get_unchecked_data",
    "sort_set_unchecked_data",
    "sort_swap_data",
    "sort_swap",
    "sort_is_sorted",
]) {
    assertOwns(impl.common, name, files.common);
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
    "sort_quick_partition_data",
    "sort_quick_range_data",
    "sort_quick",
    "sort_slice_quick",
    "sort_i32",
    "sort_quick_ret",
    "sort",
]) {
    assertOwns(impl.quick, name, files.quick);
    assertNotOwns(impl.facade, name, files.facade);
}

for (const name of ["sort_heap_sift_down_data", "sort_heap", "sort_heap_ret"]) {
    assertOwns(impl.heap, name, files.heap);
    assertNotOwns(impl.facade, name, files.facade);
}

for (const name of [
    "sort_buf_get",
    "sort_buf_set",
    "sort_merge_range_data",
    "sort_merge",
    "sort_merge_ret",
]) {
    assertOwns(impl.merge, name, files.merge);
    assertNotOwns(impl.facade, name, files.facade);
}

console.log("vec sort module split regression passed");
