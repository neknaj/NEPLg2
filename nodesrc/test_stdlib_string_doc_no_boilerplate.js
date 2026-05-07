#!/usr/bin/env node

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');

const repoRoot = path.resolve(__dirname, '..');
const relPath = 'stdlib/alloc/string.nepl';
const src = fs.readFileSync(path.join(repoRoot, relPath), 'utf8');
const accessRelPath = 'stdlib/alloc/string/access.nepl';
const accessSrc = fs.readFileSync(path.join(repoRoot, accessRelPath), 'utf8');
const builderRelPath = 'stdlib/alloc/string/builder.nepl';
const builderSrc = fs.readFileSync(path.join(repoRoot, builderRelPath), 'utf8');
const searchRelPath = 'stdlib/alloc/string/search.nepl';
const searchSrc = fs.readFileSync(path.join(repoRoot, searchRelPath), 'utf8');
const sliceRelPath = 'stdlib/alloc/string/slice.nepl';
const sliceSrc = fs.readFileSync(path.join(repoRoot, sliceRelPath), 'utf8');
const splitRelPath = 'stdlib/alloc/string/split.nepl';
const splitSrc = fs.readFileSync(path.join(repoRoot, splitRelPath), 'utf8');
const integerRelPath = 'stdlib/alloc/string/integer.nepl';
const integerSrc = fs.readFileSync(path.join(repoRoot, integerRelPath), 'utf8');
const integerFormatRelPath = 'stdlib/alloc/string/integer/format.nepl';
const integerFormatSrc = fs.readFileSync(path.join(repoRoot, integerFormatRelPath), 'utf8');
const integerParseRelPath = 'stdlib/alloc/string/integer/parse.nepl';
const integerParseSrc = fs.readFileSync(path.join(repoRoot, integerParseRelPath), 'utf8');
const floatRelPath = 'stdlib/alloc/string/float.nepl';
const floatSrc = fs.readFileSync(path.join(repoRoot, floatRelPath), 'utf8');
const concatRelPath = 'stdlib/alloc/string/concat.nepl';
const concatSrc = fs.readFileSync(path.join(repoRoot, concatRelPath), 'utf8');
const builderExtRelPath = 'stdlib/alloc/string/builder_ext.nepl';
const builderExtSrc = fs.readFileSync(path.join(repoRoot, builderExtRelPath), 'utf8');
const findRelPath = 'stdlib/alloc/string/find.nepl';
const findSrc = fs.readFileSync(path.join(repoRoot, findRelPath), 'utf8');

const forbiddenPhrases = [
    ['generic main-use title', '\u4e3b\u306a\u7528\u9014'],
    ['predefined-process placeholder', '\u5b9a\u7fa9\u6e08\u307f\u51e6\u7406'],
    ['thin-wrapper placeholder', '\u8584\u3044\u30e9\u30c3\u30d1'],
    ['move-and-rebind placeholder', '\u518d\u5229\u7528\u6642\u306f\u675f\u7e1b\u3057\u76f4'],
    ['generic enum overview', '\u5217\u6319\u578b\u306e\u6982\u8981'],
];

for (const [sourceRelPath, sourceText] of [
    [relPath, src],
    [accessRelPath, accessSrc],
    [builderRelPath, builderSrc],
    [searchRelPath, searchSrc],
    [sliceRelPath, sliceSrc],
    [splitRelPath, splitSrc],
    [integerRelPath, integerSrc],
    [integerFormatRelPath, integerFormatSrc],
    [integerParseRelPath, integerParseSrc],
    [floatRelPath, floatSrc],
    [concatRelPath, concatSrc],
    [builderExtRelPath, builderExtSrc],
    [findRelPath, findSrc],
]) {
    for (const [label, phrase] of forbiddenPhrases) {
        assert.equal(sourceText.includes(phrase), false, `${sourceRelPath} must not contain generated doc boilerplate: ${label}`);
    }
}

const requiredPhrases = [
    [accessRelPath, accessSrc, 'byte length contract', 'len: \u6587\u5b57\u5217\u306e byte \u9577\u3092\u8fd4\u3059'],
    [concatRelPath, concatSrc, 'concat Result contract', 'concat_result: 2 \u3064\u306e\u6587\u5b57\u5217\u3092\u78ba\u4fdd\u4ed8\u304d\u3067\u9023\u7d50\u3059\u308b'],
    [builderRelPath, builderSrc, 'StringBuilder byte-buffer ownership contract', 'StringBuilder: \u8907\u6570\u306e str \u7247\u3092 byte buffer \u306b\u8ffd\u52a0\u3057\u3066\u6700\u5f8c\u306b 1 \u3064\u3078\u307e\u3068\u3081\u308b'],
    [builderRelPath, builderSrc, 'StringBuilder non-Copy owner contract', 'builder \u306f `Copy` / `Clone` \u3067\u306f\u3042\u308a\u307e\u305b\u3093'],
    [builderRelPath, builderSrc, 'StringBuilder raw storage contract', 'raw storage \u306b\u306f `u8` \u3060\u3051\u3092\u7f6e\u304d'],
    [sliceRelPath, sliceSrc, 'slice UTF-8 boundary contract', 'str_slice_result: UTF-8 \u5883\u754c\u306b\u63c3\u3063\u305f byte \u7bc4\u56f2\u3092\u65b0\u3057\u3044 str \u3068\u3057\u3066\u5207\u308a\u51fa\u3059'],
    [splitRelPath, splitSrc, 'split scanner byte-scan contract', 'str_split_next: allocation \u306a\u3057\u3067\u6b21\u306e split range \u3092\u8fd4\u3059'],
    [integerParseRelPath, integerParseSrc, 'i32 parse overflow contract', 'to_i32: 10 \u9032\u6587\u5b57\u5217\u3092 i32 \u3068\u3057\u3066\u89e3\u6790\u3059\u308b'],
    [floatRelPath, floatSrc, 'f64 formatting contract', 'from_f64_result: f64 \u3092\u6709\u9650\u5024\u3060\u3051\u6587\u5b57\u5217\u5316\u3059\u308b'],
    [findRelPath, findSrc, 'find byte-index contract', 'find: \u6700\u521d\u306b\u4e00\u81f4\u3057\u305f byte index \u3092\u8fd4\u3059'],
    [accessRelPath, accessSrc, 'UTF-8 byte-length note', 'UTF-8 \u306e byte \u9577\u3067\u3042\u308a'],
    [builderRelPath, builderSrc, 'allocation failure Result guidance', 'allocation failure \u3092\u6271\u3046\u51e6\u7406\u3067\u306f'],
];

for (const [sourceRelPath, sourceText, label, phrase] of requiredPhrases) {
    assert.equal(sourceText.includes(phrase), true, `${sourceRelPath} must document ${label}`);
}

console.log('stdlib string doc boilerplate regression passed');
