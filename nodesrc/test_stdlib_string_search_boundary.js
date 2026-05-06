#!/usr/bin/env node

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const { stripNeplComments } = require('./source_policy/stdlib_builder_owner');

const repoRoot = path.resolve(__dirname, '..');
const rootRelPath = 'stdlib/alloc/string.nepl';
const searchRelPath = 'stdlib/alloc/string/search.nepl';
const rootSrc = fs.readFileSync(path.join(repoRoot, rootRelPath), 'utf8');
const searchSrc = fs.readFileSync(path.join(repoRoot, searchRelPath), 'utf8');
const rootCode = stripNeplComments(rootSrc);
const searchCode = stripNeplComments(searchSrc);

assert.match(rootSrc, /pub #import "\.\/string\/search" as \*/, 'alloc/string facade must re-export string/search');
assert.match(searchSrc, /#import "alloc\/string\/access" as \*/, 'string/search must use string/access byte readers');
assert.match(searchSrc, /#import "alloc\/string\/utf8" as \*/, 'string/search must use UTF-8 continuation classification');

for (const name of [
    'str_eq',
    'str_eq_loop',
    'str_is_space',
    'str_starts_with',
    'str_eq_at',
    'str_starts_with_at',
    'str_ends_with',
    'str_utf8_is_boundary',
    'str_match_at',
    'str_find',
    'str_range_eq',
]) {
    assert.doesNotMatch(rootCode, new RegExp(`fn\\s+${name}\\b`), `${rootRelPath} must not own ${name}`);
    assert.match(searchCode, new RegExp(`fn\\s+${name}\\b`), `${searchRelPath} must own ${name}`);
}

assert.match(
    searchCode,
    /fn\s+str_is_space[\s\S]*match\s+b:[\s\S]*' ':[\s\S]*'\\t':[\s\S]*'\\n':[\s\S]*'\\r':[\s\S]*_:/,
    'str_is_space must stay as char-literal match instead of nested if chains',
);
assert.match(
    searchCode,
    /fn\s+str_utf8_is_boundary[\s\S]*string_utf8_is_continuation\s+string_byte_at_unchecked\s+s\s+idx/,
    'str_utf8_is_boundary must classify continuation bytes through UTF-8 helpers',
);
assert.ok(searchSrc.split(/\r?\n/).length <= 390, `${searchRelPath} should stay narrowly scoped`);

console.log('alloc/string search boundary regression passed');
