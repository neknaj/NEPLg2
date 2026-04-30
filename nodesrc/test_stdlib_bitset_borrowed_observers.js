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

assert.match(code, /fn\s+len\s+<\(&BitSet\)->i32>\s+\(bs\):/, "BitSet.len must borrow the BitSet owner");
assert.doesNotMatch(code, /fn\s+len\s+<\(BitSet\)->i32>/, "BitSet.len must not consume the BitSet owner");

assert.match(
    code,
    /fn\s+contains\s+<\(&BitSet,i32\)\*>Result<bool,\s*Diag>>\s+\(bs,\s*idx\):/,
    "BitSet.contains must borrow the BitSet owner",
);
assert.doesNotMatch(
    code,
    /fn\s+contains\s+<\(BitSet,i32\)\*>Result<bool,\s*Diag>>/,
    "BitSet.contains must not consume the BitSet owner",
);

for (const testPath of ["stdlib/tests/bitset.n.md", "tests/stdlib/bitset_collections.n.md"]) {
    const testSrc = fs.readFileSync(path.join(repoRoot, testPath), "utf8");
    assert.match(testSrc, /contains\s+&bs/, `${testPath} must exercise borrowed BitSet.contains`);
    assert.match(testSrc, /len\s+&bs/, `${testPath} must exercise borrowed BitSet.len`);
    assert.match(testSrc, /free\s+bs/, `${testPath} must explicitly free observed BitSet owners`);
    assert.doesNotMatch(testSrc, /contains\s+bs[0-9]?\b/, `${testPath} must not call by-value BitSet.contains`);
    assert.doesNotMatch(testSrc, /len\s+bs[0-9]?\b/, `${testPath} must not call by-value BitSet.len`);
}

console.log("bitset borrowed observer regression passed");
