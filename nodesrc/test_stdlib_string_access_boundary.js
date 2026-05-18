#!/usr/bin/env node

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const { stripNeplComments, implementationLineCount } = require('./source_policy/stdlib_builder_owner');

const repoRoot = path.resolve(__dirname, '..');
const rootRelPath = 'stdlib/alloc/string.nepl';
const accessRelPath = 'stdlib/alloc/string/access.nepl';
const byteIndexRelPath = 'stdlib/alloc/string/byte_index.nepl';
const rootSrc = fs.readFileSync(path.join(repoRoot, rootRelPath), 'utf8');
const accessSrc = fs.readFileSync(path.join(repoRoot, accessRelPath), 'utf8');
const byteIndexSrc = fs.readFileSync(path.join(repoRoot, byteIndexRelPath), 'utf8');
const rootCode = stripNeplComments(rootSrc);
const accessCode = stripNeplComments(accessSrc);
const byteIndexCode = stripNeplComments(byteIndexSrc);

assert.match(rootSrc, /pub #import "\.\/string\/access" as \*/, 'alloc/string facade must re-export string/access');
assert.doesNotMatch(rootSrc, /pub #import "\.\/string\/byte_index" as \*/, 'alloc/string facade must not re-export raw string byte-index helpers');
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
assert.doesNotMatch(byteIndexCode, /\bpub\s+fn\s+string_byte_at_unchecked\b/, 'string/byte_index must not expose unchecked byte access');
assert.match(byteIndexCode, /\bstruct\s+StringByteIndex\b/, 'string/byte_index must define the checked byte-index witness');
assert.doesNotMatch(byteIndexCode, /\bpub\s+struct\s+StringByteIndex\b/, 'StringByteIndex constructor must stay private');
assert.match(byteIndexCode, /\bpub\s+fn\s+checked_string_byte_index\b[\s\S]*lt\s+idx\s+0[\s\S]*le\s+n\s+idx[\s\S]*some<StringByteIndex>\s+StringByteIndex\s+idx/, 'checked_string_byte_index must construct the witness only after bounds checks');
assert.match(byteIndexCode, /\bpub\s+fn\s+string_byte_at_checked\b/, 'string/byte_index must expose a witness-based byte reader');
assert.match(byteIndexCode, /\bfn\s+string_byte_access_addr\b/, 'string/byte_index must keep raw address projection private');
assert.doesNotMatch(byteIndexCode, /\bpub\s+fn\s+string_byte_access_addr\b/, 'string_byte_access_addr must not be public');
assert.match(
    accessCode,
    /fn\s+byte_at[\s\S]*lt\s+idx\s+0[\s\S]*le\s+n\s+idx[\s\S]*string_byte_at_checked_raw\s+s\s+idx/,
    'byte_at must bounds-check before delegating to private raw byte access',
);
assert.match(
    byteIndexCode,
    /fn\s+string_byte_at_checked[\s\S]*let\s+raw_idx\s+<i32>\s+string_byte_index_value\s+idx[\s\S]*load_u8\s+<i32>\s+add\s+string_byte_access_addr\s+s\s+<i32>\s+add\s+4\s+raw_idx/,
    'string_byte_at_checked must keep raw layout access behind the private byte-index witness',
);
assert.match(
    byteIndexCode,
    /fn\s+string_byte_at_checked_or_unreachable[\s\S]*match\s+checked_string_byte_index\s+s\s+idx:[\s\S]*Option::Some\s+proof:[\s\S]*string_byte_at_checked\s+s\s+proof[\s\S]*Option::None:[\s\S]*#intrinsic\s+"unreachable"/,
    'checked_or_unreachable must check before raw byte access',
);
assert.ok(implementationLineCount(accessSrc) <= 130, `${accessRelPath} should stay narrowly scoped`);
assert.ok(implementationLineCount(byteIndexSrc) <= 120, `${byteIndexRelPath} should stay narrowly scoped`);

console.log('alloc/string access boundary regression passed');
