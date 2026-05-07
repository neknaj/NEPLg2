#!/usr/bin/env node

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const { stripNeplComments } = require('./source_policy/stdlib_builder_owner');

const repoRoot = path.resolve(__dirname, '..');
const rootRelPath = 'stdlib/alloc/string.nepl';
const integerRelPath = 'stdlib/alloc/string/integer.nepl';
const commonRelPath = 'stdlib/alloc/string/integer/common.nepl';
const formatRelPath = 'stdlib/alloc/string/integer/format.nepl';
const parseRelPath = 'stdlib/alloc/string/integer/parse.nepl';
const rootSrc = fs.readFileSync(path.join(repoRoot, rootRelPath), 'utf8');
const integerSrc = fs.readFileSync(path.join(repoRoot, integerRelPath), 'utf8');
const commonSrc = fs.readFileSync(path.join(repoRoot, commonRelPath), 'utf8');
const formatSrc = fs.readFileSync(path.join(repoRoot, formatRelPath), 'utf8');
const parseSrc = fs.readFileSync(path.join(repoRoot, parseRelPath), 'utf8');
const rootCode = stripNeplComments(rootSrc);
const integerCode = stripNeplComments(integerSrc);
const commonCode = stripNeplComments(commonSrc);
const formatCode = stripNeplComments(formatSrc);
const parseCode = stripNeplComments(parseSrc);
const loaderSrc = fs.readFileSync(path.join(repoRoot, 'nepl-core/src/loader.rs'), 'utf8');

assert.match(rootSrc, /pub #import "\.\/string\/integer" as \*/, 'alloc/string facade must re-export string/integer');
assert.match(integerSrc, /pub #import "\.\/integer\/common" as \*/, 'string/integer must re-export integer/common helpers');
assert.match(integerSrc, /pub #import "\.\/integer\/format" as \*/, 'string/integer must re-export integer/format APIs');
assert.match(integerSrc, /pub #import "\.\/integer\/parse" as \*/, 'string/integer must re-export integer/parse APIs');

for (const importPath of [
    'alloc/string/storage',
    'alloc/string/builder',
]) {
    assert.match(formatSrc, new RegExp(`#import "${importPath}" as \\*`), `string/integer/format must import ${importPath} directly`);
}

for (const importPath of [
    'alloc/string/access',
    'alloc/string/slice',
]) {
    assert.match(parseSrc, new RegExp(`#import "${importPath}" as \\*`), `string/integer/parse must import ${importPath} directly`);
}

for (const name of [
    'from_bool',
    'to_bool',
    'digit_to_char_lower',
    'digit_from_char',
    'validate_radix',
    'u128_zero',
    'u128_divrem_small',
    'u128_can_mul_add_small',
]) {
    assert.doesNotMatch(rootCode, new RegExp(`fn\\s+${name}\\b`), `${rootRelPath} must not own ${name}`);
    assert.doesNotMatch(integerCode, new RegExp(`fn\\s+${name}\\b`), `${integerRelPath} must not own common helper ${name}`);
    assert.doesNotMatch(formatCode, new RegExp(`fn\\s+${name}\\b`), `${formatRelPath} must not own common helper ${name}`);
    assert.doesNotMatch(parseCode, new RegExp(`fn\\s+${name}\\b`), `${parseRelPath} must not own common helper ${name}`);
    assert.match(commonCode, new RegExp(`fn\\s+${name}\\b`), `${commonRelPath} must own ${name}`);
}

for (const name of [
    'from_i32_radix',
    'from_u128_radix',
    'from_i128_radix',
]) {
    assert.doesNotMatch(rootCode, new RegExp(`fn\\s+${name}\\b`), `${rootRelPath} must not own ${name}`);
    assert.doesNotMatch(integerCode, new RegExp(`fn\\s+${name}\\b`), `${integerRelPath} facade must not own ${name}`);
    assert.match(formatCode, new RegExp(`fn\\s+${name}\\b`), `${formatRelPath} must own ${name}`);
    assert.doesNotMatch(parseCode, new RegExp(`fn\\s+${name}\\b`), `${parseRelPath} must not own format API ${name}`);
}

for (const name of [
    'to_i64_radix',
    'to_u128_radix',
    'to_i128_radix',
]) {
    assert.doesNotMatch(rootCode, new RegExp(`fn\\s+${name}\\b`), `${rootRelPath} must not own ${name}`);
    assert.doesNotMatch(integerCode, new RegExp(`fn\\s+${name}\\b`), `${integerRelPath} facade must not own ${name}`);
    assert.doesNotMatch(formatCode, new RegExp(`fn\\s+${name}\\b`), `${formatRelPath} must not own parse API ${name}`);
    assert.match(parseCode, new RegExp(`fn\\s+${name}\\b`), `${parseRelPath} must own ${name}`);
}

for (const name of [
    'U128DivRem',
]) {
    assert.doesNotMatch(rootCode, new RegExp(`struct\\s+${name}\\b`), `${rootRelPath} must not own ${name}`);
    assert.doesNotMatch(integerCode, new RegExp(`struct\\s+${name}\\b`), `${integerRelPath} must not own common helper struct ${name}`);
    assert.match(commonCode, new RegExp(`struct\\s+${name}\\b`), `${commonRelPath} must own ${name}`);
}

assert.doesNotMatch(
    formatCode,
    /\bconcat_result\b/,
    'string/integer/format must not depend on the alloc/string root concat facade',
);
assert.match(
    formatCode,
    /fn\s+from_i128_radix[\s\S]*string_builder_with_capacity_result[\s\S]*sb_append_byte_result[\s\S]*sb_append_result[\s\S]*sb_build_result/,
    'negative i128 formatting must prepend the sign through StringBuilder ownership APIs',
);
assert.match(
    parseCode,
    /fn\s+to_u128_radix[\s\S]*u128_can_mul_add_small/,
    'u128 parsing must keep overflow checks before multiply-add',
);
assert.ok(integerSrc.split(/\r?\n/).length <= 80, `${integerRelPath} should stay within the public integer facade boundary`);
assert.ok(commonSrc.split(/\r?\n/).length <= 500, `${commonRelPath} should stay within the common helper boundary`);
assert.ok(formatSrc.split(/\r?\n/).length <= 300, `${formatRelPath} should stay within the format boundary`);
assert.ok(parseSrc.split(/\r?\n/).length <= 380, `${parseRelPath} should stay within the parse boundary`);
assert.ok(
    loaderSrc.includes('&["alloc", "string", "integer", "format.nepl"]'),
    'loader raw-memory boundary must include integer/format after moving from_u128_radix',
);

console.log('alloc/string integer boundary regression passed');
