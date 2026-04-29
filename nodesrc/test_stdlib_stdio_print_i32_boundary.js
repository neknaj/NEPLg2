#!/usr/bin/env node

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');

const repoRoot = path.resolve(__dirname, '..');
const relPath = 'stdlib/std/stdio.nepl';
const src = fs.readFileSync(path.join(repoRoot, relPath), 'utf8');

const code = src
    .split(/\r?\n/)
    .filter((line) => !/^\s*\/\//.test(line))
    .join('\n');

const match = code.match(/fn\s+noshadow\s+print_i32\s+<\(i32\)\*\>\(\)>\s+\(v\):([\s\S]*?)\nfn\s+noshadow\s+println_i32\s+/);
assert.ok(match, 'stdio print_i32 body must be found');

const body = match[1];
assert.match(
    body,
    /\bprint\s+string::from_i32\s+v\b/,
    'print_i32 must delegate integer formatting to alloc/string::from_i32',
);

const forbidden = [
    /\bstd_alloc\b/,
    /\bstd_free\b/,
    /\bstring_from_addr_unchecked\b/,
    /\bstore_u8\b/,
    /\bstore_i32\b/,
    /\bload_u8\b/,
    /\bwhile\b/,
];

for (const pattern of forbidden) {
    assert.doesNotMatch(
        body,
        pattern,
        'print_i32 must not reintroduce a local raw-memory scratch formatter',
    );
}

console.log('stdlib stdio print_i32 boundary regression passed');
