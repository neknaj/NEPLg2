#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");
const { legacyTypeSyntaxView } = require("./source_policy/nepl_source_view");

const repoRoot = path.resolve(__dirname, "..");
const apiPath = "stdlib/alloc/collections/sparse_set/api.nepl";
const observerPath = "stdlib/alloc/collections/sparse_set/api/observer.nepl";
const apiSrc = fs.readFileSync(path.join(repoRoot, apiPath), "utf8");
const observerSrc = fs.readFileSync(path.join(repoRoot, observerPath), "utf8");

const apiCode = legacyTypeSyntaxView(apiSrc);
const code = legacyTypeSyntaxView(observerSrc);

assert.match(apiSrc, /pub\s+#import\s+"\.\/api\/observer"\s+as\s+@merge/, "SparseSet api facade must re-export borrowed observers through api/observer");
assert.doesNotMatch(apiCode, /\bfn\s+/, "SparseSet api facade must not keep duplicate observer wrappers");
assert.match(code, /fn\s+len\s+<\(&SparseSet\)->i32>\s+\(s\):/, "SparseSet.len must borrow the owner");
assert.doesNotMatch(code, /fn\s+len\s+<\(SparseSet\)->i32>/, "SparseSet.len must not consume the owner");

assert.match(code, /fn\s+universe_len\s+<\(&SparseSet\)->i32>\s+\(s\):/, "SparseSet.universe_len must borrow the owner");
assert.doesNotMatch(code, /fn\s+universe_len\s+<\(SparseSet\)->i32>/, "SparseSet.universe_len must not consume the owner");

assert.match(code, /fn\s+contains\s+<\(&SparseSet,i32\)\*>Result<bool,\s*Diag>>\s+\(s,\s*value\):/, "SparseSet.contains must borrow the owner");
assert.doesNotMatch(code, /fn\s+contains\s+<\(SparseSet,i32\)\*>Result<bool,\s*Diag>>/, "SparseSet.contains must not consume the owner");

for (const testPath of ["stdlib/tests/sparse_set.n.md", "tests/stdlib/sparse_set_collections.n.md"]) {
    const testSrc = fs.readFileSync(path.join(repoRoot, testPath), "utf8");
    assert.match(testSrc, /contains\s+&s/, `${testPath} must exercise borrowed SparseSet.contains`);
    assert.match(testSrc, /len\s+&s/, `${testPath} must exercise borrowed SparseSet.len`);
    assert.match(testSrc, /universe_len\s+&s/, `${testPath} must exercise borrowed SparseSet.universe_len`);
    assert.match(testSrc, /free\s+s/, `${testPath} must explicitly free observed SparseSet owners`);
    assert.doesNotMatch(testSrc, /contains\s+s[0-9]?\s+\d/, `${testPath} must not call by-value SparseSet.contains`);
    assert.doesNotMatch(testSrc, /len\s+s[0-9]?(?:\s|$)/, `${testPath} must not call by-value SparseSet.len`);
    assert.doesNotMatch(testSrc, /universe_len\s+s[0-9]?(?:\s|$)/, `${testPath} must not call by-value SparseSet.universe_len`);
}

console.log("sparse set borrowed observer regression passed");
