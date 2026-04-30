#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");

const repoRoot = path.resolve(__dirname, "..");
const relPath = "stdlib/alloc/collections/bitset.nepl";
const src = fs.readFileSync(path.join(repoRoot, relPath), "utf8");

const code = src
    .split(/\r?\n/)
    .filter((line) => !/^\s*\/\//.test(line))
    .join("\n");

assert.match(code, /struct\s+BitSetUpdateError:\s+owner\s+<BitSet>\s+diag\s+<Diag>/, "BitSet update errors must carry the original owner and diagnostic");
assert.match(code, /fn\s+bitset_update_error_diag\s+<\(&BitSetUpdateError\)->Diag>\s+\(e\):/, "BitSet update diagnostics must be readable without moving the owner");
assert.match(code, /fn\s+bitset_update_error_owner\s+<\(BitSetUpdateError\)->BitSet>\s+\(e\):/, "BitSet update error owner recovery helper is required");
assert.match(code, /fn\s+insert\s+<\(BitSet,i32\)\*>Result<BitSet,\s*BitSetUpdateError>>\s+\(bs,\s*idx\):/, "BitSet.insert must return an owner-carrying error type");
assert.match(code, /fn\s+remove\s+<\(BitSet,i32\)\*>Result<BitSet,\s*BitSetUpdateError>>\s+\(bs,\s*idx\):/, "BitSet.remove must return an owner-carrying error type");
assert.doesNotMatch(code, /fn\s+(?:insert|remove)\s+<\(BitSet,i32\)\*>Result<BitSet,\s*Diag>>/, "BitSet mutating APIs must not lose the owner through Err(Diag)");
assert.match(code, /let\s+e\s+<BitSetUpdateError>\s+BitSetUpdateError\s+bs\s+d[\s\S]*err<BitSet,\s*BitSetUpdateError>\s+e/, "BitSet mutating Err paths must return the input owner in BitSetUpdateError");

for (const testPath of ["stdlib/tests/bitset.n.md", "tests/stdlib/bitset_collections.n.md"]) {
    const testSrc = fs.readFileSync(path.join(repoRoot, testPath), "utf8");
    assert.match(testSrc, /Result::Err\s+e:[\s\S]*bitset_update_error_owner\s+e[\s\S]*free\s+recovered/, `${testPath} must recover and free BitSet owner after update error`);
}

console.log("bitset update error owner regression passed");
