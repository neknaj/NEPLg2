#!/usr/bin/env node

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const { legacyTypeSyntaxView } = require('./source_policy/nepl_source_view');

const repoRoot = path.resolve(__dirname, '..');
const relPath = 'stdlib/std/stdio.nepl';
const src = fs.readFileSync(path.join(repoRoot, relPath), 'utf8');
const printRelPath = 'stdlib/std/stdio/print.nepl';
const printSrc = fs.readFileSync(path.join(repoRoot, printRelPath), 'utf8');

const code = legacyTypeSyntaxView(src);
const printCode = legacyTypeSyntaxView(printSrc);

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

const match = printCode.match(/(?:pub\s+)?fn\s+noshadow\s+print_i32\s+<\(i32\)\*>unit>\s+\(v\):([\s\S]*?)\n(?:pub\s+)?fn\s+noshadow\s+println_i32\s+/);
assert.ok(match, 'stdio print_i32 body must be found');

const body = match[1];
assert.match(
    body,
    /\bi32_decimal::i32_decimal_len\s+v\b/,
    'print_i32 must read decimal output length from alloc/string/integer/format/i32_decimal',
);
assert.match(
    body,
    /\bprint_byte\s+i32_decimal::i32_decimal_byte_at\s+v\s+idx\b/,
    'print_i32 must delegate digit byte generation to alloc/string/integer/format/i32_decimal',
);
assert.match(
    printCode,
    /#import\s+"alloc\/string\/integer\/format\/i32_decimal"\s+as\s+i32_decimal/,
    'stdio print_i32 must import the allocation-free integer decimal formatting submodule directly',
);
assert.match(
    printCode,
    /#import\s+"std\/stdio\/write\/byte"\s+as\s+\*/,
    'stdio print_i32 must obtain byte output through the stdio byte writer boundary',
);

const forbidden = [
    /\bstd_alloc\b/,
    /\bstd_free\b/,
    /\bstring_from_addr_unchecked\b/,
    /\bstore_u8\b/,
    /\bstore_i32\b/,
    /\bload_u8\b/,
    /\bdiv_s\b/,
    /\brem_s\b/,
    /\bstring_integer::from_i32\b/,
];

for (const pattern of forbidden) {
    assert.doesNotMatch(
        body,
        pattern,
        'print_i32 must not reintroduce a local digit formatter or raw-memory scratch formatter',
    );
}

console.log('stdlib stdio print_i32 boundary regression passed');
