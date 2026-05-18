#!/usr/bin/env node

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const { stripNeplComments, implementationLineCount } = require('./source_policy/stdlib_builder_owner');

const repoRoot = path.resolve(__dirname, '..');
const rootRelPath = 'stdlib/alloc/string.nepl';
const accessRelPath = 'stdlib/alloc/string/access.nepl';
const rootSrc = fs.readFileSync(path.join(repoRoot, rootRelPath), 'utf8');
const accessSrc = fs.readFileSync(path.join(repoRoot, accessRelPath), 'utf8');
const rootCode = stripNeplComments(rootSrc);
const accessCode = stripNeplComments(accessSrc);

assert.match(rootSrc, /pub #import "\.\/string\/access" as \*/, 'alloc/string facade must re-export string/access');
assert.doesNotMatch(accessSrc, /#import "alloc\/string\/storage" as \*/, 'string/access must not depend on public storage raw-address helpers');
assert.match(accessCode, /\bfn\s+string_access_addr\b/, 'string/access must keep str raw-address projection private to the access module');
assert.doesNotMatch(accessCode, /\bpub\s+fn\s+string_access_addr\b/, 'string_access_addr must not be public');

for (const name of [
    'len',
    'str_byte_len',
    'byte_at',
    'string_byte_at_unchecked',
]) {
    assert.doesNotMatch(rootCode, new RegExp(`fn\\s+${name}\\b`), `${rootRelPath} must not own ${name}`);
    assert.match(accessCode, new RegExp(`fn\\s+${name}\\b`), `${accessRelPath} must own ${name}`);
}

assert.match(
    accessCode,
    /fn\s+byte_at[\s\S]*lt\s+idx\s+0[\s\S]*le\s+n\s+idx[\s\S]*string_byte_at_unchecked\s+s\s+idx/,
    'byte_at must bounds-check before delegating to unchecked byte access',
);
assert.match(
    accessCode,
    /fn\s+string_byte_at_unchecked[\s\S]*load_u8\s+<i32>\s+add\s+string_access_addr\s+s\s+<i32>\s+add\s+4\s+idx/,
    'string_byte_at_unchecked must keep raw layout access isolated in string/access',
);
assert.ok(implementationLineCount(accessSrc) <= 130, `${accessRelPath} should stay narrowly scoped`);

console.log('alloc/string access boundary regression passed');
