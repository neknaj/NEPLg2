#!/usr/bin/env node

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');

const repoRoot = path.resolve(__dirname, '..');
const relPath = 'stdlib/alloc/collections/fenwick.nepl';
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

assert.match(code, /fn\s+fenwick_store_owned\s+/, 'Fenwick must centralize owned array writes');
assert.match(code, /fn\s+fenwick_load_owned\s+/, 'Fenwick must centralize owned array reads');
assert.match(code, /#import\s+"alloc\/collections\/vec"\s+as\s+vec/, 'Fenwick must use typed Vec storage');
assert.match(code, /struct\s+Fenwick:\s+[\s\S]*\bn\s+<i32>[\s\S]*\bbit\s+<Vec<i32>>/, 'Fenwick must store bit payload as a typed Vec<i32> owner');
assert.match(code, /fn\s+new\s+<\(i32\)\*>Result<Fenwick,\s*Diag>>[\s\S]*lt\s+n\s+0[\s\S]*fenwick_diag_len[\s\S]*vec::filled<i32>\s+bit_len\s+0/, 'Fenwick.new must reject negative lengths and allocate initialized typed storage through Vec.filled');
assert.match(code, /fn\s+fenwick_load_owned\s+<\(&Vec<i32>,i32\)->i32>[\s\S]*vec::get<i32>/, 'Fenwick must read tree cells through Vec.get');
assert.match(code, /fn\s+fenwick_store_owned\s+<\(&Vec<i32>,i32,i32\)\*>\(\)>[\s\S]*vec::replace<i32>/, 'Fenwick must update tree cells through Vec.replace');
assert.match(code, /fn\s+free\s+<\(Fenwick\)\*>\(\)>\s+\(fw\):[\s\S]*vec::free<i32>\s+bit/, 'Fenwick.free must close typed Vec<i32> storage');
assert.doesNotMatch(code, /\bMemPtr\b/, 'Fenwick must not expose raw MemPtr storage');
assert.doesNotMatch(code, /\bmem_ptr_wrap\b/, 'Fenwick must not use raw pointer arithmetic');
assert.doesNotMatch(code, /\bmem_ptr_addr\b/, 'Fenwick must not recover raw addresses for bit storage');
assert.doesNotMatch(code, /\balloc_ptr\b/, 'Fenwick must not allocate bit storage through raw pointer APIs');
assert.doesNotMatch(code, /\bload_i32\b/, 'Fenwick must not read bit storage through raw i32 loads');
assert.doesNotMatch(code, /\bstore_i32\b/, 'Fenwick must not write bit storage through raw i32 stores');
assert.doesNotMatch(code, /\bdealloc_raw\b/, 'Fenwick must not deallocate bit storage through raw pointer APIs');
assert.doesNotMatch(code, /\bdealloc_ptr\b/, 'Fenwick.free must not unwrap checked deallocation for owned storage');

console.log('fenwick unsafe unwrap regression passed');
