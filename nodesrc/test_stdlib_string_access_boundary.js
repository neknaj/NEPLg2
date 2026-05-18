#!/usr/bin/env node

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const { stripNeplComments, implementationLineCount } = require('./source_policy/stdlib_builder_owner');

const repoRoot = path.resolve(__dirname, '..');
const rootRelPath = 'stdlib/alloc/string.nepl';
const accessRelPath = 'stdlib/alloc/string/access.nepl';
const uncheckedAccessRelPath = 'stdlib/alloc/string/unchecked_access.nepl';
const rootSrc = fs.readFileSync(path.join(repoRoot, rootRelPath), 'utf8');
const accessSrc = fs.readFileSync(path.join(repoRoot, accessRelPath), 'utf8');
const uncheckedAccessSrc = fs.readFileSync(path.join(repoRoot, uncheckedAccessRelPath), 'utf8');
const rootCode = stripNeplComments(rootSrc);
const accessCode = stripNeplComments(accessSrc);
const uncheckedAccessCode = stripNeplComments(uncheckedAccessSrc);

assert.match(rootSrc, /pub #import "\.\/string\/access" as \*/, 'alloc/string facade must re-export string/access');
assert.doesNotMatch(rootSrc, /pub #import "\.\/string\/unchecked_access" as \*/, 'alloc/string facade must not re-export unchecked string byte access');
assert.doesNotMatch(accessSrc, /#import "alloc\/string\/storage" as \*/, 'string/access must not depend on public storage raw-address helpers');
assert.match(accessCode, /\bfn\s+string_access_addr\b/, 'string/access must keep str raw-address projection private to the access module');
assert.doesNotMatch(accessCode, /\bpub\s+fn\s+string_access_addr\b/, 'string_access_addr must not be public');

for (const name of [
    'len',
    'str_byte_len',
    'byte_at',
]) {
    assert.doesNotMatch(rootCode, new RegExp(`fn\\s+${name}\\b`), `${rootRelPath} must not own ${name}`);
    assert.match(accessCode, new RegExp(`fn\\s+${name}\\b`), `${accessRelPath} must own ${name}`);
}

assert.doesNotMatch(accessCode, /\bpub\s+fn\s+string_byte_at_unchecked\b/, 'string/access must not expose unchecked byte access');
assert.match(uncheckedAccessCode, /\bpub\s+fn\s+string_byte_at_unchecked\b/, 'string/unchecked_access must own the explicit unchecked byte access boundary');
assert.match(uncheckedAccessCode, /\bfn\s+unchecked_string_access_addr\b/, 'string/unchecked_access must keep raw address projection private');
assert.doesNotMatch(uncheckedAccessCode, /\bpub\s+fn\s+unchecked_string_access_addr\b/, 'unchecked_string_access_addr must not be public');
assert.match(
    accessCode,
    /fn\s+byte_at[\s\S]*lt\s+idx\s+0[\s\S]*le\s+n\s+idx[\s\S]*string_byte_at_checked_raw\s+s\s+idx/,
    'byte_at must bounds-check before delegating to private raw byte access',
);
assert.match(
    uncheckedAccessCode,
    /fn\s+string_byte_at_unchecked[\s\S]*load_u8\s+<i32>\s+add\s+unchecked_string_access_addr\s+s\s+<i32>\s+add\s+4\s+idx/,
    'string_byte_at_unchecked must keep raw layout access isolated in string/unchecked_access',
);
assert.ok(implementationLineCount(accessSrc) <= 130, `${accessRelPath} should stay narrowly scoped`);
assert.ok(implementationLineCount(uncheckedAccessSrc) <= 80, `${uncheckedAccessRelPath} should stay narrowly scoped`);

console.log('alloc/string access boundary regression passed');
