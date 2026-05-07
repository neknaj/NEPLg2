#!/usr/bin/env node

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');

const repoRoot = path.resolve(__dirname, '..');
const relPaths = [
    'stdlib/alloc/hash/hash32.nepl',
    'stdlib/core/traits/hash.nepl',
    'stdlib/tests/hash.n.md',
];

for (const relPath of relPaths) {
    const src = fs.readFileSync(path.join(repoRoot, relPath), 'utf8');
    assert.match(
        src,
        /#import\s+"alloc\/string\/access"\s+as\s+string/,
        `${relPath} must import the string access implementation for qualified byte reads`,
    );
    assert.doesNotMatch(
        src,
        /#import\s+"alloc\/string"\s+as\s+string/,
        `${relPath} must not rely on qualified re-export through the broad alloc/string facade`,
    );
}

console.log('stdlib hash string access boundary regression passed');
