#!/usr/bin/env node

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const { stripNeplComments } = require('./source_policy/stdlib_builder_owner');

const repoRoot = path.resolve(__dirname, '..');
const rootRelPath = 'stdlib/alloc/string.nepl';
const integerRelPath = 'stdlib/alloc/string/integer.nepl';
const commonRelPath = 'stdlib/alloc/string/integer/common.nepl';
const commonBoolRelPath = 'stdlib/alloc/string/integer/common/bool.nepl';
const commonRadixRelPath = 'stdlib/alloc/string/integer/common/radix.nepl';
const commonU128RelPath = 'stdlib/alloc/string/integer/common/u128.nepl';
const formatRelPath = 'stdlib/alloc/string/integer/format.nepl';
const parseRelPath = 'stdlib/alloc/string/integer/parse.nepl';
const rootSrc = fs.readFileSync(path.join(repoRoot, rootRelPath), 'utf8');
const integerSrc = fs.readFileSync(path.join(repoRoot, integerRelPath), 'utf8');
const commonSrc = fs.readFileSync(path.join(repoRoot, commonRelPath), 'utf8');
const commonBoolSrc = fs.readFileSync(path.join(repoRoot, commonBoolRelPath), 'utf8');
const commonRadixSrc = fs.readFileSync(path.join(repoRoot, commonRadixRelPath), 'utf8');
const commonU128Src = fs.readFileSync(path.join(repoRoot, commonU128RelPath), 'utf8');
const formatSrc = fs.readFileSync(path.join(repoRoot, formatRelPath), 'utf8');
const parseSrc = fs.readFileSync(path.join(repoRoot, parseRelPath), 'utf8');
const rootCode = stripNeplComments(rootSrc);
const integerCode = stripNeplComments(integerSrc);
const commonCode = stripNeplComments(commonSrc);
const commonBoolCode = stripNeplComments(commonBoolSrc);
const commonRadixCode = stripNeplComments(commonRadixSrc);
const commonU128Code = stripNeplComments(commonU128Src);
const formatCode = stripNeplComments(formatSrc);
const parseCode = stripNeplComments(parseSrc);

