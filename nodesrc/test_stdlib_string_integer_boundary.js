#!/usr/bin/env node

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const { stripNeplComments } = require('./source_policy/stdlib_builder_owner');

const repoRoot = path.resolve(__dirname, '..');
const rootRelPath = 'stdlib/alloc/string.nepl';
const integerRelPath = 'stdlib/alloc/string/integer.nepl';
const commonRelPath = 'stdlib/alloc/string/integer/common.nepl';
const rootSrc = fs.readFileSync(path.join(repoRoot, rootRelPath), 'utf8');
const integerSrc = fs.readFileSync(path.join(repoRoot, integerRelPath), 'utf8');
const commonSrc = fs.readFileSync(path.join(repoRoot, commonRelPath), 'utf8');
const rootCode = stripNeplComments(rootSrc);
const integerCode = stripNeplComments(integerSrc);
const commonCode = stripNeplComments(commonSrc);

assert.match(rootSrc, /pub #import "\.\/string\/integer" as \*/, 'alloc/string facade must re-export string/integer');
assert.match(integerSrc, /pub #import "\.\/integer\/common" as \*/, 'string/integer must re-export integer/common helpers');

for (const importPath of [
    'alloc/string/access',
    'alloc/string/search',
    'alloc/string/slice',
    'alloc/string/storage',
    'alloc/string/builder',
]) {
    assert.match(integerSrc, new RegExp(`#import "${importPath}" as \\*`), `string/integer must import ${importPath} directly`);
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
    assert.match(commonCode, new RegExp(`fn\\s+${name}\\b`), `${commonRelPath} must own ${name}`);
}

for (const name of [
    'from_i32_radix',
    'to_i64_radix',
    'from_u128_radix',
    'to_i128_radix',
]) {
    assert.doesNotMatch(rootCode, new RegExp(`fn\\s+${name}\\b`), `${rootRelPath} must not own ${name}`);
    assert.match(integerCode, new RegExp(`fn\\s+${name}\\b`), `${integerRelPath} must own ${name}`);
}

for (const name of [
    'U128DivRem',
]) {
    assert.doesNotMatch(rootCode, new RegExp(`struct\\s+${name}\\b`), `${rootRelPath} must not own ${name}`);
    assert.doesNotMatch(integerCode, new RegExp(`struct\\s+${name}\\b`), `${integerRelPath} must not own common helper struct ${name}`);
    assert.match(commonCode, new RegExp(`struct\\s+${name}\\b`), `${commonRelPath} must own ${name}`);
}

assert.doesNotMatch(
    integerCode,
    /\bconcat_result\b/,
    'string/integer must not depend on the alloc/string root concat facade',
);
assert.match(
    integerCode,
    /fn\s+from_i128_radix[\s\S]*string_builder_with_capacity_result[\s\S]*sb_append_byte_result[\s\S]*sb_append_result[\s\S]*sb_build_result/,
    'negative i128 formatting must prepend the sign through StringBuilder ownership APIs',
);
assert.match(
    integerCode,
    /fn\s+to_u128_radix[\s\S]*u128_can_mul_add_small/,
    'u128 parsing must keep overflow checks before multiply-add',
);
assert.ok(integerSrc.split(/\r?\n/).length <= 650, `${integerRelPath} should stay within the public integer conversion boundary`);
assert.ok(commonSrc.split(/\r?\n/).length <= 500, `${commonRelPath} should stay within the common helper boundary`);

console.log('alloc/string integer boundary regression passed');
