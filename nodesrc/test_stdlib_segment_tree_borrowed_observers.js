#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");

const repoRoot = path.resolve(__dirname, "..");
const apiPath = "stdlib/alloc/collections/segment_tree/api.nepl";
const apiSrc = fs.readFileSync(path.join(repoRoot, apiPath), "utf8");

const code = apiSrc
    .split(/\r?\n/)
    .filter((line) => !/^\s*\/\//.test(line))
    .join("\n");

assert.match(code, /fn\s+len\s+<\(&SegmentTree\)->i32>\s+\(st\):/, "SegmentTree.len must borrow the owner");
assert.doesNotMatch(code, /fn\s+len\s+<\(SegmentTree\)->i32>/, "SegmentTree.len must not consume the owner");
assert.doesNotMatch(code, /fn\s+len_ref\s+<\(&SegmentTree\)->i32>/, "SegmentTree must not keep a duplicate len_ref observer");

assert.match(
    code,
    /fn\s+sum_range\s+<\(&SegmentTree,i32,i32\)\*>Result<i32,\s*Diag>>\s+\(st,\s*l,\s*r\):/,
    "SegmentTree.sum_range must borrow the owner",
);
assert.doesNotMatch(
    code,
    /fn\s+sum_range\s+<\(SegmentTree,i32,i32\)\*>Result<i32,\s*Diag>>/,
    "SegmentTree.sum_range must not consume the owner",
);

for (const testPath of ["stdlib/tests/segment_tree.n.md", "tests/stdlib/segment_tree_collections.n.md"]) {
    const testSrc = fs.readFileSync(path.join(repoRoot, testPath), "utf8");
    assert.match(testSrc, /len\s+&st[0-9]?\b/, `${testPath} must exercise borrowed SegmentTree.len`);
    assert.match(testSrc, /sum_range\s+&st[0-9]?\b/, `${testPath} must exercise borrowed SegmentTree.sum_range`);
    assert.match(testSrc, /free\s+st[0-9]?\b/, `${testPath} must explicitly free observed SegmentTree owners`);
    assert.doesNotMatch(testSrc, /len\s+st[0-9]?\b/, `${testPath} must not call by-value SegmentTree.len`);
    assert.doesNotMatch(testSrc, /sum_range\s+st[0-9]?\b/, `${testPath} must not call by-value SegmentTree.sum_range`);
}

console.log("segment tree borrowed observer regression passed");
