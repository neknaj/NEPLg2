#!/usr/bin/env node

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');

const repoRoot = path.resolve(__dirname, '..');
const relPath = 'stdlib/alloc/collections/segment_tree.nepl';
const src = fs.readFileSync(path.join(repoRoot, relPath), 'utf8');

const code = src
    .split(/\r?\n/)
    .filter((line) => !/^\s*\/\//.test(line))
    .join('\n');

const forbidden = [
    /\bunwrap\b/,
    /\bunwrap_ok\b/,
    /\bunwrap_err\b/,
    /\buwok\b/,
    /\buwerr\b/,
    /#intrinsic\s+"unreachable"/,
];

for (const pattern of forbidden) {
    assert.doesNotMatch(code, pattern, `${relPath} must not use unsafe unwrap helpers in implementation code`);
}

assert.match(code, /fn\s+seg_store_owned\s+/, 'SegmentTree must centralize owned array raw stores');
assert.match(code, /fn\s+seg_load_owned\s+/, 'SegmentTree must centralize owned array loads');
assert.match(code, /struct\s+SegmentTreeUpdateError:\s+tree\s+<SegmentTree>\s+diag\s+<Diag>/, 'SegmentTree update errors must carry the original owner and diagnostic');
assert.match(code, /fn\s+update_error_diag\s+<\(&SegmentTreeUpdateError\)->Diag>\s+\(e\):/, 'SegmentTree update error diagnostics must be readable without moving the owner');
assert.match(code, /fn\s+update_error_tree\s+<\(SegmentTreeUpdateError\)->SegmentTree>\s+\(e\):/, 'SegmentTree update error owner recovery helper is required');
assert.match(code, /fn\s+len\s+<\(&SegmentTree\)->i32>\s+\(st\):/, 'SegmentTree.len must borrow the owner');
assert.doesNotMatch(code, /fn\s+len\s+<\(SegmentTree\)->i32>/, 'SegmentTree.len must not consume the owner');
assert.match(code, /fn\s+replace\s+<\(SegmentTree,i32,i32\)\*>Result<SegmentTree,\s*SegmentTreeUpdateError>>\s+\(st,\s*idx,\s*value\):/, 'SegmentTree.replace must return an owner-carrying error type');
assert.match(code, /fn\s+add\s+<\(SegmentTree,i32,i32\)\*>Result<SegmentTree,\s*SegmentTreeUpdateError>>\s+\(st,\s*idx,\s*delta\):/, 'SegmentTree.add must return an owner-carrying error type');
assert.doesNotMatch(code, /fn\s+replace\s+<\(SegmentTree,i32,i32\)\*>Result<SegmentTree,\s*Diag>>/, 'SegmentTree.replace must not lose the owner through Err(Diag)');
assert.doesNotMatch(code, /fn\s+add\s+<\(SegmentTree,i32,i32\)\*>Result<SegmentTree,\s*Diag>>/, 'SegmentTree.add must not lose the owner through Err(Diag)');
assert.match(code, /let\s+e\s+<SegmentTreeUpdateError>\s+SegmentTreeUpdateError\s+st\s+d[\s\S]*err<SegmentTree,\s*SegmentTreeUpdateError>\s+e/, 'SegmentTree update Err paths must return the input owner in SegmentTreeUpdateError');
assert.match(code, /fn\s+free\s+<\(SegmentTree\)->\(\)>\s+\(st\):[\s\S]*dealloc_raw\s+mem_ptr_addr\s+data\s+mul\s+mul\s+base\s+2\s+4/, 'SegmentTree.free must use raw owner cleanup for tree storage');
assert.match(code, /fn\s+free\s+<\(SegmentTree\)->\(\)>\s+\(st\):[\s\S]*field::get\s+st\s+"data"/, 'SegmentTree.free must consume the data owner field');
assert.doesNotMatch(code, /fn\s+free\s+<\(SegmentTree\)->\(\)>\s+\(st\):[\s\S]*field::get_ref\s+&st\s+"data"/, 'SegmentTree.free must not borrow-read the data owner field');
assert.doesNotMatch(code, /dealloc_ptr/, 'SegmentTree must not use checked deallocation for owned internals');

console.log('segment tree unsafe unwrap regression passed');
