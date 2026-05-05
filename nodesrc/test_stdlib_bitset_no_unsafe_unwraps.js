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
    /\bMemPtr\b/,
    /\balloc_ptr\b/,
    /\balloc_raw\b/,
    /\bdealloc_raw\b/,
    /\bdealloc_ptr\b/,
    /\bload_u8\b/,
    /\bstore_u8\b/,
    /\bmem_ptr_addr\b/,
    /\bmem_ptr_wrap\b/,
];

for (const pattern of forbidden) {
    assert.doesNotMatch(code, pattern, `${relPath} must not use unsafe unwrap or raw storage helpers in implementation code`);
}

assert.match(code, /#import\s+"alloc\/collections\/vec"\s+as\s+vec/, 'BitSet must use the typed Vec storage boundary');
assert.match(code, /struct\s+BitSet:[\s\S]*nbits\s+<i32>[\s\S]*nbytes\s+<i32>[\s\S]*bits\s+<Vec<i32>>/, 'BitSet must expose typed Vec<i32> byte storage as an owner field');
assert.match(code, /fn\s+bitset_byte_at\s+<\(&Vec<i32>,i32\)->Option<i32>>[\s\S]*vec::get<i32>\s+bits\s+bitset_byte_index\s+bit_idx/, 'BitSet reads must go through typed Vec get');
assert.match(code, /fn\s+bitset_store_byte\s+<\(&Vec<i32>,i32,i32\)\*>\(\)>[\s\S]*vec::replace<i32>\s+bits\s+bitset_byte_index\s+bit_idx\s+byte_value/, 'BitSet writes must go through typed Vec replace');
assert.match(code, /fn\s+new\s+<\(i32\)\*>Result<BitSet,\s*Diag>>[\s\S]*match\s+vec::filled<i32>\s+nbytes\s+0:/, 'BitSet allocation must initialize typed storage through Vec.filled');
assert.match(code, /fn\s+free\s+<\(BitSet\)->\(\)>[\s\S]*let\s+bits\s+<Vec<i32>>\s+field::get\s+bs\s+"bits"[\s\S]*vec::free<i32>\s+bits/, 'BitSet.free must close the typed Vec owner');

console.log('bitset unsafe unwrap regression passed');
