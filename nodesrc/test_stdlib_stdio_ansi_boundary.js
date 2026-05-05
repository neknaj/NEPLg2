#!/usr/bin/env node

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');

const repoRoot = path.resolve(__dirname, '..');
const rootRelPath = 'stdlib/std/stdio.nepl';
const ansiRelPath = 'stdlib/std/stdio/ansi.nepl';
const rootSrc = fs.readFileSync(path.join(repoRoot, rootRelPath), 'utf8');
const ansiSrc = fs.readFileSync(path.join(repoRoot, ansiRelPath), 'utf8');

const stripComments = (src) => src
    .split(/\r?\n/)
    .filter((line) => !/^\s*\/\//.test(line))
    .join('\n');

const rootCode = stripComments(rootSrc);
const ansiCode = stripComments(ansiSrc);

assert.match(
    rootCode,
    /pub\s+#import\s+"\.\/stdio\/ansi"\s+as\s+\*/,
    'std/stdio facade must re-export stdio ansi submodule',
);

for (const helper of [
    'ansi_reset',
    'ansi_red',
    'ansi_gray',
    'print_color',
    'println_color',
]) {
    assert.doesNotMatch(rootCode, new RegExp(`\\bfn\\s+${helper}\\b`), `${helper} must stay in stdio/ansi`);
    assert.match(ansiCode, new RegExp(`\\bfn\\s+${helper}\\b`), `${helper} must exist in stdio/ansi`);
}

const printColorMatch = ansiCode.match(
    /fn\s+print_color\s+<\(str,str\)\*>\(\)>\s+\(color,\s*s\):([\s\S]*?)\nfn\s+println_color\s+/,
);
assert.ok(printColorMatch, 'print_color body must be found');
assert.match(
    printColorMatch[1],
    /\bprint\s+color\b[\s\S]*\bprint\s+s\b[\s\S]*\bprint\s+ansi_reset\b/,
    'print_color must delegate output to stdio/print and reset color',
);

console.log('stdlib stdio ansi boundary regression passed');
