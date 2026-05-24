#!/usr/bin/env node

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const { implementationLineCount } = require('./source_policy/stdlib_builder_owner');
const { legacyTypeSyntaxView } = require('./source_policy/nepl_source_view');

const repoRoot = path.resolve(__dirname, '..');
const rootRelPath = 'stdlib/alloc/string.nepl';
const storageRelPath = 'stdlib/alloc/string/storage.nepl';
const rootSrc = fs.readFileSync(path.join(repoRoot, rootRelPath), 'utf8');
const storageSrc = fs.readFileSync(path.join(repoRoot, storageRelPath), 'utf8');
const rootCode = legacyTypeSyntaxView(rootSrc);
const storageCode = legacyTypeSyntaxView(storageSrc);

assert.doesNotMatch(rootSrc, /pub #import "\.\/string\/storage" as \*/, 'alloc/string facade must not re-export string/storage raw helpers');
assert.doesNotMatch(rootSrc, /pub #import "\.\/string\/utf8" as \*/, 'alloc/string facade must not re-export string/utf8 raw helpers');
assert.match(rootSrc, /通常利用者向けの安全な string API/, 'alloc/string facade must document the safe public surface');
assert.match(storageSrc, /#import "alloc\/string\/utf8" as \*/, 'string/storage must depend on UTF-8 validation instead of duplicating it');

for (const name of [
    'string_alloc_region',
    'string_region_data_ptr',
    'string_data_ptr',
    'string_finish',
    'string_from_mem_unchecked_result',
    'string_from_utf8_mem_result',
]) {
    assert.doesNotMatch(rootCode, new RegExp(`fn\\s+${name}\\b`), `${rootRelPath} must not own ${name}`);
    assert.match(storageCode, new RegExp(`pub\\s+fn\\s+${name}\\b`), `${storageRelPath} must publish ${name}`);
}

assert.match(storageCode, /\bfn\s+string_from_addr_unchecked\b/, `${storageRelPath} must own the final raw-address-to-str conversion`);
assert.doesNotMatch(storageCode, /\bpub\s+fn\s+string_from_addr_unchecked\b/, 'string_from_addr_unchecked must stay private to string_finish');
assert.match(storageCode, /\bfn\s+string_addr\b/, `${storageRelPath} must own str raw-address projection for string_data_ptr`);
assert.doesNotMatch(storageCode, /\bpub\s+fn\s+string_addr\b/, 'string_addr must stay private to string_data_ptr');
assert.doesNotMatch(storageCode, /\bfn\s+string_finish_base\b/, 'MemPtr-based string_finish_base must not be kept as a storage API');
assert.doesNotMatch(storageCode, /\bfn\s+string_region_len_ptr\b/, 'unused header pointer projection must not be exposed as a storage API');

assert.match(
    storageCode,
    /fn\s+string_from_utf8_mem_result[\s\S]*string_utf8_validate_mem[\s\S]*string_from_mem_unchecked_result/,
    'string_from_utf8_mem_result must validate raw bytes before unchecked string construction',
);
assert.match(
    storageCode,
    /fn\s+string_finish[\s\S]*let\s+base_raw\s+<i32>\s+get\s+region\s+"raw"[\s\S]*store_i32\s+base_raw\s+byte_len[\s\S]*string_from_addr_unchecked\s+base_raw/,
    'string_finish must directly consume the RegionToken raw owner at the final RegionToken-to-str ownership boundary',
);
assert.match(
    storageCode,
    /fn\s+string_region_data_ptr\s+<\(&RegionToken<u8>\)->MemPtr<u8>>/,
    'string_region_data_ptr must project from a RegionToken reference',
);
assert.ok(implementationLineCount(storageSrc) <= 230, `${storageRelPath} should stay narrowly scoped`);

console.log('alloc/string storage boundary regression passed');