assert.match(rootSrc, /pub #import "\.\/string\/integer" as \*/, 'alloc/string facade must re-export string/integer');
assert.match(integerSrc, /pub #import "\.\/integer\/common" as \*/, 'string/integer must re-export integer/common helpers');
assert.match(integerSrc, /pub #import "\.\/integer\/format" as \*/, 'string/integer must re-export integer/format APIs');
assert.match(integerSrc, /pub #import "\.\/integer\/parse" as \*/, 'string/integer must re-export integer/parse APIs');
assert.match(commonSrc, /pub #import "\.\/common\/bool" as \*/, 'string/integer/common facade must re-export bool helpers');
assert.match(commonSrc, /pub #import "\.\/common\/radix" as \*/, 'string/integer/common facade must re-export radix helpers');
assert.match(commonSrc, /pub #import "\.\/common\/u128" as \*/, 'string/integer/common facade must re-export u128 helpers');

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
]) {
    assert.doesNotMatch(rootCode, new RegExp(`fn\\s+${name}\\b`), `${rootRelPath} must not own ${name}`);
    assert.doesNotMatch(integerCode, new RegExp(`fn\\s+${name}\\b`), `${integerRelPath} must not own common helper ${name}`);
    assert.doesNotMatch(commonCode, new RegExp(`fn\\s+${name}\\b`), `${commonRelPath} facade must not own common helper ${name}`);
    assert.match(commonBoolCode, new RegExp(`fn\\s+${name}\\b`), `${commonBoolRelPath} must own ${name}`);
    assert.doesNotMatch(commonRadixCode, new RegExp(`fn\\s+${name}\\b`), `${commonRadixRelPath} must not own bool helper ${name}`);
    assert.doesNotMatch(commonU128Code, new RegExp(`fn\\s+${name}\\b`), `${commonU128RelPath} must not own bool helper ${name}`);
    assert.doesNotMatch(formatCode, new RegExp(`fn\\s+${name}\\b`), `${formatRelPath} must not own common helper ${name}`);
    assert.doesNotMatch(parseCode, new RegExp(`fn\\s+${name}\\b`), `${parseRelPath} must not own common helper ${name}`);
}

for (const name of [
    'digit_to_char_lower',
    'digit_from_char',
    'validate_radix',
]) {
    assert.doesNotMatch(rootCode, new RegExp(`fn\\s+${name}\\b`), `${rootRelPath} must not own ${name}`);
    assert.doesNotMatch(integerCode, new RegExp(`fn\\s+${name}\\b`), `${integerRelPath} must not own common helper ${name}`);
    assert.doesNotMatch(commonCode, new RegExp(`fn\\s+${name}\\b`), `${commonRelPath} facade must not own common helper ${name}`);
    assert.doesNotMatch(commonBoolCode, new RegExp(`fn\\s+${name}\\b`), `${commonBoolRelPath} must not own radix helper ${name}`);
    assert.match(commonRadixCode, new RegExp(`fn\\s+${name}\\b`), `${commonRadixRelPath} must own ${name}`);
    assert.doesNotMatch(commonU128Code, new RegExp(`fn\\s+${name}\\b`), `${commonU128RelPath} must not own radix helper ${name}`);
    assert.doesNotMatch(formatCode, new RegExp(`fn\\s+${name}\\b`), `${formatRelPath} must not own common helper ${name}`);
    assert.doesNotMatch(parseCode, new RegExp(`fn\\s+${name}\\b`), `${parseRelPath} must not own common helper ${name}`);
}

for (const name of [
    'u128_zero',
    'u128_divrem_small',
    'u128_can_mul_add_small',
]) {
    assert.doesNotMatch(rootCode, new RegExp(`fn\\s+${name}\\b`), `${rootRelPath} must not own ${name}`);
    assert.doesNotMatch(integerCode, new RegExp(`fn\\s+${name}\\b`), `${integerRelPath} must not own common helper ${name}`);
    assert.doesNotMatch(commonCode, new RegExp(`fn\\s+${name}\\b`), `${commonRelPath} facade must not own common helper ${name}`);
    assert.doesNotMatch(commonBoolCode, new RegExp(`fn\\s+${name}\\b`), `${commonBoolRelPath} must not own u128 helper ${name}`);
    assert.doesNotMatch(commonRadixCode, new RegExp(`fn\\s+${name}\\b`), `${commonRadixRelPath} must not own u128 helper ${name}`);
    assert.match(commonU128Code, new RegExp(`fn\\s+${name}\\b`), `${commonU128RelPath} must own ${name}`);
    assert.doesNotMatch(formatCode, new RegExp(`fn\\s+${name}\\b`), `${formatRelPath} must not own common helper ${name}`);
    assert.doesNotMatch(parseCode, new RegExp(`fn\\s+${name}\\b`), `${parseRelPath} must not own common helper ${name}`);
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
    assert.doesNotMatch(commonCode, new RegExp(`struct\\s+${name}\\b`), `${commonRelPath} facade must not own common helper struct ${name}`);
    assert.match(commonU128Code, new RegExp(`struct\\s+${name}\\b`), `${commonU128RelPath} must own ${name}`);
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
assert.ok(codeLineCount(integerSrc) <= 80, `${integerRelPath} should stay within the public integer facade implementation boundary`);
assert.doesNotMatch(commonCode, /\b(?:fn|struct|enum)\s+/, `${commonRelPath} must stay as a small common facade`);
assert.ok(codeLineCount(commonSrc) <= 40, `${commonRelPath} should stay within the common facade implementation boundary`);
assert.ok(codeLineCount(commonBoolSrc) <= 120, `${commonBoolRelPath} should stay within the bool helper implementation boundary`);
assert.ok(codeLineCount(commonRadixSrc) <= 120, `${commonRadixRelPath} should stay within the radix helper implementation boundary`);
assert.ok(codeLineCount(commonU128Src) <= 330, `${commonU128RelPath} should stay within the u128 helper implementation boundary`);
assert.ok(codeLineCount(formatSrc) <= 300, `${formatRelPath} should stay within the format implementation boundary`);
assert.ok(codeLineCount(parseSrc) <= 380, `${parseRelPath} should stay within the parse implementation boundary`);
assert.match(formatCode, /\b(?:mem_ptr_addr|store_u8|RegionToken)\b/, 'string/integer/format must carry source-level raw memory evidence');
assert.doesNotMatch(integerCode, /\b(?:mem_ptr_addr|store_u8|load_u8|mem_copy|RegionToken)\b/, 'string/integer facade must not carry direct raw memory evidence');

console.log('alloc/string integer boundary regression passed');

function codeLineCount(source) {
    return stripNeplComments(source)
        .split(/\r?\n/)
        .filter((line) => line.trim().length > 0)
        .length;
}
