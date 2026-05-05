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
    'ansi_color_code',
    'ansi_text_weight_code',
    'ansi_text_decoration_code',
    'ansi_reset_code',
    'print_style_start',
    'print_style_reset',
    'print_style',
    'println_style',
    'print_color',
    'println_color',
]) {
    assert.doesNotMatch(rootCode, new RegExp(`\\bfn\\s+${helper}\\b`), `${helper} must stay in stdio/ansi`);
    assert.match(ansiCode, new RegExp(`\\bfn\\s+${helper}\\b`), `${helper} must exist in stdio/ansi`);
}

assert.match(
    ansiCode,
    /enum\s+AnsiColor:[\s\S]*Default[\s\S]*Red[\s\S]*Green[\s\S]*Yellow[\s\S]*Blue[\s\S]*Magenta[\s\S]*Cyan[\s\S]*White[\s\S]*Gray/,
    'stdio/ansi must model colors as an enum',
);
assert.match(ansiCode, /enum\s+AnsiTextWeight:[\s\S]*Regular[\s\S]*Bold/, 'stdio/ansi must model text weight as an enum');
assert.match(ansiCode, /enum\s+AnsiTextDecoration:[\s\S]*Plain[\s\S]*Underline/, 'stdio/ansi must model text decoration as an enum');
assert.match(
    ansiCode,
    /struct\s+AnsiTextStyle:[\s\S]*color\s+<AnsiColor>[\s\S]*weight\s+<AnsiTextWeight>[\s\S]*decoration\s+<AnsiTextDecoration>/,
    'stdio/ansi must model composite text style as typed fields',
);

const styleCodeMatch = ansiCode.match(
    /fn\s+ansi_color_code\s+<\(AnsiColor\)->str>\s+\(color\):([\s\S]*?)\nfn\s+ansi_text_weight_code\s+/,
);
assert.ok(styleCodeMatch, 'ansi_color_code body must be found');
assert.doesNotMatch(styleCodeMatch[1], /\n\s*_:/, 'ansi_color_code must not use wildcard fallback');
for (const variant of [
    'Default',
    'Red',
    'Green',
    'Yellow',
    'Blue',
    'Magenta',
    'Cyan',
    'White',
    'Gray',
]) {
    assert.match(styleCodeMatch[1], new RegExp(`AnsiColor::${variant}:`), `ansi_color_code must cover ${variant}`);
}

const weightCodeMatch = ansiCode.match(
    /fn\s+ansi_text_weight_code\s+<\(AnsiTextWeight\)->str>\s+\(weight\):([\s\S]*?)\nfn\s+ansi_text_decoration_code\s+/,
);
assert.ok(weightCodeMatch, 'ansi_text_weight_code body must be found');
assert.doesNotMatch(weightCodeMatch[1], /\n\s*_:/, 'ansi_text_weight_code must not use wildcard fallback');
for (const variant of ['Regular', 'Bold']) {
    assert.match(weightCodeMatch[1], new RegExp(`AnsiTextWeight::${variant}:`), `ansi_text_weight_code must cover ${variant}`);
}

const decorationCodeMatch = ansiCode.match(
    /fn\s+ansi_text_decoration_code\s+<\(AnsiTextDecoration\)->str>\s+\(decoration\):([\s\S]*?)\nfn\s+ansi_reset_code\s+/,
);
assert.ok(decorationCodeMatch, 'ansi_text_decoration_code body must be found');
assert.doesNotMatch(decorationCodeMatch[1], /\n\s*_:/, 'ansi_text_decoration_code must not use wildcard fallback');
for (const variant of ['Plain', 'Underline']) {
    assert.match(decorationCodeMatch[1], new RegExp(`AnsiTextDecoration::${variant}:`), `ansi_text_decoration_code must cover ${variant}`);
}

for (const obsolete of [
    'ansi_red',
    'ansi_green',
    'ansi_bold',
    'ansi_reset',
    'AnsiStyle',
]) {
    assert.doesNotMatch(ansiCode, new RegExp(`\\b${obsolete}\\b`), `${obsolete} raw or single-enum facade must not be reintroduced`);
}

const printStyleMatch = ansiCode.match(
    /fn\s+print_style\s+<\(AnsiTextStyle,str\)\*>\(\)>\s+\(style,\s*s\):([\s\S]*?)\nfn\s+println_style\s+/,
);
assert.ok(printStyleMatch, 'print_style body must be found');
assert.match(
    printStyleMatch[1],
    /\bprint_style_start\s+style\b[\s\S]*\bprint\s+s\b[\s\S]*\bprint_style_reset\b/,
    'print_style must use typed style start and reset helpers',
);

assert.match(
    ansiCode,
    /fn\s+print_color\s+<\(AnsiColor,str\)\*>\(\)>\s+\(color,\s*s\):[\s\S]*print_style\s+ansi_color_style\s+color\s+s/,
    'print_color must be a typed AnsiColor convenience wrapper',
);

console.log('stdlib stdio ansi boundary regression passed');
