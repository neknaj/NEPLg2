#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");

const repoRoot = path.resolve(__dirname, "..");
const relPath = "stdlib/alloc/collections/adjacency_matrix.nepl";
const src = fs.readFileSync(path.join(repoRoot, relPath), "utf8");

const code = src
    .split(/\r?\n/)
    .filter((line) => !/^\s*\/\//.test(line))
    .join("\n");

assert.match(code, /struct\s+AdjacencyMatrixUpdateError:\s+owner\s+<AdjacencyMatrix>\s+diag\s+<Diag>/, "AdjacencyMatrix update errors must carry the original owner and diagnostic");
assert.match(code, /fn\s+adjacency_matrix_update_error_diag\s+<\(&AdjacencyMatrixUpdateError\)->Diag>\s+\(e\):/, "AdjacencyMatrix update diagnostics must be readable without moving the owner");
assert.match(code, /fn\s+adjacency_matrix_update_error_owner\s+<\(AdjacencyMatrixUpdateError\)->AdjacencyMatrix>\s+\(e\):/, "AdjacencyMatrix update error owner recovery helper is required");
assert.match(code, /fn\s+insert\s+<\(AdjacencyMatrix,i32,i32\)\*>Result<AdjacencyMatrix,\s*AdjacencyMatrixUpdateError>>\s+\(g,\s*from,\s*to\):/, "AdjacencyMatrix.insert must return an owner-carrying error type");
assert.match(code, /fn\s+remove\s+<\(AdjacencyMatrix,i32,i32\)\*>Result<AdjacencyMatrix,\s*AdjacencyMatrixUpdateError>>\s+\(g,\s*from,\s*to\):/, "AdjacencyMatrix.remove must return an owner-carrying error type");
assert.doesNotMatch(code, /fn\s+(?:insert|remove)\s+<\(AdjacencyMatrix,i32,i32\)\*>Result<AdjacencyMatrix,\s*Diag>>/, "AdjacencyMatrix mutating APIs must not lose the owner through Err(Diag)");
assert.match(code, /let\s+e\s+<AdjacencyMatrixUpdateError>\s+AdjacencyMatrixUpdateError\s+g\s+d[\s\S]*err<AdjacencyMatrix,\s*AdjacencyMatrixUpdateError>\s+e/, "AdjacencyMatrix mutating Err paths must return the input owner in AdjacencyMatrixUpdateError");

for (const testPath of ["stdlib/tests/adjacency_matrix.n.md", "tests/stdlib/adjacency_matrix_collections.n.md"]) {
    const testSrc = fs.readFileSync(path.join(repoRoot, testPath), "utf8");
    assert.match(testSrc, /Result::Err\s+e:[\s\S]*adjacency_matrix_update_error_owner\s+e[\s\S]*free\s+recovered/, `${testPath} must recover and free AdjacencyMatrix owner after update error`);
}

console.log("adjacency matrix update error owner regression passed");
