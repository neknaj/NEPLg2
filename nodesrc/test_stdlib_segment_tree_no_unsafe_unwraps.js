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
assert.match(code, /fn\s+free\s+<\(SegmentTree\)->\(\)>\s+\(st\):[\s\S]*dealloc_raw\s+mem_ptr_addr\s+data\s+mul\s+mul\s+base\s+2\s+4/, 'SegmentTree.free must use raw owner cleanup for tree storage');
assert.doesNotMatch(code, /dealloc_ptr/, 'SegmentTree must not use checked deallocation for owned internals');

console.log('segment tree unsafe unwrap regression passed');
