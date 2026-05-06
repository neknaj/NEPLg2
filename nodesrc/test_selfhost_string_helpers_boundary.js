#!/usr/bin/env node

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');

const repoRoot = path.resolve(__dirname, '..');

function read(rel) {
    return fs.readFileSync(path.join(repoRoot, rel), 'utf8');
}

const stringSrc = read('stdlib/alloc/string.nepl');
const stringSearchSrc = read('stdlib/alloc/string/search.nepl');
const lexerSrc = read('stdlib/neplg2/core/syntax/lexer.nepl');
const importSpecSrc = read('stdlib/neplg2/core/module/import_spec.nepl');

assert.match(
    stringSrc,
    /pub\s+#import\s+"\.\/string\/search"\s+as\s+\*/,
    'alloc/string.nepl must re-export string/search for offset-based scanners',
);

assert.match(
    stringSearchSrc,
    /\bfn\s+str_starts_with_at\s+<\(str,i32,str\)->bool>/,
    'alloc/string/search.nepl must own str_starts_with_at for offset-based scanners',
);

assert.match(
    stringSearchSrc,
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
    lexerSrc,
    /fn\s+lex_stack_drop_top[\s\S]*let\s+stack_storage\s+<VecStorageState>[\s\S]*let\s+stack_data\s+<MemPtr<i32>>\s+field::get\s+stack\s+"data"[\s\S]*Vec<i32>\s+sub\s+stack_len\s+1\s+stack_cap\s+stack_storage\s+stack_data/,
    'lex_stack_drop_top must preserve Vec storage state and move the data owner into the returned Vec',
);

assert.doesNotMatch(
    lexerSrc,
    /Vec<i32>\s+sub\s+stack_len\s+1\s+stack_cap\s+stack_data/,
    'lex_stack_drop_top must not use the obsolete four-field Vec constructor',
);

assert.match(
    importSpecSrc,
    /str_starts_with_at\s+s\s+idx\s+"as"/,
    'import spec parser must use str_starts_with_at for the as keyword',
);

assert.match(
    importSpecSrc,
    /fn\s+selfhost_import_spec_free[\s\S]*field::get\s+spec\s+"path"[\s\S]*field::get\s+spec\s+"alias"/,
    'import spec parser must keep an explicit cleanup helper for parsed path and alias string owners',
);

console.log('selfhost string helper boundary regression passed');
