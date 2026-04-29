#!/usr/bin/env node

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');

const repoRoot = path.resolve(__dirname, '..');

function read(rel) {
    return fs.readFileSync(path.join(repoRoot, rel), 'utf8');
}

const stringSrc = read('stdlib/alloc/string.nepl');
const lexerSrc = read('stdlib/neplg2/core/syntax/lexer.nepl');
const importSpecSrc = read('stdlib/neplg2/core/module/import_spec.nepl');

assert.match(
    stringSrc,
    /\bfn\s+str_starts_with_at\s+<\(str,i32,str\)->bool>/,
    'alloc/string.nepl must expose str_starts_with_at for offset-based scanners',
);

assert.match(
    stringSrc,
    /\bstr_eq_at\s+s\s+prefix\s+start\s+lp\s+0\b/,
    'str_starts_with_at must centralize the internal str_eq_at loop-index argument',
);

assert.doesNotMatch(
    lexerSrc,
    /\blet\s+ok_hash\b|\blet\s+ok_i\b|\blet\s+ok_n2\b/,
    'lexer must not hand-roll #indent byte comparisons',
);

assert.match(
    lexerSrc,
    /str_starts_with_at\s+source\s+start\s+directive/,
    'lex_starts_with_indent_directive must use str_starts_with_at',
);

assert.match(
    lexerSrc,
    /str_starts_with_at\s+source\s+start\s+"#if\[target="/,
    'lexer must use str_starts_with_at for #if[target=',
);

assert.match(
    lexerSrc,
    /str_starts_with_at\s+source\s+start\s+"#if\[profile="/,
    'lexer must use str_starts_with_at for #if[profile=',
);

assert.doesNotMatch(
    lexerSrc,
    /string::str_eq_at/,
    'self-host lexer must not call internal-style str_eq_at directly',
);

assert.match(
    importSpecSrc,
    /str_starts_with_at\s+s\s+idx\s+"as"/,
    'import spec parser must use str_starts_with_at for the as keyword',
);

console.log('selfhost string helper boundary regression passed');
