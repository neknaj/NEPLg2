#!/usr/bin/env node

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const { stripNeplComments, implementationLineCount } = require('./source_policy/stdlib_builder_owner');

const repoRoot = path.resolve(__dirname, '..');
const rootRelPath = 'stdlib/alloc/string.nepl';
const searchRelPath = 'stdlib/alloc/string/search.nepl';
const compareRelPath = 'stdlib/alloc/string/search/compare.nepl';
const boundaryRelPath = 'stdlib/alloc/string/search/boundary.nepl';
const byteFindRelPath = 'stdlib/alloc/string/search/byte_find.nepl';
const rootSrc = fs.readFileSync(path.join(repoRoot, rootRelPath), 'utf8');
const searchSrc = fs.readFileSync(path.join(repoRoot, searchRelPath), 'utf8');
const compareSrc = fs.readFileSync(path.join(repoRoot, compareRelPath), 'utf8');
const boundarySrc = fs.readFileSync(path.join(repoRoot, boundaryRelPath), 'utf8');
const byteFindSrc = fs.readFileSync(path.join(repoRoot, byteFindRelPath), 'utf8');
const rootCode = stripNeplComments(rootSrc);
const searchCode = stripNeplComments(searchSrc);
const compareCode = stripNeplComments(compareSrc);
const boundaryCode = stripNeplComments(boundarySrc);
const byteFindCode = stripNeplComments(byteFindSrc);

assert.match(rootSrc, /pub #import "\.\/string\/search" as \*/, 'alloc/string facade must re-export string/search');
assert.match(searchSrc, /pub #import "\.\/search\/compare" as @merge/, 'string/search facade must merge compare helpers for qualified imports');
assert.match(searchSrc, /pub #import "\.\/search\/boundary" as @merge/, 'string/search facade must merge UTF-8 boundary helpers for qualified imports');
assert.match(searchSrc, /pub #import "\.\/search\/byte_find" as @merge/, 'string/search facade must merge byte find helpers for qualified imports');
assert.doesNotMatch(searchCode, /\bfn\s+/, 'string/search facade must not own implementation function bodies');
assert.match(compareSrc, /#import "alloc\/string\/access" as \*/, 'string/search/compare must use string/access byte readers');
assert.match(boundarySrc, /#import "alloc\/string\/access" as \*/, 'string/search/boundary must use string/access byte readers');
assert.match(boundarySrc, /#import "alloc\/string\/utf8" as \*/, 'string/search/boundary must use UTF-8 continuation classification');
assert.match(byteFindSrc, /#import "alloc\/string\/access" as \*/, 'string/search/byte_find must use string/access byte readers');

const ownerByName = new Map([
    ['str_eq', compareCode],
    ['str_eq_loop', compareCode],
    ['str_is_space', compareCode],
    ['str_starts_with', compareCode],
    ['str_eq_at', compareCode],
    ['str_starts_with_at', compareCode],
    ['str_ends_with', compareCode],
    ['str_range_eq', compareCode],
    ['str_utf8_is_boundary', boundaryCode],
    ['str_match_at', byteFindCode],
    ['str_find', byteFindCode],
]);

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
    assert.doesNotMatch(searchCode, new RegExp(`fn\\s+${name}\\b`), `${searchRelPath} facade must not own ${name}`);
    assert.match(ownerByName.get(name), new RegExp(`fn\\s+${name}\\b`), `string/search submodule must own ${name}`);
}

assert.match(
    compareCode,
    /fn\s+str_is_space[\s\S]*match\s+b:[\s\S]*' ':[\s\S]*'\\t':[\s\S]*'\\n':[\s\S]*'\\r':[\s\S]*_:/,
    'str_is_space must stay as char-literal match instead of nested if chains',
);
assert.match(
    boundaryCode,
    /fn\s+str_utf8_is_boundary[\s\S]*string_utf8_is_continuation\s+string_byte_at_unchecked\s+s\s+idx/,
    'str_utf8_is_boundary must classify continuation bytes through UTF-8 helpers',
);
assert.ok(implementationLineCount(searchSrc) <= 35, `${searchRelPath} should stay as a small facade`);
assert.ok(implementationLineCount(compareSrc) <= 270, `${compareRelPath} should stay narrowly scoped`);
assert.ok(implementationLineCount(boundarySrc) <= 75, `${boundaryRelPath} should stay narrowly scoped`);
assert.ok(implementationLineCount(byteFindSrc) <= 125, `${byteFindRelPath} should stay narrowly scoped`);

console.log('alloc/string search boundary regression passed');
