#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");

const repoRoot = path.resolve(__dirname, "..");
const relPath = "stdlib/kp/kpgraph.nepl";
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
        "kpgraph must not be a raw-memory-boundary module",
    );
}

for (const rawHelper of [
    /\balloc_raw\b/,
    /\bdealloc_raw\b/,
    /\bload_i32\b/,
    /\bstore_i32\b/,
    /\bload_u8\b/,
    /\bstore_u8\b/,
    /\bmem_ptr_addr\b/,
    /\bdata_mem_ptr\b/,
]) {
    assert.doesNotMatch(
        implementation,
        rawHelper,
        "kpgraph must not manipulate graph or BFS storage through raw addresses",
    );
}

assert.doesNotMatch(
    implementation,
    /\bmat\s+<i32>/,
    "DenseGraph must not expose the dense matrix as a raw i32 address field",
);
assert.doesNotMatch(
    implementation,
    /\b(?:pub\s+)?fn\s+dense_graph_bfs_dist_raw\b/,
    "kpgraph must not keep the raw matrix BFS API",
);
assert.match(
    implementation,
    /pub\s+struct\s+DenseGraph:\s*\n\s+matrix\s+<AdjacencyMatrix>/,
    "DenseGraph must wrap an AdjacencyMatrix owner",
);
assert.match(
    implementation,
    /pub\s+fn\s+dense_graph_new\s+<\(i32\)\*>Result<DenseGraph,\s*Diag>>/,
    "dense_graph_new must return Result<DenseGraph, Diag>",
);
assert.match(
    implementation,
    /pub\s+fn\s+dense_graph_add_undirected\s+<\(DenseGraph,i32,i32\)\*>Result<DenseGraph,\s*DenseGraphUpdateError>>/,
    "dense_graph_add_undirected must consume and return the graph owner",
);
assert.match(
    implementation,
    /pub\s+fn\s+dense_graph_bfs_dist\s+<\(&DenseGraph,i32\)\*>Result<Vec<i32>,\s*Diag>>/,
    "dense_graph_bfs_dist must borrow DenseGraph and return a typed Vec owner result",
);

console.log("kpgraph owner-boundary regression passed");
