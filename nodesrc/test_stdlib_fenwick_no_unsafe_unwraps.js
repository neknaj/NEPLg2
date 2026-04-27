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

assert.match(code, /fn\s+fenwick_store_owned\s+/, 'Fenwick must centralize owned array raw stores');
assert.match(code, /fn\s+fenwick_load_owned\s+/, 'Fenwick must centralize owned array raw loads');
assert.match(code, /fn\s+free\s+<\(Fenwick\)\*>\(\)>\s+\(fw\):[\s\S]*dealloc_raw\s+mem_ptr_addr\s+bit\s+bytes/, 'Fenwick.free must use raw owner cleanup for bit storage');
assert.doesNotMatch(code, /fn\s+free\s+<\(Fenwick\)\*>\(\)>\s+\(fw\):[\s\S]*dealloc_ptr/, 'Fenwick.free must not unwrap checked deallocation for owned storage');

console.log('fenwick unsafe unwrap regression passed');
