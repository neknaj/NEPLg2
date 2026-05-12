#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");

const repoRoot = path.resolve(__dirname, "..");
const typesCode = sourceWithoutComments("stdlib/alloc/collections/disjoint_set/types.nepl");
const apiCode = sourceWithoutComments("stdlib/alloc/collections/disjoint_set/api.nepl");
const mutationCode = sourceWithoutComments("stdlib/alloc/collections/disjoint_set/api/mutation.nepl");

assert.match(typesCode, /struct\s+DisjointSetUpdateError:\s+[\s\S]*owner\s+<DisjointSet>[\s\S]*diag\s+<Diag>/, "DisjointSet union errors must carry the original owner and diagnostic");
assert.match(typesCode, /fn\s+disjoint_set_update_error_diag\s+<\(&DisjointSetUpdateError\)->Diag>\s+\(e\):/, "DisjointSet update diagnostics must be readable without moving the owner");
assert.match(typesCode, /fn\s+disjoint_set_update_error_owner\s+<\(DisjointSetUpdateError\)->DisjointSet>\s+\(e\):/, "DisjointSet update error owner recovery helper is required");
assert.match(apiCode, /pub\s+#import\s+"\.\/api\/mutation"\s+as\s+@merge/, "DisjointSet api facade must re-export owner-consuming mutation API");
assert.doesNotMatch(apiCode, /\bfn\s+union\b/, "DisjointSet api facade must not keep a duplicate union implementation");
assert.match(mutationCode, /fn\s+union\s+<\(DisjointSet,i32,i32\)\*>Result<DisjointSet,\s*DisjointSetUpdateError>>\s+\(dsu,\s*a,\s*b\):/, "DisjointSet.union must return an owner-carrying error type");
assert.doesNotMatch(mutationCode, /fn\s+union\s+<\(DisjointSet,i32,i32\)\*>Result<DisjointSet,\s*Diag>>/, "DisjointSet.union must not lose the owner through Err(Diag)");
assert.match(mutationCode, /let\s+e\s+<DisjointSetUpdateError>\s+DisjointSetUpdateError\s+dsu\s+d[\s\S]*err<DisjointSet,\s*DisjointSetUpdateError>\s+e/, "DisjointSet.union Err path must return the input owner in DisjointSetUpdateError");

for (const testPath of ["stdlib/tests/disjoint_set.n.md", "tests/stdlib/disjoint_set_collections.n.md"]) {
    const testSrc = fs.readFileSync(path.join(repoRoot, testPath), "utf8");
    assert.match(testSrc, /Result::Err\s+e:[\s\S]*disjoint_set_update_error_owner\s+e[\s\S]*free\s+recovered/, `${testPath} must recover and free DisjointSet owner after union error`);
}

console.log("disjoint set union error owner regression passed");

function sourceWithoutComments(file) {
    return fs.readFileSync(path.join(repoRoot, file), "utf8")
        .split(/\r?\n/)
        .filter((line) => !/^\s*\/\//.test(line))
        .join("\n");
}
