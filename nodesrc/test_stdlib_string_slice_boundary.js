#!/usr/bin/env node

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const { stripNeplComments } = require('./source_policy/stdlib_builder_owner');

const repoRoot = path.resolve(__dirname, '..');
const rootRelPath = 'stdlib/alloc/string.nepl';
const sliceRelPath = 'stdlib/alloc/string/slice.nepl';
const charOffsetsRelPath = 'stdlib/alloc/string/char_offsets.nepl';
const rootSrc = fs.readFileSync(path.join(repoRoot, rootRelPath), 'utf8');
const sliceSrc = fs.readFileSync(path.join(repoRoot, sliceRelPath), 'utf8');
const charOffsetsSrc = fs.readFileSync(path.join(repoRoot, charOffsetsRelPath), 'utf8');
const rootCode = stripNeplComments(rootSrc);
const sliceCode = stripNeplComments(sliceSrc);
const charOffsetsCode = stripNeplComments(charOffsetsSrc);

assert.match(rootSrc, /pub #import "\.\/string\/slice" as \*/, 'alloc/string facade must re-export string/slice');
assert.doesNotMatch(rootSrc, /pub #import "\.\/string\/char_offsets" as \*/, 'alloc/string facade must not expose char offset helpers directly');
assert.match(sliceSrc, /#import "alloc\/string\/search" as \*/, 'string/slice must use string/search predicates');
assert.match(sliceSrc, /#import "alloc\/string\/storage" as \*/, 'string/slice must build owned output through string/storage');
assert.match(sliceSrc, /#import "alloc\/string\/char_offsets" as \*/, 'string/slice must delegate char offset calculation');
assert.match(charOffsetsSrc, /#import "alloc\/string\/access" as \*/, 'string/char_offsets must read str bytes through string/access');
assert.match(charOffsetsSrc, /#import "alloc\/string\/utf8" as \*/, 'string/char_offsets must classify UTF-8 through string/utf8');

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
    assert.match(sliceCode, new RegExp(`fn\\s+${name}\\b`), `${sliceRelPath} must own ${name}`);
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
    sliceCode,
    /fn\s+str_slice_result[\s\S]*str_utf8_is_boundary\s+s\s+s0[\s\S]*string_from_mem_unchecked_result\s+mem_ptr_add\s+string_data_ptr\s+s\s+s0\s+out_len/,
    'str_slice_result must validate UTF-8 byte boundaries before owned string construction',
);
assert.match(
    sliceCode,
    /fn\s+str_next_char_result[\s\S]*match\s+string_utf8_lead_kind\s+b0:[\s\S]*StringUtf8LeadKind::Two:[\s\S]*StringUtf8LeadKind::Three:[\s\S]*StringUtf8LeadKind::Four:/,
    'str_next_char_result must decode UTF-8 through exhaustive leading-byte enum match',
);
assert.match(
    charOffsetsCode,
    /fn\s+str_utf8_step_width_at[\s\S]*match\s+string_utf8_lead_kind\s+b0:[\s\S]*StringUtf8LeadKind::Ascii:[\s\S]*StringUtf8LeadKind::Two:[\s\S]*StringUtf8LeadKind::Three:[\s\S]*StringUtf8LeadKind::Four:[\s\S]*StringUtf8LeadKind::Invalid:/,
    'str_utf8_step_width_at must calculate byte width through exhaustive leading-byte enum match',
);
assert.ok(sliceSrc.split(/\r?\n/).length <= 470, `${sliceRelPath} should stay narrowly scoped`);
assert.ok(charOffsetsSrc.split(/\r?\n/).length <= 220, `${charOffsetsRelPath} should stay narrowly scoped`);

console.log('alloc/string slice boundary regression passed');
