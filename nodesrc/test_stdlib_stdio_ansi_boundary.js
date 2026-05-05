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
    'ansi_style_code',
    'ansi_reset_code',
    'print_style',
    'println_style',
]) {
    assert.doesNotMatch(rootCode, new RegExp(`\\bfn\\s+${helper}\\b`), `${helper} must stay in stdio/ansi`);
    assert.match(ansiCode, new RegExp(`\\bfn\\s+${helper}\\b`), `${helper} must exist in stdio/ansi`);
}

assert.match(
    ansiCode,
    /enum\s+AnsiStyle:[\s\S]*Reset[\s\S]*Bold[\s\S]*Underline[\s\S]*Red[\s\S]*Green[\s\S]*Yellow[\s\S]*Blue[\s\S]*Magenta[\s\S]*Cyan[\s\S]*White[\s\S]*Gray/,
    'stdio/ansi must model styles as an enum',
);

const styleCodeMatch = ansiCode.match(
    /fn\s+ansi_style_code\s+<\(AnsiStyle\)->str>\s+\(style\):([\s\S]*?)\nfn\s+ansi_reset_code\s+/,
);
assert.ok(styleCodeMatch, 'ansi_style_code body must be found');
assert.doesNotMatch(styleCodeMatch[1], /\n\s*_:/, 'ansi_style_code must not use wildcard fallback');
for (const variant of [
    'Reset',
    'Bold',
    'Underline',
    'Red',
    'Green',
    'Yellow',
    'Blue',
    'Magenta',
    'Cyan',
    'White',
    'Gray',
]) {
    assert.match(styleCodeMatch[1], new RegExp(`AnsiStyle::${variant}:`), `ansi_style_code must cover ${variant}`);
}

for (const obsolete of [
    'ansi_red',
    'ansi_green',
    'print_color',
    'println_color',
]) {
    assert.doesNotMatch(ansiCode, new RegExp(`\\bfn\\s+${obsolete}\\b`), `${obsolete} string facade must not be reintroduced`);
}

const printStyleMatch = ansiCode.match(
    /fn\s+print_style\s+<\(AnsiStyle,str\)\*>\(\)>\s+\(style,\s*s\):([\s\S]*?)\nfn\s+println_style\s+/,
);
assert.ok(printStyleMatch, 'print_style body must be found');
assert.match(
    printStyleMatch[1],
    /\bprint\s+ansi_style_code\s+style\b[\s\S]*\bprint\s+s\b[\s\S]*\bprint\s+ansi_reset_code\b/,
    'print_style must delegate output to stdio/print and reset style',
);

console.log('stdlib stdio ansi boundary regression passed');
