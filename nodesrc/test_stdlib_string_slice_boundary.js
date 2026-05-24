#!/usr/bin/env node

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const { implementationLineCount } = require('./source_policy/stdlib_builder_owner');
const { legacyTypeSyntaxView } = require('./source_policy/nepl_source_view');

const repoRoot = path.resolve(__dirname, '..');
const rootRelPath = 'stdlib/alloc/string.nepl';
const sliceRelPath = 'stdlib/alloc/string/slice.nepl';
const sliceByteRelPath = 'stdlib/alloc/string/slice/byte.nepl';
const sliceCharRelPath = 'stdlib/alloc/string/slice/char.nepl';
const sliceTrimRelPath = 'stdlib/alloc/string/slice/trim.nepl';
const charOffsetsRelPath = 'stdlib/alloc/string/char_offsets.nepl';
const rootSrc = fs.readFileSync(path.join(repoRoot, rootRelPath), 'utf8');
const sliceSrc = fs.readFileSync(path.join(repoRoot, sliceRelPath), 'utf8');
const sliceByteSrc = fs.readFileSync(path.join(repoRoot, sliceByteRelPath), 'utf8');
const sliceCharSrc = fs.readFileSync(path.join(repoRoot, sliceCharRelPath), 'utf8');
const sliceTrimSrc = fs.readFileSync(path.join(repoRoot, sliceTrimRelPath), 'utf8');
const charOffsetsSrc = fs.readFileSync(path.join(repoRoot, charOffsetsRelPath), 'utf8');
const rootCode = legacyTypeSyntaxView(rootSrc);
const sliceCode = legacyTypeSyntaxView(sliceSrc);
const sliceByteCode = legacyTypeSyntaxView(sliceByteSrc);
const sliceCharCode = legacyTypeSyntaxView(sliceCharSrc);
const sliceTrimCode = legacyTypeSyntaxView(sliceTrimSrc);
const charOffsetsCode = legacyTypeSyntaxView(charOffsetsSrc);

