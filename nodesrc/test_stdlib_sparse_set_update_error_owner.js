#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");
const { legacyTypeSyntaxView } = require("./source_policy/nepl_source_view");

const repoRoot = path.resolve(__dirname, "..");
const typesCode = sourceWithoutComments("stdlib/alloc/collections/sparse_set/types.nepl");
const apiCode = sourceWithoutComments("stdlib/alloc/collections/sparse_set/api.nepl");
const updateCode = sourceWithoutComments("stdlib/alloc/collections/sparse_set/api/update.nepl");

assert.match(typesCode, /struct\s+SparseSetUpdateError:\s+[\s\S]*owner\s+<SparseSet>[\s\S]*diag\s+<Diag>/, "SparseSet update errors must carry the original owner and diagnostic");
assert.match(typesCode, /fn\s+sparse_set_update_error_diag\s+<\(&SparseSetUpdateError\)->Diag>\s+\(e\):/, "SparseSet update diagnostics must be readable without moving the owner");
assert.match(typesCode, /fn\s+sparse_set_update_error_owner\s+<\(SparseSetUpdateError\)->SparseSet>\s+\(e\):/, "SparseSet update error owner recovery helper is required");
assert.match(apiCode, /pub\s+#import\s+"\.\/api\/update"\s+as\s+@merge/, "SparseSet api facade must re-export owner-preserving update APIs through api/update");
assert.doesNotMatch(apiCode, /\bfn\s+/, "SparseSet api facade must not keep duplicate update wrappers");
assert.match(updateCode, /fn\s+insert\s+<\(SparseSet,i32\)\*>Result<SparseSet,\s*SparseSetUpdateError>>\s+\(s,\s*value\):/, "SparseSet.insert must return an owner-carrying error type");
assert.match(updateCode, /fn\s+remove\s+<\(SparseSet,i32\)\*>Result<SparseSet,\s*SparseSetUpdateError>>\s+\(s,\s*value\):/, "SparseSet.remove must return an owner-carrying error type");
assert.match(updateCode, /fn\s+sparse_set_update_err\s+<\(SparseSet,Diag\)->Result<SparseSet,\s*SparseSetUpdateError>>\s+\(s,\s*d\):/, "SparseSet mutating APIs must share one owner-preserving error helper");
assert.doesNotMatch(updateCode, /fn\s+(?:insert|remove)\s+<\(SparseSet,i32\)\*>Result<SparseSet,\s*Diag>>/, "SparseSet mutating APIs must not lose the owner through Err(Diag)");
assert.match(updateCode, /SparseSetUpdateError\s+s\s+d[\s\S]*err\s+e/, "SparseSet mutating Err paths must return the input owner in SparseSetUpdateError");

for (const testPath of ["stdlib/tests/sparse_set.n.md", "tests/stdlib/sparse_set_collections.n.md"]) {
    const testSrc = fs.readFileSync(path.join(repoRoot, testPath), "utf8");
    assert.match(testSrc, /Result::Err\s+e:[\s\S]*sparse_set_update_error_owner\s+e[\s\S]*free\s+recovered/, `${testPath} must recover and free SparseSet owner after update error`);
}

console.log("sparse set update error owner regression passed");

function sourceWithoutComments(file) {
    return legacyTypeSyntaxView(fs.readFileSync(path.join(repoRoot, file), 'utf8'));
}
