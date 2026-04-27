#!/usr/bin/env node

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');

const repoRoot = path.resolve(__dirname, '..');
const relPath = 'stdlib/alloc/collections/bloom_filter.nepl';
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

assert.match(code, /fn\s+free\s+<\.T,\.H>\s+<\(BloomFilter<\.T,\.H>\)->\(\)>[\s\S]*dealloc_raw\s+mem_ptr_addr\s+bits\s+nbytes/, 'BloomFilter.free must use raw owner cleanup for bit array storage');
assert.doesNotMatch(code, /fn\s+free\s+<\.T,\.H>\s+<\(BloomFilter<\.T,\.H>\)->\(\)>[\s\S]*dealloc_ptr/, 'BloomFilter.free must not unwrap checked deallocation for owned storage');

console.log('bloom filter unsafe unwrap regression passed');
