#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");
const { legacyTypeSyntaxView } = require("./source_policy/nepl_source_view");

const repoRoot = path.resolve(__dirname, "..");
const apiPath = "stdlib/alloc/collections/disjoint_set/api.nepl";
const observerPath = "stdlib/alloc/collections/disjoint_set/api/observer.nepl";
const apiSrc = fs.readFileSync(path.join(repoRoot, apiPath), "utf8");
const observerSrc = fs.readFileSync(path.join(repoRoot, observerPath), "utf8");

const apiCode = legacyTypeSyntaxView(apiSrc);
const code = legacyTypeSyntaxView(observerSrc);

assert.match(apiSrc, /pub\s+#import\s+"\.\/api\/observer"\s+as\s+@merge/, "DisjointSet api facade must re-export borrowed observers through api/observer");
assert.doesNotMatch(apiCode, /\bfn\s+/, "DisjointSet api facade must not keep duplicate borrowed observer wrappers");
assert.match(code, /fn\s+len\s+<\(&DisjointSet\)->i32>\s+\(dsu\):/, "DisjointSet.len must borrow the owner");
assert.doesNotMatch(code, /fn\s+len\s+<\(DisjointSet\)->i32>/, "DisjointSet.len must not consume the owner");
assert.doesNotMatch(code, /fn\s+len_ref\s+<\(&DisjointSet\)->i32>/, "DisjointSet must not keep a duplicate len_ref observer");

assert.match(code, /fn\s+find\s+<\(&DisjointSet,i32\)\*>Result<i32,\s*Diag>>\s+\(dsu,\s*idx\):/, "DisjointSet.find must borrow the owner");
assert.match(code, /fn\s+same\s+<\(&DisjointSet,i32,i32\)\*>Result<bool,\s*Diag>>\s+\(dsu,\s*a,\s*b\):/, "DisjointSet.same must borrow the owner");
assert.match(code, /fn\s+size\s+<\(&DisjointSet,i32\)\*>Result<i32,\s*Diag>>\s+\(dsu,\s*idx\):/, "DisjointSet.size must borrow the owner");

for (const testPath of ["stdlib/tests/disjoint_set.n.md", "tests/stdlib/disjoint_set_collections.n.md"]) {
    const testSrc = fs.readFileSync(path.join(repoRoot, testPath), "utf8");
    assert.match(testSrc, /len\s+&dsu[0-9]?\b/, `${testPath} must exercise borrowed DisjointSet.len`);
    assert.match(testSrc, /(?:find|same|size)\s+&dsu[0-9]?\b/, `${testPath} must exercise borrowed DisjointSet queries`);
    assert.match(testSrc, /free\s+dsu[0-9]?\b/, `${testPath} must explicitly free observed DisjointSet owners`);
    assert.doesNotMatch(testSrc, /len_ref\s+&dsu[0-9]?\b/, `${testPath} must not call duplicate DisjointSet.len_ref`);
    assert.doesNotMatch(testSrc, /len\s+dsu[0-9]?\b/, `${testPath} must not call by-value DisjointSet.len`);
}

console.log("disjoint set borrowed observer regression passed");
