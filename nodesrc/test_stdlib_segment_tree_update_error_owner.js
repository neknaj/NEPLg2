#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");

const repoRoot = path.resolve(__dirname, "..");
const relPath = "stdlib/alloc/collections/segment_tree.nepl";
const src = fs.readFileSync(path.join(repoRoot, relPath), "utf8");

const code = src
    .split(/\r?\n/)
    .filter((line) => !/^\s*\/\//.test(line))
    .join("\n");

assert.match(code, /struct\s+SegmentTreeUpdateError:\s+owner\s+<SegmentTree>\s+diag\s+<Diag>/, "SegmentTree update errors must carry the original owner and diagnostic");
assert.match(code, /fn\s+segment_tree_update_error_diag\s+<\(&SegmentTreeUpdateError\)->Diag>\s+\(e\):/, "SegmentTree update diagnostics must be readable without moving the owner");
assert.match(code, /fn\s+segment_tree_update_error_owner\s+<\(SegmentTreeUpdateError\)->SegmentTree>\s+\(e\):/, "SegmentTree update error owner recovery helper is required");
assert.match(code, /fn\s+replace\s+<\(SegmentTree,i32,i32\)\*>Result<SegmentTree,\s*SegmentTreeUpdateError>>\s+\(st,\s*idx,\s*value\):/, "SegmentTree.replace must return an owner-carrying error type");
assert.match(code, /fn\s+add\s+<\(SegmentTree,i32,i32\)\*>Result<SegmentTree,\s*SegmentTreeUpdateError>>\s+\(st,\s*idx,\s*delta\):/, "SegmentTree.add must return an owner-carrying error type");
assert.doesNotMatch(code, /fn\s+(?:replace|add)\s+<\(SegmentTree,i32,i32\)\*>Result<SegmentTree,\s*Diag>>/, "SegmentTree mutating APIs must not lose the owner through Err(Diag)");
assert.match(code, /let\s+e\s+<SegmentTreeUpdateError>\s+SegmentTreeUpdateError\s+st\s+d[\s\S]*err<SegmentTree,\s*SegmentTreeUpdateError>\s+e/, "SegmentTree mutating Err paths must return the input owner in SegmentTreeUpdateError");

for (const testPath of ["stdlib/tests/segment_tree.n.md", "tests/stdlib/segment_tree_collections.n.md"]) {
    const testSrc = fs.readFileSync(path.join(repoRoot, testPath), "utf8");
    assert.match(testSrc, /Result::Err\s+e[0-9]?:[\s\S]*segment_tree_update_error_owner\s+e[0-9]?[\s\S]*free\s+(?:recovered|st0)/, `${testPath} must recover and free SegmentTree owner after update error`);
}

console.log("segment tree update error owner regression passed");
