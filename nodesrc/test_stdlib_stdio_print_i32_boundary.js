#!/usr/bin/env node

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');

const repoRoot = path.resolve(__dirname, '..');
const relPath = 'stdlib/std/stdio.nepl';
const src = fs.readFileSync(path.join(repoRoot, relPath), 'utf8');
const printRelPath = 'stdlib/std/stdio/print.nepl';
const printSrc = fs.readFileSync(path.join(repoRoot, printRelPath), 'utf8');

const code = src
    .split(/\r?\n/)
    .filter((line) => !/^\s*\/\//.test(line))
    .join('\n');
const printCode = printSrc
    .split(/\r?\n/)
    .filter((line) => !/^\s*\/\//.test(line))
    .join('\n');

assert.match(
    code,
    /pub\s+#import\s+"\.\/stdio\/print"\s+as\s+\*/,
    'std/stdio facade must re-export stdio print submodule',
);
assert.doesNotMatch(
    code,
    /\bfn\s+noshadow\s+print_i32\b/,
    'print_i32 implementation must stay in stdio/print',
);

const match = printCode.match(/fn\s+noshadow\s+print_i32\s+<\(i32\)\*\>\(\)>\s+\(v\):([\s\S]*?)\nfn\s+noshadow\s+println_i32\s+/);
assert.ok(match, 'stdio print_i32 body must be found');

const body = match[1];
assert.match(
    body,
    /\bprint\s+string_integer::from_i32\s+v\b/,
    'print_i32 must delegate integer formatting to alloc/string/integer/format::from_i32',
);
assert.match(
    printCode,
    /#import\s+"alloc\/string\/integer\/format"\s+as\s+string_integer/,
    'stdio print_i32 must import the integer formatting module directly',
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
