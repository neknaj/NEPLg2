#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");

const repoRoot = path.resolve(__dirname, "..");

const modules = [
    {
        name: "kpfenwick",
        relPath: "stdlib/kp/kpfenwick.nepl",
        forbiddenSignatures: [
            /pub\s+fn\s+fenwick_new\s+<\(i32\)\*>i32>/,
            /pub\s+fn\s+fenwick_free\s+<\(i32\)/,
            /pub\s+fn\s+fenwick_add\s+<\(i32,/,
            /pub\s+fn\s+fenwick_sum_prefix\s+<\(i32,/,
            /pub\s+fn\s+fenwick_sum_range\s+<\(i32,/,
        ],
        requiredSignatures: [
            /pub\s+fn\s+fenwick_new\s+<\(i32\)\*>Result<Fenwick,\s*Diag>>/,
            /pub\s+fn\s+fenwick_free\s+<\(Fenwick\)\*>\(\)>/,
            /pub\s+fn\s+fenwick_add\s+<\(Fenwick,i32,i32\)\*>Result<Fenwick,\s*FenwickAddError>>/,
            /pub\s+fn\s+fenwick_sum_prefix\s+<\(&Fenwick,i32\)\*>Result<i32,\s*Diag>>/,
            /pub\s+fn\s+fenwick_sum_range\s+<\(&Fenwick,i32,i32\)\*>Result<i32,\s*Diag>>/,
        ],
        requiredImports: [/alloc\/collections\/fenwick/],
    },
    {
        name: "kpdsu",
        relPath: "stdlib/kp/kpdsu.nepl",
        forbiddenSignatures: [
            /pub\s+fn\s+dsu_new\s+<\(i32\)\*>i32>/,
            /pub\s+fn\s+dsu_free\s+<\(i32\)/,
            /pub\s+fn\s+dsu_find\s+<\(i32,/,
            /pub\s+fn\s+dsu_same\s+<\(i32,/,
            /pub\s+fn\s+dsu_unite\s+<\(i32,/,
            /pub\s+fn\s+dsu_size\s+<\(i32,/,
        ],
        requiredSignatures: [
            /pub\s+fn\s+dsu_new\s+<\(i32\)\*>Result<DisjointSet,\s*Diag>>/,
            /pub\s+fn\s+dsu_free\s+<\(DisjointSet\)\*>\(\)>/,
            /pub\s+fn\s+dsu_find\s+<\(&DisjointSet,i32\)\*>Result<i32,\s*Diag>>/,
            /pub\s+fn\s+dsu_same\s+<\(&DisjointSet,i32,i32\)\*>Result<bool,\s*Diag>>/,
            /pub\s+fn\s+dsu_unite\s+<\(DisjointSet,i32,i32\)\*>Result<DisjointSet,\s*DisjointSetUpdateError>>/,
            /pub\s+fn\s+dsu_size\s+<\(&DisjointSet,i32\)\*>Result<i32,\s*Diag>>/,
        ],
        requiredImports: [/alloc\/collections\/disjoint_set/],
    },
];

for (const moduleInfo of modules) {
    const source = fs.readFileSync(path.join(repoRoot, moduleInfo.relPath), "utf8");
    const implementation = stripComments(source);

    assertNoRawMemoryBoundary(moduleInfo.name, implementation);

    for (const signature of moduleInfo.forbiddenSignatures) {
        assert.doesNotMatch(
            implementation,
            signature,
            `${moduleInfo.name} must not expose raw i32 owner-handle signatures`,
        );
    }

    for (const signature of moduleInfo.requiredSignatures) {
        assert.match(
            implementation,
            signature,
            `${moduleInfo.name} must expose the typed owner/borrow Result API`,
        );
    }

    for (const importPattern of moduleInfo.requiredImports) {
        assert.match(
            implementation,
            importPattern,
            `${moduleInfo.name} must delegate to the typed alloc/collections implementation`,
        );
    }
}

console.log("kp Fenwick/DSU owner-boundary regression passed");

function assertNoRawMemoryBoundary(moduleName, implementation) {
    for (const rawImport of [
        /#import\s+"core\/mem"/,
        /#import\s+"core\/mem\/internal"/,
        /#import\s+"core\/mem\/allocator"/,
        /#import\s+"core\/mem\/raw"/,
    ]) {
        assert.doesNotMatch(
            implementation,
            rawImport,
            `${moduleName} must not be a raw-memory-boundary module`,
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
            `${moduleName} must not manipulate raw storage directly`,
        );
    }
}

function stripComments(source) {
    return source
        .split(/\r?\n/)
        .filter((line) => !/^\s*\/\//.test(line))
        .join("\n");
}
