#!/usr/bin/env node

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const { stripNeplComments, implementationLineCount } = require('./source_policy/stdlib_builder_owner');

const repoRoot = path.resolve(__dirname, '..');
const rootRelPath = 'stdlib/alloc/string.nepl';
const storageRelPath = 'stdlib/alloc/string/storage.nepl';
const rootSrc = fs.readFileSync(path.join(repoRoot, rootRelPath), 'utf8');
const storageSrc = fs.readFileSync(path.join(repoRoot, storageRelPath), 'utf8');
const rootCode = stripNeplComments(rootSrc);
const storageCode = stripNeplComments(storageSrc);

assert.doesNotMatch(rootSrc, /pub #import "\.\/string\/storage" as \*/, 'alloc/string facade must not re-export string/storage raw helpers');
assert.doesNotMatch(rootSrc, /pub #import "\.\/string\/utf8" as \*/, 'alloc/string facade must not re-export string/utf8 raw helpers');
assert.match(rootSrc, /通常利用者向けの安全な string API/, 'alloc/string facade must document the safe public surface');
assert.match(storageSrc, /#import "alloc\/string\/utf8" as \*/, 'string/storage must depend on UTF-8 validation instead of duplicating it');

for (const name of [
    'string_alloc_region',
    'string_region_len_ptr',
    'string_region_data_ptr',
    'string_addr',
    'string_data_ptr',
    'string_finish',
    'string_from_addr_unchecked',
    'string_finish_base',
    'string_from_mem_unchecked_result',
    'string_from_utf8_mem_result',
]) {
    assert.doesNotMatch(rootCode, new RegExp(`fn\\s+${name}\\b`), `${rootRelPath} must not own ${name}`);
    assert.match(storageCode, new RegExp(`fn\\s+${name}\\b`), `${storageRelPath} must own ${name}`);
}

assert.match(
    storageCode,
    /fn\s+string_from_utf8_mem_result[\s\S]*string_utf8_validate_mem[\s\S]*string_from_mem_unchecked_result/,
    'string_from_utf8_mem_result must validate raw bytes before unchecked string construction',
);
assert.match(
    storageCode,
    /fn\s+string_finish[\s\S]*get\s+region\s+"ptr"[\s\S]*string_finish_base\s+base\s+byte_len/,
    'string_finish must be the final RegionToken-to-str ownership boundary',
);
assert.match(
    storageCode,
    /fn\s+string_region_data_ptr\s+<\(&RegionToken<u8>\)->MemPtr<u8>>/,
    'string_region_data_ptr must project from a RegionToken reference',
);
assert.ok(implementationLineCount(storageSrc) <= 230, `${storageRelPath} should stay narrowly scoped`);

console.log('alloc/string storage boundary regression passed');
