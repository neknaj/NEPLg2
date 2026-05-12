#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");

const repoRoot = path.resolve(__dirname, "..");
const apiCode = sourceWithoutComments("stdlib/alloc/collections/fenwick/api.nepl");
const observerCode = sourceWithoutComments("stdlib/alloc/collections/fenwick/api/observer.nepl");
const queryCode = sourceWithoutComments("stdlib/alloc/collections/fenwick/api/query.nepl");

assert.match(apiCode, /pub\s+#import\s+"\.\/api\/observer"\s+as\s+@merge/, "Fenwick api facade must re-export borrowed observer APIs through api/observer");
assert.match(apiCode, /pub\s+#import\s+"\.\/api\/query"\s+as\s+@merge/, "Fenwick api facade must re-export borrowed query APIs through api/query");
assert.doesNotMatch(apiCode, /\bfn\s+/, "Fenwick api facade must not keep duplicate borrowed wrappers");

assert.match(observerCode, /fn\s+len\s+<\(&Fenwick\)->i32>\s+\(fw\):/, "Fenwick.len must borrow the owner");
assert.doesNotMatch(observerCode, /fn\s+len\s+<\(Fenwick\)->i32>/, "Fenwick.len must not consume the owner");

assert.match(queryCode, /fn\s+sum_prefix\s+<\(&Fenwick,i32\)\*>Result<i32,\s*Diag>>\s+\(fw,\s*r\):/, "Fenwick.sum_prefix must borrow the owner");
assert.doesNotMatch(queryCode, /fn\s+sum_prefix\s+<\(Fenwick,i32\)\*>Result<i32,\s*Diag>>/, "Fenwick.sum_prefix must not consume the owner");

assert.match(queryCode, /fn\s+sum_range\s+<\(&Fenwick,i32,i32\)\*>Result<i32,\s*Diag>>\s+\(fw,\s*l,\s*r\):/, "Fenwick.sum_range must borrow the owner");
assert.doesNotMatch(queryCode, /fn\s+sum_range\s+<\(Fenwick,i32,i32\)\*>Result<i32,\s*Diag>>/, "Fenwick.sum_range must not consume the owner");

for (const testPath of ["stdlib/tests/fenwick.n.md", "tests/stdlib/fenwick_collections.n.md"]) {
    const testSrc = fs.readFileSync(path.join(repoRoot, testPath), "utf8");
    assert.match(testSrc, /len\s+&fw/, `${testPath} must exercise borrowed Fenwick.len`);
    assert.match(testSrc, /sum_prefix\s+&fw/, `${testPath} must exercise borrowed Fenwick.sum_prefix`);
    assert.match(testSrc, /sum_range\s+&fw/, `${testPath} must exercise borrowed Fenwick.sum_range`);
    assert.match(testSrc, /free\s+fw/, `${testPath} must explicitly free observed Fenwick owners`);
    assert.doesNotMatch(testSrc, /sum_prefix\s+fw(?:\s|[0-9])/, `${testPath} must not call by-value Fenwick.sum_prefix`);
    assert.doesNotMatch(testSrc, /sum_range\s+fw(?:\s|[0-9])/, `${testPath} must not call by-value Fenwick.sum_range`);
    assert.doesNotMatch(testSrc, /len\s+fw(?:\s|[0-9])/, `${testPath} must not call by-value Fenwick.len`);
}

console.log("fenwick borrowed query regression passed");

function sourceWithoutComments(file) {
    return fs.readFileSync(path.join(repoRoot, file), "utf8")
        .split(/\r?\n/)
        .filter((line) => !/^\s*\/\//.test(line))
        .join("\n");
}
