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

assert.match(code, /fn\s+dsu_store_owned\s+/, 'DisjointSet must centralize owned array raw stores');
assert.match(code, /fn\s+dsu_load_owned\s+/, 'DisjointSet must centralize owned array loads');
assert.match(code, /eq\s+n\s+0[\s\S]*ok<DisjointSet,\s*Diag>\s+DisjointSet\s+0\s+mem_ptr_wrap\s+0\s+mem_ptr_wrap\s+0/, 'DisjointSet.new must treat zero length as an empty owned set without allocating zero bytes');
assert.match(code, /fn\s+free\s+<\(DisjointSet\)->\(\)>\s+\(dsu\):[\s\S]*dealloc_raw\s+mem_ptr_addr\s+parent[\s\S]*dealloc_raw\s+mem_ptr_addr\s+sizes/, 'DisjointSet.free must use raw owner cleanup for parent and size storage');
assert.match(code, /fn\s+free\s+<\(DisjointSet\)->\(\)>\s+\(dsu\):[\s\S]*field::get\s+dsu\s+"parent"[\s\S]*field::get\s+dsu\s+"sizes"/, 'DisjointSet.free must consume parent and size owner fields');
assert.doesNotMatch(code, /fn\s+free\s+<\(DisjointSet\)->\(\)>\s+\(dsu\):[\s\S]*field::get_ref\s+&dsu\s+"(?:parent|sizes)"/, 'DisjointSet.free must not borrow-read owned array fields');
assert.doesNotMatch(code, /dealloc_ptr/, 'DisjointSet must not use checked deallocation for owned internals');

console.log('disjoint set unsafe unwrap regression passed');