assert.match(rootSrc, /pub #import "\.\/string\/slice" as \*/, 'alloc/string facade must re-export string/slice');
assert.doesNotMatch(rootSrc, /pub #import "\.\/string\/char_offsets" as \*/, 'alloc/string facade must not expose char offset helpers directly');
assert.match(sliceSrc, /pub #import "\.\/slice\/byte" as @merge/, 'string/slice facade must merge byte slice helpers for qualified imports');
assert.match(sliceSrc, /pub #import "\.\/slice\/char" as @merge/, 'string/slice facade must merge char traversal helpers for qualified imports');
assert.match(sliceSrc, /pub #import "\.\/slice\/trim" as @merge/, 'string/slice facade must merge trim helpers for qualified imports');
assert.doesNotMatch(sliceCode, /\bfn\s+/, 'string/slice facade must not own implementation function bodies');
assert.match(sliceByteSrc, /#import "alloc\/string\/search" as \*/, 'string/slice/byte must use string/search predicates');
assert.match(sliceByteSrc, /#import "alloc\/string\/storage" as \*/, 'string/slice/byte must build owned output through string/storage');
assert.match(sliceCharSrc, /#import "alloc\/string\/char_offsets" as \*/, 'string/slice/char must delegate char offset calculation');
assert.match(sliceTrimSrc, /#import "alloc\/string\/search" as \*/, 'string/slice/trim must use string/search predicates');
assert.match(charOffsetsSrc, /#import "alloc\/string\/access" as \*/, 'string/char_offsets must read str bytes through string/access');
assert.match(charOffsetsSrc, /#import "alloc\/string\/utf8" as \*/, 'string/char_offsets must classify UTF-8 through string/utf8');

const ownerByName = new Map([
    ['str_trim_suffix_cr', sliceTrimCode],
    ['str_slice_trim_suffix_cr', sliceTrimCode],
    ['str_slice_result', sliceByteCode],
    ['str_slice', sliceByteCode],
    ['str_next_char_result', sliceCharCode],
    ['str_char_count', sliceCharCode],
    ['str_char_byte_index_result', sliceCharCode],
    ['str_char_at_result', sliceCharCode],
    ['str_slice_chars_result', sliceCharCode],
    ['str_starts_with_char', sliceCharCode],
    ['str_contains_char', sliceCharCode],
    ['str_trim', sliceTrimCode],
]);

for (const name of [
    'str_trim_suffix_cr',
    'str_slice_trim_suffix_cr',
    'str_slice_result',
    'str_slice',
    'str_next_char_result',
    'str_char_count',
    'str_char_byte_index_result',
    'str_char_at_result',
    'str_slice_chars_result',
    'str_starts_with_char',
    'str_contains_char',
    'str_trim',
]) {
    assert.doesNotMatch(rootCode, new RegExp(`fn\\s+${name}\\b`), `${rootRelPath} must not own ${name}`);
    assert.doesNotMatch(sliceCode, new RegExp(`fn\\s+${name}\\b`), `${sliceRelPath} facade must not own ${name}`);
    assert.match(ownerByName.get(name), new RegExp(`fn\\s+${name}\\b`), `string/slice submodule must own ${name}`);
}

for (const name of [
    'str_utf8_step_width_at',
    'str_char_slice_offsets_result',
]) {
    assert.doesNotMatch(rootCode, new RegExp(`fn\\s+${name}\\b`), `${rootRelPath} must not own ${name}`);
    assert.doesNotMatch(sliceCode, new RegExp(`fn\\s+${name}\\b`), `${sliceRelPath} must not own ${name}`);
    assert.match(charOffsetsCode, new RegExp(`fn\\s+${name}\\b`), `${charOffsetsRelPath} must own ${name}`);
}

assert.match(
    sliceByteCode,
    /fn\s+str_slice_result[\s\S]*str_utf8_is_boundary\s+s\s+s0[\s\S]*let\s+src\s+<MemPtr<u8>>\s+mem_ptr_add\s+string_data_ptr\s+s\s+s0[\s\S]*string_from_mem_unchecked_result\s+src\s+out_len/,
    'str_slice_result must validate UTF-8 byte boundaries and expose mem_ptr_add as call-head evidence before owned string construction',
);
assert.match(
    sliceCharCode,
    /fn\s+str_next_char_result[\s\S]*match\s+string_utf8_lead_kind\s+b0:[\s\S]*StringUtf8LeadKind::Two:[\s\S]*StringUtf8LeadKind::Three:[\s\S]*StringUtf8LeadKind::Four:/,
    'str_next_char_result must decode UTF-8 through exhaustive leading-byte enum match',
);
assert.match(
    charOffsetsCode,
    /fn\s+str_utf8_step_width_at[\s\S]*match\s+string_utf8_lead_kind\s+b0:[\s\S]*StringUtf8LeadKind::Ascii:[\s\S]*StringUtf8LeadKind::Two:[\s\S]*StringUtf8LeadKind::Three:[\s\S]*StringUtf8LeadKind::Four:[\s\S]*StringUtf8LeadKind::Invalid:/,
    'str_utf8_step_width_at must calculate byte width through exhaustive leading-byte enum match',
);
assert.ok(implementationLineCount(sliceSrc) <= 35, `${sliceRelPath} should stay as a small facade`);
assert.ok(implementationLineCount(sliceByteSrc) <= 90, `${sliceByteRelPath} should stay narrowly scoped`);
assert.ok(implementationLineCount(sliceCharSrc) <= 290, `${sliceCharRelPath} should stay narrowly scoped`);
assert.ok(implementationLineCount(sliceTrimSrc) <= 150, `${sliceTrimRelPath} should stay narrowly scoped`);
assert.ok(implementationLineCount(charOffsetsSrc) <= 220, `${charOffsetsRelPath} should stay narrowly scoped`);

console.log('alloc/string slice boundary regression passed');
