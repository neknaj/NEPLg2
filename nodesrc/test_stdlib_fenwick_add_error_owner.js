#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");

const repoRoot = path.resolve(__dirname, "..");
const relPath = "stdlib/alloc/collections/fenwick.nepl";
const src = fs.readFileSync(path.join(repoRoot, relPath), "utf8");

const code = src
    .split(/\r?\n/)
    .filter((line) => !/^\s*\/\//.test(line))
    .join("\n");

assert.match(code, /struct\s+FenwickAddError:\s+tree\s+<Fenwick>\s+diag\s+<Diag>/, "Fenwick.add errors must carry the original owner and diagnostic");
assert.match(code, /fn\s+add_error_diag\s+<\(&FenwickAddError\)->Diag>\s+\(e\):/, "Fenwick.add error diagnostics must be readable without moving the owner");
assert.match(code, /fn\s+add_error_tree\s+<\(FenwickAddError\)->Fenwick>\s+\(e\):/, "Fenwick.add error owner recovery helper is required");
assert.match(code, /fn\s+add\s+<\(Fenwick,i32,i32\)\*>Result<Fenwick,\s*FenwickAddError>>\s+\(fw,\s*idx,\s*delta\):/, "Fenwick.add must return an owner-carrying error type");
assert.doesNotMatch(code, /fn\s+add\s+<\(Fenwick,i32,i32\)\*>Result<Fenwick,\s*Diag>>/, "Fenwick.add must not lose the owner through Err(Diag)");
assert.match(code, /let\s+e\s+<FenwickAddError>\s+FenwickAddError\s+fw\s+d[\s\S]*err<Fenwick,\s*FenwickAddError>\s+e/, "Fenwick.add Err path must return the input owner in FenwickAddError");

for (const testPath of ["stdlib/tests/fenwick.n.md", "tests/stdlib/fenwick_collections.n.md"]) {
    const testSrc = fs.readFileSync(path.join(repoRoot, testPath), "utf8");
    assert.match(testSrc, /Result::Err\s+e:[\s\S]*add_error_tree\s+e[\s\S]*free\s+recovered/, `${testPath} must recover and free Fenwick owner after add error`);
}

console.log("fenwick add error owner regression passed");
