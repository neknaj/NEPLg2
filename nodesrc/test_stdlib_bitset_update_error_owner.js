#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");

const repoRoot = path.resolve(__dirname, "..");
const typesCode = sourceWithoutComments("stdlib/alloc/collections/bitset/types.nepl");
const apiCode = sourceWithoutComments("stdlib/alloc/collections/bitset/api.nepl");

assert.match(typesCode, /struct\s+BitSetUpdateError:\s+[\s\S]*owner\s+<BitSet>[\s\S]*diag\s+<Diag>/, "BitSet update errors must carry the original owner and diagnostic");
assert.match(typesCode, /fn\s+bitset_update_error_diag\s+<\(&BitSetUpdateError\)->Diag>\s+\(e\):/, "BitSet update diagnostics must be readable without moving the owner");
assert.match(typesCode, /fn\s+bitset_update_error_owner\s+<\(BitSetUpdateError\)->BitSet>\s+\(e\):/, "BitSet update error owner recovery helper is required");
assert.match(apiCode, /fn\s+insert\s+<\(BitSet,i32\)\*>Result<BitSet,\s*BitSetUpdateError>>\s+\(bs,\s*idx\):/, "BitSet.insert must return an owner-carrying error type");
assert.match(apiCode, /fn\s+remove\s+<\(BitSet,i32\)\*>Result<BitSet,\s*BitSetUpdateError>>\s+\(bs,\s*idx\):/, "BitSet.remove must return an owner-carrying error type");
assert.doesNotMatch(apiCode, /fn\s+(?:insert|remove)\s+<\(BitSet,i32\)\*>Result<BitSet,\s*Diag>>/, "BitSet mutating APIs must not lose the owner through Err(Diag)");
assert.match(apiCode, /let\s+e\s+<BitSetUpdateError>\s+BitSetUpdateError\s+bs\s+d[\s\S]*err<BitSet,\s*BitSetUpdateError>\s+e/, "BitSet mutating Err paths must return the input owner in BitSetUpdateError");

for (const testPath of ["stdlib/tests/bitset.n.md", "tests/stdlib/bitset_collections.n.md"]) {
    const testSrc = fs.readFileSync(path.join(repoRoot, testPath), "utf8");
    assert.match(testSrc, /Result::Err\s+e:[\s\S]*bitset_update_error_owner\s+e[\s\S]*free\s+recovered/, `${testPath} must recover and free BitSet owner after update error`);
}

console.log("bitset update error owner regression passed");

function sourceWithoutComments(file) {
    return fs.readFileSync(path.join(repoRoot, file), "utf8")
        .split(/\r?\n/)
        .filter((line) => !/^\s*\/\//.test(line))
        .join("\n");
}
