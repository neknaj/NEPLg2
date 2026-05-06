#!/usr/bin/env node

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');

const repoRoot = path.resolve(__dirname, '..');
const relPath = 'stdlib/alloc/collections/bitset.nepl';
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

assert.match(code, /#import\s+"alloc\/collections\/vec"\s+as\s+vec/, 'BitSet must use typed Vec storage');
assert.match(code, /struct\s+BitSet:\s+[\s\S]*\bnbits\s+<i32>[\s\S]*\bnbytes\s+<i32>[\s\S]*\bbits\s+<Vec<u8>>/, 'BitSet must store bit payload as a typed Vec<u8> owner');
assert.match(code, /fn\s+bitset_alloc_bits\s+<\(i32,i32\)\*>Result<Vec<u8>,\s*Diag>>[\s\S]*vec::filled<u8>\s+nbytes\s+byte/, 'BitSet must allocate initialized typed byte storage through Vec.filled');
assert.match(code, /fn\s+bitset_byte_at\s+<\(&Vec<u8>,i32\)->i32>[\s\S]*vec::get<u8>/, 'BitSet must read bit bytes through Vec.get');
assert.match(code, /fn\s+bitset_store_byte\s+<\(&Vec<u8>,i32,i32\)\*>\(\)>[\s\S]*vec::replace<u8>/, 'BitSet must update bit bytes through Vec.replace');
assert.match(code, /fn\s+free\s+<\(BitSet\)->\(\)>[\s\S]*vec::free<u8>\s+bits/, 'BitSet.free must close typed Vec<u8> storage');
assert.doesNotMatch(code, /\bMemPtr\b/, 'BitSet must not expose raw MemPtr storage');
assert.doesNotMatch(code, /\bmem_ptr_wrap\b/, 'BitSet must not use raw pointer arithmetic');
assert.doesNotMatch(code, /\bmem_ptr_addr\b/, 'BitSet must not recover raw addresses for bit storage');
assert.doesNotMatch(code, /\balloc_ptr\b/, 'BitSet must not allocate bit storage through raw pointer APIs');
assert.doesNotMatch(code, /\bload_u8\b/, 'BitSet must not read bit storage through raw byte loads');
assert.doesNotMatch(code, /\bstore_u8\b/, 'BitSet must not write bit storage through raw byte stores');
assert.doesNotMatch(code, /\bdealloc_raw\b/, 'BitSet must not deallocate bit storage through raw pointer APIs');
assert.doesNotMatch(code, /\bdealloc_ptr\b/, 'BitSet.free must not unwrap checked deallocation for owned storage');

console.log('bitset unsafe unwrap regression passed');
