#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");
const { legacyTypeSyntaxView } = require("./source_policy/nepl_source_view");

const repoRoot = path.resolve(__dirname, "..");
const apiPath = "stdlib/alloc/collections/adjacency_matrix/api.nepl";
const observerPath = "stdlib/alloc/collections/adjacency_matrix/api/observer.nepl";
const apiSrc = fs.readFileSync(path.join(repoRoot, apiPath), "utf8");
const observerSrc = fs.readFileSync(path.join(repoRoot, observerPath), "utf8");

const apiCode = legacyTypeSyntaxView(apiSrc);
const code = legacyTypeSyntaxView(observerSrc);

assert.match(apiSrc, /pub\s+#import\s+"\.\/api\/observer"\s+as\s+@merge/, "AdjacencyMatrix api facade must re-export borrowed observers through api/observer");
assert.doesNotMatch(apiCode, /\bfn\s+/, "AdjacencyMatrix api facade must not keep duplicate observer wrappers");
assert.match(code, /fn\s+len\s+<\(&AdjacencyMatrix\)->i32>\s+\(g\):/, "AdjacencyMatrix.len must borrow the owner");
assert.doesNotMatch(code, /fn\s+len\s+<\(AdjacencyMatrix\)->i32>/, "AdjacencyMatrix.len must not consume the owner");

assert.match(
    code,
    /fn\s+contains\s+<\(&AdjacencyMatrix,i32,i32\)\*>Result<bool,\s*Diag>>\s+\(g,\s*from,\s*to\):/,
    "AdjacencyMatrix.contains must borrow the owner",
);
assert.doesNotMatch(
    code,
    /fn\s+contains\s+<\(AdjacencyMatrix,i32,i32\)\*>Result<bool,\s*Diag>>/,
    "AdjacencyMatrix.contains must not consume the owner",
);

for (const testPath of ["stdlib/tests/adjacency_matrix.n.md", "tests/stdlib/adjacency_matrix_collections.n.md"]) {
    const testSrc = fs.readFileSync(path.join(repoRoot, testPath), "utf8");
    assert.match(testSrc, /contains\s+&g/, `${testPath} must exercise borrowed AdjacencyMatrix.contains`);
    assert.match(testSrc, /len\s+&g/, `${testPath} must exercise borrowed AdjacencyMatrix.len`);
    assert.match(testSrc, /free\s+g/, `${testPath} must explicitly free observed AdjacencyMatrix owners`);
    assert.doesNotMatch(testSrc, /contains\s+g[0-9]?\b/, `${testPath} must not call by-value AdjacencyMatrix.contains`);
    assert.doesNotMatch(testSrc, /len\s+g[0-9]?\b/, `${testPath} must not call by-value AdjacencyMatrix.len`);
}

console.log("adjacency matrix borrowed observer regression passed");
