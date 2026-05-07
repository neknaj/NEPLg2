#!/usr/bin/env node

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');

const repoRoot = path.resolve(__dirname, '..');

function read(rel) {
    return fs.readFileSync(path.join(repoRoot, rel), 'utf8');
}

const stringSrc = read('stdlib/alloc/string.nepl');
const stringSliceSrc = read('stdlib/alloc/string/slice.nepl');
const stringSliceTrimSrc = read('stdlib/alloc/string/slice/trim.nepl');
const scannerSrc = read('stdlib/alloc/string/scanner.nepl');
const importSpecSrc = read('stdlib/neplg2/core/module/import_spec.nepl');
const nmParserSrc = read('stdlib/nm/parser.nepl');
const nmParserJsonInlineSrc = read('stdlib/nm/parser/json_inline.nepl');
const nmHtmlSrc = read('stdlib/nm/html_gen.nepl');
const nmHtmlInlineSrc = read('stdlib/nm/html_inline.nepl');

assert.doesNotMatch(
    stringSrc,
    /#import\s+"alloc\/string\/scanner"\s+as\s+scanner/,
    'alloc/string.nepl must not hide scanner helpers behind a facade import',
);

for (const [name, src] of [
    ['import_spec', importSpecSrc],
    ['nm parser', nmParserSrc],
    ['nm parser json_inline', nmParserJsonInlineSrc],
    ['nm html_gen', nmHtmlSrc],
    ['nm html_inline', nmHtmlInlineSrc],
]) {
    assert.match(
        src,
        /#import\s+"alloc\/string\/scanner"\s+as\s+scanner/,
        `${name} must import the scanner module directly`,
    );
}

for (const fnName of [
    'str_find_byte_range',
    'str_line_end',
    'str_next_line_pos',
    'str_skip_inline_space_range',
    'str_word_end_inline_space_range',
    'str_byte_is_ascii_digit',
    'str_byte_is_ascii_alpha',
    'str_byte_is_ascii_hex_digit',
    'str_byte_is_ascii_inline_space',
]) {
    assert.match(scannerSrc, new RegExp(`\\bpub\\s+fn\\s+${fnName}\\b`), `alloc/string/scanner.nepl must expose ${fnName}`);
    assert.doesNotMatch(stringSrc, new RegExp(`\\bfn\\s+${fnName}\\b`), `alloc/string.nepl must not keep scanner facade ${fnName}`);
}

assert.match(
    stringSrc,
    /pub\s+#import\s+"\.\/string\/slice"\s+as\s+\*/,
    'alloc/string.nepl must re-export slice helpers from alloc/string/slice.nepl',
);
assert.match(
    stringSliceSrc,
    /pub\s+#import\s+"\.\/slice\/trim"\s+as\s+@merge/,
    'alloc/string/slice.nepl must merge trim helpers from alloc/string/slice/trim.nepl for qualified imports',
);

for (const fnName of [
    'str_trim_suffix_cr',
    'str_slice_trim_suffix_cr',
]) {
    assert.match(stringSliceTrimSrc, new RegExp(`\\bfn\\s+${fnName}\\b`), `alloc/string/slice/trim.nepl must own ${fnName}`);
    assert.doesNotMatch(stringSliceSrc, new RegExp(`\\bfn\\s+${fnName}\\b`), `alloc/string/slice.nepl facade must not own ${fnName}`);
    assert.doesNotMatch(stringSrc, new RegExp(`\\bfn\\s+${fnName}\\b`), `alloc/string.nepl must not keep slice helper ${fnName}`);
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
    'scanner::str_find_byte_range',
    'scanner::str_skip_inline_space_range',
    'scanner::str_word_end_inline_space_range',
    'scanner::str_byte_is_ascii_inline_space',
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
    assert.doesNotMatch(nmParserJsonInlineSrc, new RegExp(`\\b${localName}\\b`), `nm parser json_inline must use alloc/string scanner helpers instead of ${localName}`);
    assert.doesNotMatch(nmHtmlSrc, new RegExp(`\\b${localName}\\b`), `nm html_gen must use alloc/string scanner helpers instead of ${localName}`);
    assert.doesNotMatch(nmHtmlInlineSrc, new RegExp(`\\b${localName}\\b`), `nm html_inline must use alloc/string scanner helpers instead of ${localName}`);
}

for (const symbol of [
    'str_slice_trim_suffix_cr',
]) {
    assert.match(nmParserSrc, new RegExp(`\\b${symbol}\\b`), `nm parser must call ${symbol}`);
    assert.match(nmHtmlSrc, new RegExp(`\\b${symbol}\\b`), `nm html_gen must call ${symbol}`);
}

for (const symbol of [
    'scanner::str_line_end',
    'scanner::str_next_line_pos',
]) {
    assert.match(nmParserSrc, new RegExp(symbol.replaceAll(':', '\\:')), `nm parser must call ${symbol}`);
}

for (const symbol of [
    'scanner::str_find_byte_range',
]) {
    assert.match(nmParserJsonInlineSrc, new RegExp(symbol.replaceAll(':', '\\:')), `nm parser json_inline must call ${symbol}`);
}

for (const symbol of [
    'scanner::str_line_end',
    'scanner::str_next_line_pos',
]) {
    assert.match(nmHtmlSrc, new RegExp(symbol.replaceAll(':', '\\:')), `nm html_gen must call ${symbol}`);
}

for (const symbol of [
    'scanner::str_find_byte_range',
]) {
    assert.match(nmHtmlInlineSrc, new RegExp(symbol.replaceAll(':', '\\:')), `nm html_inline must call ${symbol}`);
}

console.log('stdlib byte scanner helper boundary regression passed');
