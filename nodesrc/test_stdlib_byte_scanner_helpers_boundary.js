#!/usr/bin/env node

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');

const repoRoot = path.resolve(__dirname, '..');

function read(rel) {
    return fs.readFileSync(path.join(repoRoot, rel), 'utf8');
}

const stringSrc = read('stdlib/alloc/string.nepl');
const importSpecSrc = read('stdlib/neplg2/core/module/import_spec.nepl');
const nmParserSrc = read('stdlib/nm/parser.nepl');
const nmHtmlSrc = read('stdlib/nm/html_gen.nepl');

for (const fnName of [
    'str_find_byte_range',
    'str_line_end',
    'str_next_line_pos',
    'str_trim_suffix_cr',
    'str_skip_inline_space_range',
    'str_word_end_inline_space_range',
    'str_byte_is_ascii_digit',
    'str_byte_is_ascii_alpha',
    'str_byte_is_ascii_hex_digit',
    'str_byte_is_ascii_inline_space',
]) {
    assert.match(stringSrc, new RegExp(`\\bfn\\s+${fnName}\\b`), `alloc/string.nepl must expose ${fnName}`);
}

for (const localName of [
    'selfhost_import_find_byte',
    'selfhost_import_is_space',
    'selfhost_import_skip_space',
    'selfhost_import_word_end',
]) {
    assert.doesNotMatch(importSpecSrc, new RegExp(`\\b${localName}\\b`), `import_spec must use alloc/string scanner helpers instead of ${localName}`);
}

for (const symbol of [
    'string::str_find_byte_range',
    'string::str_skip_inline_space_range',
    'string::str_word_end_inline_space_range',
    'string::str_byte_is_ascii_inline_space',
]) {
    assert.match(importSpecSrc, new RegExp(symbol.replaceAll(':', '\\:')), `import_spec must call ${symbol}`);
}

for (const localName of [
    'nm_line_end',
    'nm_next_line_pos',
    'trim_cr',
    'nm_find_byte',
]) {
    assert.doesNotMatch(nmParserSrc, new RegExp(`\\b${localName}\\b`), `nm parser must use alloc/string scanner helpers instead of ${localName}`);
    assert.doesNotMatch(nmHtmlSrc, new RegExp(`\\b${localName}\\b`), `nm html_gen must use alloc/string scanner helpers instead of ${localName}`);
}

for (const symbol of [
    'str_line_end',
    'str_next_line_pos',
    'str_trim_suffix_cr',
    'str_find_byte_range',
]) {
    assert.match(nmParserSrc, new RegExp(`\\b${symbol}\\b`), `nm parser must call ${symbol}`);
    assert.match(nmHtmlSrc, new RegExp(`\\b${symbol}\\b`), `nm html_gen must call ${symbol}`);
}

console.log('stdlib byte scanner helper boundary regression passed');
