#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");

const repoRoot = path.resolve(__dirname, "..");
const relPath = "stdlib/alloc/collections/disjoint_set.nepl";
const src = fs.readFileSync(path.join(repoRoot, relPath), "utf8");

const code = src
    .split(/\r?\n/)
    .filter((line) => !/^\s*\/\//.test(line))
    .join("\n");

assert.match(code, /struct\s+DisjointSetUpdateError:\s+owner\s+<DisjointSet>\s+diag\s+<Diag>/, "DisjointSet union errors must carry the original owner and diagnostic");
assert.match(code, /fn\s+disjoint_set_update_error_diag\s+<\(&DisjointSetUpdateError\)->Diag>\s+\(e\):/, "DisjointSet update diagnostics must be readable without moving the owner");
assert.match(code, /fn\s+disjoint_set_update_error_owner\s+<\(DisjointSetUpdateError\)->DisjointSet>\s+\(e\):/, "DisjointSet update error owner recovery helper is required");
assert.match(code, /fn\s+union\s+<\(DisjointSet,i32,i32\)\*>Result<DisjointSet,\s*DisjointSetUpdateError>>\s+\(dsu,\s*a,\s*b\):/, "DisjointSet.union must return an owner-carrying error type");
assert.doesNotMatch(code, /fn\s+union\s+<\(DisjointSet,i32,i32\)\*>Result<DisjointSet,\s*Diag>>/, "DisjointSet.union must not lose the owner through Err(Diag)");
assert.match(code, /let\s+e\s+<DisjointSetUpdateError>\s+DisjointSetUpdateError\s+dsu\s+d[\s\S]*err<DisjointSet,\s*DisjointSetUpdateError>\s+e/, "DisjointSet.union Err path must return the input owner in DisjointSetUpdateError");

for (const testPath of ["stdlib/tests/disjoint_set.n.md", "tests/stdlib/disjoint_set_collections.n.md"]) {
    const testSrc = fs.readFileSync(path.join(repoRoot, testPath), "utf8");
    assert.match(testSrc, /Result::Err\s+e:[\s\S]*disjoint_set_update_error_owner\s+e[\s\S]*free\s+recovered/, `${testPath} must recover and free DisjointSet owner after union error`);
}

console.log("disjoint set union error owner regression passed");
