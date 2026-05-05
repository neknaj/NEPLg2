#!/usr/bin/env node

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');

const repoRoot = path.resolve(__dirname, '..');
const rootRelPath = 'stdlib/std/stdio.nepl';
const debugRelPath = 'stdlib/std/stdio/debug.nepl';
const rootSrc = fs.readFileSync(path.join(repoRoot, rootRelPath), 'utf8');
const debugSrc = fs.readFileSync(path.join(repoRoot, debugRelPath), 'utf8');

const stripComments = (src) => src
    .split(/\r?\n/)
    .filter((line) => !/^\s*\/\//.test(line))
    .join('\n');

const rootCode = stripComments(rootSrc);
const debugCode = stripComments(debugSrc);

assert.match(
    rootCode,
    /pub\s+#import\s+"\.\/stdio\/debug"\s+as\s+\*/,
    'std/stdio facade must re-export stdio debug submodule',
);

for (const helper of [
    'debug',
    'debug_color',
    'debugln',
    'debugln_color',
]) {
    assert.doesNotMatch(rootCode, new RegExp(`\\bfn\\s+${helper}\\b`), `${helper} must stay in stdio/debug`);
    assert.match(debugCode, new RegExp(`\\bfn\\s+${helper}\\b`), `${helper} must exist in stdio/debug`);
}

assert.match(
    debugCode,
    /#if\[profile=debug\][\s\S]*fn\s+debug\s+<\(str\)\*>\(\)>\s+\(s\):[\s\S]*\bprint\s+s\b/,
    'debug profile debug must delegate to print',
);
assert.match(
    debugCode,
    /#if\[profile=release\][\s\S]*fn\s+debug\s+<\(str\)\*>\(\)>\s+\(_s\):[\s\S]*\(\)/,
    'release profile debug must stay no-op',
);

console.log('stdlib stdio debug boundary regression passed');
