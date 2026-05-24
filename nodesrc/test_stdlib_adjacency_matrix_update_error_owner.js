#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");
const { legacyTypeSyntaxView } = require("./source_policy/nepl_source_view");

const repoRoot = path.resolve(__dirname, "..");
const typesCode = sourceWithoutComments("stdlib/alloc/collections/adjacency_matrix/types.nepl");
const apiCode = sourceWithoutComments("stdlib/alloc/collections/adjacency_matrix/api.nepl");
const updateCode = sourceWithoutComments("stdlib/alloc/collections/adjacency_matrix/api/update.nepl");

assert.match(typesCode, /struct\s+AdjacencyMatrixUpdateError:\s+[\s\S]*owner\s+<AdjacencyMatrix>[\s\S]*diag\s+<Diag>/, "AdjacencyMatrix update errors must carry the original owner and diagnostic");
assert.match(typesCode, /fn\s+adjacency_matrix_update_error_diag\s+<\(&AdjacencyMatrixUpdateError\)->Diag>\s+\(e\):/, "AdjacencyMatrix update diagnostics must be readable without moving the owner");
assert.match(typesCode, /fn\s+adjacency_matrix_update_error_owner\s+<\(AdjacencyMatrixUpdateError\)->AdjacencyMatrix>\s+\(e\):/, "AdjacencyMatrix update error owner recovery helper is required");
assert.match(apiCode, /pub\s+#import\s+"\.\/api\/update"\s+as\s+@merge/, "AdjacencyMatrix api facade must re-export owner-preserving update APIs through api/update");
assert.doesNotMatch(apiCode, /\bfn\s+/, "AdjacencyMatrix api facade must not keep duplicate update wrappers");
assert.match(updateCode, /fn\s+insert\s+<\(AdjacencyMatrix,i32,i32\)\*>Result<AdjacencyMatrix,\s*AdjacencyMatrixUpdateError>>\s+\(g,\s*from,\s*to\):/, "AdjacencyMatrix.insert must return an owner-carrying error type");
assert.match(updateCode, /fn\s+remove\s+<\(AdjacencyMatrix,i32,i32\)\*>Result<AdjacencyMatrix,\s*AdjacencyMatrixUpdateError>>\s+\(g,\s*from,\s*to\):/, "AdjacencyMatrix.remove must return an owner-carrying error type");
assert.match(updateCode, /fn\s+adjacency_matrix_update\s+<\(AdjacencyMatrix,i32,i32,bool\)\*>Result<AdjacencyMatrix,\s*AdjacencyMatrixUpdateError>>\s+\(g,\s*from,\s*to,\s*set_edge\):/, "AdjacencyMatrix insert/remove must share one owner-preserving update helper");
assert.doesNotMatch(updateCode, /fn\s+(?:insert|remove)\s+<\(AdjacencyMatrix,i32,i32\)\*>Result<AdjacencyMatrix,\s*Diag>>/, "AdjacencyMatrix mutating APIs must not lose the owner through Err(Diag)");
assert.match(updateCode, /let\s+e\s+<AdjacencyMatrixUpdateError>\s+AdjacencyMatrixUpdateError\s+g\s+d[\s\S]*err<AdjacencyMatrix,\s*AdjacencyMatrixUpdateError>\s+e/, "AdjacencyMatrix mutating Err paths must return the input owner in AdjacencyMatrixUpdateError");

for (const testPath of ["stdlib/tests/adjacency_matrix.n.md", "tests/stdlib/adjacency_matrix_collections.n.md"]) {
    const testSrc = fs.readFileSync(path.join(repoRoot, testPath), "utf8");
    assert.match(testSrc, /Result::Err\s+e:[\s\S]*adjacency_matrix_update_error_owner\s+e[\s\S]*free\s+recovered/, `${testPath} must recover and free AdjacencyMatrix owner after update error`);
}

console.log("adjacency matrix update error owner regression passed");

function sourceWithoutComments(file) {
    return legacyTypeSyntaxView(fs.readFileSync(path.join(repoRoot, file), 'utf8'));
}
