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

assert.match(code, /fn\s+seg_store_owned\s+/, 'SegmentTree must centralize owned array writes');
assert.match(code, /fn\s+seg_load_owned\s+/, 'SegmentTree must centralize owned array loads');
assert.match(code, /#import\s+"alloc\/collections\/vec"\s+as\s+vec/, 'SegmentTree must use typed Vec storage');
assert.match(code, /struct\s+SegmentTree:\s+[\s\S]*\bn\s+<i32>[\s\S]*\bbase\s+<i32>[\s\S]*\bdata\s+<Vec<i32>>/, 'SegmentTree must store tree payload as a typed Vec<i32> owner');
assert.match(code, /fn\s+new\s+<\(i32\)\*>Result<SegmentTree,\s*Diag>>[\s\S]*lt\s+n\s+0[\s\S]*seg_diag_len[\s\S]*vec::filled<i32>\s+cells\s+0/, 'SegmentTree.new must reject negative lengths and allocate initialized typed storage through Vec.filled');
assert.match(code, /fn\s+seg_load_owned\s+<\(&Vec<i32>,i32\)->i32>[\s\S]*vec::get<i32>/, 'SegmentTree must read tree cells through Vec.get');
assert.match(code, /fn\s+seg_store_owned\s+<\(&Vec<i32>,i32,i32\)\*>\(\)>[\s\S]*vec::replace<i32>/, 'SegmentTree must update tree cells through Vec.replace');
assert.match(code, /fn\s+free\s+<\(SegmentTree\)->\(\)>\s+\(st\):[\s\S]*vec::free<i32>\s+data/, 'SegmentTree.free must close typed Vec<i32> storage');
assert.match(code, /fn\s+free\s+<\(SegmentTree\)->\(\)>\s+\(st\):[\s\S]*field::get\s+st\s+"data"/, 'SegmentTree.free must consume the data owner field');
assert.doesNotMatch(code, /fn\s+free\s+<\(SegmentTree\)->\(\)>\s+\(st\):[\s\S]*field::get_ref\s+&st\s+"data"/, 'SegmentTree.free must not borrow-read the data owner field');
assert.doesNotMatch(code, /\bMemPtr\b/, 'SegmentTree must not expose raw MemPtr storage');
assert.doesNotMatch(code, /\bmem_ptr_wrap\b/, 'SegmentTree must not use raw pointer arithmetic');
assert.doesNotMatch(code, /\bmem_ptr_addr\b/, 'SegmentTree must not recover raw addresses for tree storage');
assert.doesNotMatch(code, /\balloc_ptr\b/, 'SegmentTree must not allocate tree storage through raw pointer APIs');
assert.doesNotMatch(code, /\bload_i32\b/, 'SegmentTree must not read tree storage through raw i32 loads');
assert.doesNotMatch(code, /\bstore_i32\b/, 'SegmentTree must not write tree storage through raw i32 stores');
assert.doesNotMatch(code, /\bdealloc_raw\b/, 'SegmentTree must not deallocate tree storage through raw pointer APIs');
assert.doesNotMatch(code, /dealloc_ptr/, 'SegmentTree must not use checked deallocation for owned internals');

console.log('segment tree unsafe unwrap regression passed');
