#!/usr/bin/env node

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');

const repoRoot = path.resolve(__dirname, '..');
const relPath = 'stdlib/alloc/collections/adjacency_matrix.nepl';
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

assert.match(code, /#import\s+"alloc\/collections\/vec"\s+as\s+vec/, 'AdjacencyMatrix must use typed Vec storage');
assert.match(code, /struct\s+AdjacencyMatrix:\s+[\s\S]*\bnverts\s+<i32>[\s\S]*\bnbytes\s+<i32>[\s\S]*\bbits\s+<Vec<u8>>/, 'AdjacencyMatrix must store matrix bytes as a typed Vec<u8> owner');
assert.match(code, /fn\s+adjacency_matrix_alloc_bits\s+<\(i32,i32\)\*>Result<Vec<u8>,\s*Diag>>[\s\S]*vec::filled<u8>\s+nbytes\s+byte/, 'AdjacencyMatrix must allocate initialized byte storage through Vec.filled');
assert.match(code, /fn\s+adjacency_matrix_byte_at\s+<\(&Vec<u8>,i32\)->Option<i32>>[\s\S]*vec::get<u8>[\s\S]*Option::None:[\s\S]*none<i32>/, 'AdjacencyMatrix must read matrix bytes through Vec.get and expose missing bytes as Option');
assert.match(code, /fn\s+adjacency_matrix_store_byte\s+<\(&Vec<u8>,i32,i32\)\*>\(\)>[\s\S]*vec::replace<u8>/, 'AdjacencyMatrix must update matrix bytes through Vec.replace');
assert.match(code, /fn\s+free\s+<\(AdjacencyMatrix\)->\(\)>[\s\S]*vec::free<u8>\s+bits/, 'AdjacencyMatrix.free must close typed Vec<u8> storage');
assert.doesNotMatch(code, /\bMemPtr\b/, 'AdjacencyMatrix must not expose raw MemPtr storage');
assert.doesNotMatch(code, /\bmem_ptr_wrap\b/, 'AdjacencyMatrix must not use raw pointer arithmetic');
assert.doesNotMatch(code, /\bmem_ptr_addr\b/, 'AdjacencyMatrix must not recover raw addresses for matrix storage');
assert.doesNotMatch(code, /\balloc_ptr\b/, 'AdjacencyMatrix must not allocate matrix storage through raw pointer APIs');
assert.doesNotMatch(code, /\bload_u8\b/, 'AdjacencyMatrix must not read matrix storage through raw byte loads');
assert.doesNotMatch(code, /\bstore_u8\b/, 'AdjacencyMatrix must not write matrix storage through raw byte stores');
assert.doesNotMatch(code, /\bdealloc_raw\b/, 'AdjacencyMatrix must not deallocate matrix storage through raw pointer APIs');
assert.doesNotMatch(code, /fn\s+free\s+<\(AdjacencyMatrix\)->\(\)>[\s\S]*dealloc_ptr/, 'AdjacencyMatrix.free must not unwrap checked deallocation for owned storage');

console.log('adjacency matrix unsafe unwrap regression passed');
