#!/usr/bin/env node

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');

const repoRoot = path.resolve(__dirname, '..');
const relPath = 'stdlib/alloc/collections/counting_bloom_filter.nepl';
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

assert.match(code, /#import\s+"alloc\/collections\/vec"\s+as\s+vec/, 'CountingBloomFilter must use typed Vec storage');
assert.match(code, /struct\s+CountingBloomFilter<\.T,\.H>:\s+[\s\S]*\bnslots\s+<i32>[\s\S]*\bcounters\s+<Vec<u8>>[\s\S]*\bhasher\s+<\.H>/, 'CountingBloomFilter must store counter bytes as a typed Vec<u8> owner');
assert.doesNotMatch(code, /\bnbytes\s+<i32>/, 'CountingBloomFilter must not keep a duplicate nbytes field for u8 counters');
assert.match(code, /fn\s+counting_bloom_alloc_counters\s+<\(i32,i32\)\*>Result<Vec<u8>,\s*Diag>>[\s\S]*vec::filled<u8>\s+nslots\s+counter/, 'CountingBloomFilter must allocate initialized counter storage through Vec.filled');
assert.match(code, /fn\s+counting_bloom_counter_at\s+<\(&Vec<u8>,i32\)->Option<i32>>[\s\S]*vec::get<u8>[\s\S]*Option::None:[\s\S]*none<i32>/, 'CountingBloomFilter must read counters through Vec.get and expose missing counters as Option');
assert.match(code, /fn\s+counting_bloom_store_counter\s+<\(&Vec<u8>,i32,i32\)\*>\(\)>[\s\S]*vec::replace<u8>/, 'CountingBloomFilter must update counters through Vec.replace');
assert.match(code, /fn\s+free\s+<\.T,\.H>\s+<\(CountingBloomFilter<\.T,\.H>\)->\(\)>[\s\S]*vec::free<u8>\s+counters/, 'CountingBloomFilter.free must close typed Vec<u8> storage');
assert.doesNotMatch(code, /\bMemPtr\b/, 'CountingBloomFilter must not expose raw MemPtr storage');
assert.doesNotMatch(code, /\bmem_ptr_wrap\b/, 'CountingBloomFilter must not use raw pointer arithmetic');
assert.doesNotMatch(code, /\bmem_ptr_addr\b/, 'CountingBloomFilter must not recover raw addresses for counter storage');
assert.doesNotMatch(code, /\balloc_ptr\b/, 'CountingBloomFilter must not allocate counter storage through raw pointer APIs');
assert.doesNotMatch(code, /\bload_u8\b/, 'CountingBloomFilter must not read counter storage through raw byte loads');
assert.doesNotMatch(code, /\bstore_u8\b/, 'CountingBloomFilter must not write counter storage through raw byte stores');
assert.doesNotMatch(code, /\bdealloc_raw\b/, 'CountingBloomFilter must not deallocate counter storage through raw pointer APIs');
assert.doesNotMatch(code, /fn\s+free\s+<\.T,\.H>\s+<\(CountingBloomFilter<\.T,\.H>\)->\(\)>[\s\S]*dealloc_ptr/, 'CountingBloomFilter.free must not unwrap checked deallocation for owned storage');

console.log('counting bloom filter unsafe unwrap regression passed');
