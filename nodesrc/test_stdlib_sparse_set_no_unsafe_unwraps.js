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
assert.match(code, /struct\s+SparseSet:\s+[\s\S]*\bn\s+<i32>[\s\S]*\blen0\s+<i32>[\s\S]*\bdense\s+<MemPtr<i32>>[\s\S]*\bsparse\s+<MemPtr<i32>>/, 'SparseSet must expose typed storage fields instead of an opaque raw header');
assert.match(code, /eq\s+n\s+0[\s\S]*ok<SparseSet,\s*Diag>\s+SparseSet\s+0\s+0\s+mem_ptr_wrap\s+0\s+mem_ptr_wrap\s+0/, 'SparseSet.new must treat zero universe as an empty set without allocating zero-byte dense/sparse arrays');
assert.match(code, /fn\s+sparse_set_free_arrays\s+<\(i32,MemPtr<i32>,MemPtr<i32>\)->\(\)>\s+\(n,\s*dense,\s*sparse\):[\s\S]*lt\s+0\s+n[\s\S]*dealloc_raw\s+mem_ptr_addr\s+dense[\s\S]*dealloc_raw\s+mem_ptr_addr\s+sparse/, 'SparseSet must centralize allocated dense/sparse owner cleanup');
assert.match(code, /fn\s+free\s+<\(SparseSet\)->\(\)>\s+\(s\):[\s\S]*sparse_set_free_arrays\s+n\s+dense\s+sparse/, 'SparseSet.free must close dense and sparse storage through the cleanup helper');
assert.match(code, /fn\s+insert\s+<\(SparseSet,i32\)\*>Result<SparseSet,\s*Diag>>[\s\S]*not\s+sparse_set_valid_index\s+value\s+n[\s\S]*sparse_set_free_arrays\s+n\s+dense\s+sparse[\s\S]*err<SparseSet,\s*Diag>/, 'SparseSet.insert must clean up the consumed owner on invalid index errors');
assert.match(code, /fn\s+remove\s+<\(SparseSet,i32\)\*>Result<SparseSet,\s*Diag>>[\s\S]*not\s+sparse_set_valid_index\s+value\s+n[\s\S]*sparse_set_free_arrays\s+n\s+dense\s+sparse[\s\S]*err<SparseSet,\s*Diag>/, 'SparseSet.remove must clean up the consumed owner on invalid index errors');
assert.doesNotMatch(code, /\bhdr\s+<i32>/, 'SparseSet must not store dense/sparse owners behind an opaque raw header');
assert.doesNotMatch(code, /sparse_set_hdr_/, 'SparseSet must not reintroduce raw header helpers');
assert.doesNotMatch(code, /alloc_ptr<u8>\s+16/, 'SparseSet must not allocate a raw header block');
assert.doesNotMatch(code, /dealloc_raw\s+hdr\s+16/, 'SparseSet must not deallocate a raw header block');
assert.doesNotMatch(code, /dealloc_ptr/, 'SparseSet must not use checked pointer deallocation for owned internals');
assert.doesNotMatch(code, /\bdealloc\b/, 'SparseSet must not use checked deallocation for owned internals');

console.log('sparse set unsafe unwrap regression passed');
