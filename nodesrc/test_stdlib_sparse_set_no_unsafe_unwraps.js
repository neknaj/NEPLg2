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

assert.match(code, /#import\s+"alloc\/collections\/vec"\s+as\s+vec/, 'SparseSet must use the typed Vec storage module');
assert.match(code, /struct\s+SparseSet:\s+[\s\S]*\bn\s+<i32>[\s\S]*\blen0\s+<i32>[\s\S]*\bdense\s+<Vec<i32>>[\s\S]*\bsparse\s+<Vec<i32>>/, 'SparseSet must store dense/sparse payloads as typed Vec<i32> owners');
assert.match(code, /fn\s+sparse_set_alloc_array\s+<\(i32\)\*>Result<Vec<i32>,\s*Diag>>[\s\S]*vec::filled<i32>\s+n\s+0/, 'SparseSet must allocate dense/sparse arrays through typed Vec initialization');
assert.match(code, /fn\s+sparse_set_store_owned\s+<\(&Vec<i32>,i32,i32\)\*>\(\)>\s+\(base,\s*idx,\s*value\):[\s\S]*vec::replace<i32>\s+base\s+idx\s+value/, 'SparseSet must update dense/sparse slots through Vec.replace');
assert.match(code, /fn\s+sparse_set_free_arrays\s+<\(Vec<i32>,Vec<i32>\)->\(\)>\s+\(dense,\s*sparse\):[\s\S]*vec::free<i32>\s+dense[\s\S]*vec::free<i32>\s+sparse/, 'SparseSet must centralize Vec owner cleanup for dense/sparse storage');
assert.match(code, /fn\s+free\s+<\(SparseSet\)->\(\)>\s+\(s\):[\s\S]*sparse_set_free_arrays\s+dense\s+sparse/, 'SparseSet.free must close dense and sparse storage through the cleanup helper');
assert.match(code, /fn\s+insert\s+<\(SparseSet,i32\)\*>Result<SparseSet,\s*Diag>>[\s\S]*not\s+sparse_set_valid_index\s+value\s+n[\s\S]*sparse_set_free_arrays\s+dense\s+sparse[\s\S]*err<SparseSet,\s*Diag>/, 'SparseSet.insert must clean up the consumed owner on invalid index errors');
assert.match(code, /fn\s+remove\s+<\(SparseSet,i32\)\*>Result<SparseSet,\s*Diag>>[\s\S]*not\s+sparse_set_valid_index\s+value\s+n[\s\S]*sparse_set_free_arrays\s+dense\s+sparse[\s\S]*err<SparseSet,\s*Diag>/, 'SparseSet.remove must clean up the consumed owner on invalid index errors');
assert.doesNotMatch(code, /\bhdr\s+<i32>/, 'SparseSet must not store dense/sparse owners behind an opaque raw header');
assert.doesNotMatch(code, /sparse_set_hdr_/, 'SparseSet must not reintroduce raw header helpers');
assert.doesNotMatch(code, /\bMemPtr\b/, 'SparseSet must not expose raw MemPtr storage');
assert.doesNotMatch(code, /\bmem_ptr_wrap\b/, 'SparseSet must not use a null pointer sentinel for empty storage');
assert.doesNotMatch(code, /\bmem_ptr_addr\b/, 'SparseSet must not recover raw addresses for dense/sparse storage');
assert.doesNotMatch(code, /\balloc_ptr\b/, 'SparseSet must not allocate dense/sparse storage through raw pointer APIs');
assert.doesNotMatch(code, /\bload_i32\b/, 'SparseSet must not read dense/sparse storage through raw i32 loads');
assert.doesNotMatch(code, /\bstore_i32\b/, 'SparseSet must not write dense/sparse storage through raw i32 stores');
assert.doesNotMatch(code, /alloc_ptr<u8>\s+16/, 'SparseSet must not allocate a raw header block');
assert.doesNotMatch(code, /dealloc_raw\s+hdr\s+16/, 'SparseSet must not deallocate a raw header block');
assert.doesNotMatch(code, /dealloc_ptr/, 'SparseSet must not use checked pointer deallocation for owned internals');
assert.doesNotMatch(code, /dealloc_raw/, 'SparseSet must not deallocate dense/sparse storage through raw pointer APIs');
assert.doesNotMatch(code, /\bdealloc\b/, 'SparseSet must not use checked deallocation for owned internals');

console.log('sparse set unsafe unwrap regression passed');
