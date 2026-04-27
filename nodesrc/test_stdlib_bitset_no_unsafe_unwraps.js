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

assert.match(code, /fn\s+free\s+<\(BitSet\)->\(\)>[\s\S]*dealloc_raw\s+mem_ptr_addr\s+bits\s+nbytes/, 'BitSet.free must use raw owner cleanup for bit storage');
assert.doesNotMatch(code, /fn\s+free\s+<\(BitSet\)->\(\)>[\s\S]*dealloc_ptr/, 'BitSet.free must not unwrap checked deallocation for owned storage');

console.log('bitset unsafe unwrap regression passed');
