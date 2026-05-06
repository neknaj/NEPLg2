#!/usr/bin/env node

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');

const repoRoot = path.resolve(__dirname, '..');
const relPath = 'stdlib/alloc/collections/disjoint_set.nepl';
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

assert.match(code, /fn\s+dsu_store_owned\s+/, 'DisjointSet must centralize owned array writes');
assert.match(code, /fn\s+dsu_load_owned\s+/, 'DisjointSet must centralize owned array loads');
assert.match(code, /#import\s+"alloc\/collections\/vec"\s+as\s+vec/, 'DisjointSet must use typed Vec storage');
assert.match(code, /struct\s+DisjointSet:\s+[\s\S]*\bn\s+<i32>[\s\S]*\bparent\s+<Vec<i32>>[\s\S]*\bsizes\s+<Vec<i32>>/, 'DisjointSet must store parent and size payloads as typed Vec<i32> owners');
assert.match(code, /fn\s+new\s+<\(i32\)\*>Result<DisjointSet,\s*Diag>>[\s\S]*lt\s+n\s+0[\s\S]*dsu_diag_len[\s\S]*vec::filled<i32>\s+n\s+0[\s\S]*vec::filled<i32>\s+n\s+1/, 'DisjointSet.new must reject negative lengths and allocate initialized typed storage through Vec.filled');
assert.doesNotMatch(code, /DisjointSet\s+0\s+mem_ptr_wrap\s+0\s+mem_ptr_wrap\s+0/, 'DisjointSet.new must not encode empty storage with null raw pointers');
assert.match(code, /fn\s+dsu_load_owned\s+<\(&Vec<i32>,i32\)->Option<i32>>[\s\S]*vec::get<i32>[\s\S]*Option::None:[\s\S]*none<i32>/, 'DisjointSet must read parent and size cells through Vec.get and expose missing cells as Option');
assert.match(code, /fn\s+dsu_store_owned\s+<\(&Vec<i32>,i32,i32\)\*>\(\)>[\s\S]*vec::replace<i32>/, 'DisjointSet must update parent and size cells through Vec.replace');
assert.match(code, /fn\s+free\s+<\(DisjointSet\)->\(\)>\s+\(dsu\):[\s\S]*vec::free<i32>\s+parent[\s\S]*vec::free<i32>\s+sizes/, 'DisjointSet.free must close typed Vec<i32> storage');
assert.match(code, /fn\s+free\s+<\(DisjointSet\)->\(\)>\s+\(dsu\):[\s\S]*field::get\s+dsu\s+"parent"[\s\S]*field::get\s+dsu\s+"sizes"/, 'DisjointSet.free must consume parent and size owner fields');
assert.doesNotMatch(code, /fn\s+free\s+<\(DisjointSet\)->\(\)>\s+\(dsu\):[\s\S]*field::get_ref\s+&dsu\s+"(?:parent|sizes)"/, 'DisjointSet.free must not borrow-read owned array fields');
assert.doesNotMatch(code, /\bMemPtr\b/, 'DisjointSet must not expose raw MemPtr storage');
assert.doesNotMatch(code, /\bmem_ptr_wrap\b/, 'DisjointSet must not use raw pointer sentinels');
assert.doesNotMatch(code, /\bmem_ptr_addr\b/, 'DisjointSet must not recover raw addresses for parent or size storage');
assert.doesNotMatch(code, /\balloc_ptr\b/, 'DisjointSet must not allocate storage through raw pointer APIs');
assert.doesNotMatch(code, /\bload_i32\b/, 'DisjointSet must not read storage through raw i32 loads');
assert.doesNotMatch(code, /\bstore_i32\b/, 'DisjointSet must not write storage through raw i32 stores');
assert.doesNotMatch(code, /\bdealloc_raw\b/, 'DisjointSet must not deallocate storage through raw pointer APIs');
assert.doesNotMatch(code, /dealloc_ptr/, 'DisjointSet must not use checked deallocation for owned internals');

console.log('disjoint set unsafe unwrap regression passed');
