#!/usr/bin/env node

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');

const repoRoot = path.resolve(__dirname, '..');
const relPath = 'stdlib/alloc/collections/sparse_set.nepl';
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

assert.match(code, /fn\s+sparse_set_store_owned\s+/, 'SparseSet must centralize owned dense/sparse array raw stores');
assert.match(code, /fn\s+sparse_set_hdr_store_i32\s+/, 'SparseSet must centralize owned header raw stores');
assert.match(code, /eq\s+n\s+0[\s\S]*sparse_set_hdr_set_dense\s+hdr_raw\s+mem_ptr_wrap\s+0[\s\S]*sparse_set_hdr_set_sparse\s+hdr_raw\s+mem_ptr_wrap\s+0[\s\S]*ok<SparseSet,\s*Diag>\s+SparseSet\s+hdr_raw/, 'SparseSet.new must treat zero universe as an empty set without allocating zero-byte dense/sparse arrays');
assert.match(code, /fn\s+free\s+<\(SparseSet\)->\(\)>\s+\(s\):[\s\S]*dealloc_raw\s+mem_ptr_addr\s+dense[\s\S]*dealloc_raw\s+mem_ptr_addr\s+sparse[\s\S]*dealloc_raw\s+hdr\s+16/, 'SparseSet.free must use raw owner cleanup for dense, sparse, and header storage');
assert.doesNotMatch(code, /dealloc_ptr/, 'SparseSet must not use checked pointer deallocation for owned internals');
assert.doesNotMatch(code, /\bdealloc\b/, 'SparseSet must not use checked deallocation for owned internals');

console.log('sparse set unsafe unwrap regression passed');
