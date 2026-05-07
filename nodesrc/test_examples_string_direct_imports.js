#!/usr/bin/env node

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');

const repoRoot = path.resolve(__dirname, '..');

for (const relPath of [
    'examples/rpn.nepl',
    'examples/rpn_legacy.nepl',
    'examples/bf.nepl',
]) {
    const src = fs.readFileSync(path.join(repoRoot, relPath), 'utf8');

    assert.doesNotMatch(
        src,
        /#import\s+"alloc\/string"\s+as\s+\w+/,
        `${relPath} must not import the broad alloc/string facade as a qualified namespace`,
    );
    assert.doesNotMatch(
        src,
        /\bs::(?:len|byte_at|str_trim|str_slice_result|to_i32|from_i32|str_eq|concat3)\b/,
        `${relPath} must not call through the former broad string facade alias`,
    );
}

console.log('examples string direct import regression passed');
