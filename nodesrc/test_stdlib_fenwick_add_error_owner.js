#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");
const { legacyTypeSyntaxView } = require("./source_policy/nepl_source_view");

const repoRoot = path.resolve(__dirname, "..");
const typesCode = sourceWithoutComments("stdlib/alloc/collections/fenwick/types.nepl");
const apiCode = sourceWithoutComments("stdlib/alloc/collections/fenwick/api.nepl");
const updateCode = sourceWithoutComments("stdlib/alloc/collections/fenwick/api/update.nepl");

assert.match(typesCode, /struct\s+FenwickAddError:\s+[\s\S]*tree\s+<Fenwick>[\s\S]*diag\s+<Diag>/, "Fenwick.add errors must carry the original owner and diagnostic");
assert.match(typesCode, /fn\s+add_error_diag\s+<\(&FenwickAddError\)->Diag>\s+\(e\):/, "Fenwick.add error diagnostics must be readable without moving the owner");
assert.match(typesCode, /fn\s+add_error_tree\s+<\(FenwickAddError\)->Fenwick>\s+\(e\):/, "Fenwick.add error owner recovery helper is required");
assert.match(apiCode, /pub\s+#import\s+"\.\/api\/update"\s+as\s+@merge/, "Fenwick api facade must re-export owner-preserving update APIs through api/update");
assert.doesNotMatch(apiCode, /\bfn\s+/, "Fenwick api facade must not keep duplicate update wrappers");
assert.match(updateCode, /fn\s+add\s+<\(Fenwick,i32,i32\)\*>Result<Fenwick,\s*FenwickAddError>>\s+\(fw,\s*idx,\s*delta\):/, "Fenwick.add must return an owner-carrying error type");
assert.match(updateCode, /fn\s+fenwick_add_err\s+<\(Fenwick,Diag\)->Result<Fenwick,\s*FenwickAddError>>\s+\(fw,\s*d\):/, "Fenwick.add must share one owner-preserving error helper");
assert.doesNotMatch(updateCode, /fn\s+add\s+<\(Fenwick,i32,i32\)\*>Result<Fenwick,\s*Diag>>/, "Fenwick.add must not lose the owner through Err(Diag)");
assert.match(updateCode, /let\s+e\s+<FenwickAddError>\s+FenwickAddError\s+fw\s+d[\s\S]*err\s+e/, "Fenwick.add Err path must return the input owner in FenwickAddError");

for (const testPath of ["stdlib/tests/fenwick.n.md", "tests/stdlib/fenwick_collections.n.md"]) {
    const testSrc = fs.readFileSync(path.join(repoRoot, testPath), "utf8");
    assert.match(testSrc, /Result::Err\s+e:[\s\S]*add_error_tree\s+e[\s\S]*free\s+recovered/, `${testPath} must recover and free Fenwick owner after add error`);
}

console.log("fenwick add error owner regression passed");

function sourceWithoutComments(file) {
    return legacyTypeSyntaxView(fs.readFileSync(path.join(repoRoot, file), 'utf8'));
}
