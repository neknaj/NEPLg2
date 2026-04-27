#!/usr/bin/env node

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');

const repoRoot = path.resolve(__dirname, '..');
const relPath = 'stdlib/core/cast.nepl';
const src = fs.readFileSync(path.join(repoRoot, relPath), 'utf8');

const forbiddenPhrases = [
    ['generic main-use title', '\u4e3b\u306a\u7528\u9014'],
    ['predefined-process placeholder', '\u5b9a\u7fa9\u6e08\u307f\u51e6\u7406'],
    ['thin-wrapper placeholder', '\u8584\u3044\u30e9\u30c3\u30d1'],
    ['move-and-rebind placeholder', '\u518d\u5229\u7528\u6642\u306f\u675f\u7e1b\u3057\u76f4'],
];

for (const [label, phrase] of forbiddenPhrases) {
    assert.equal(src.includes(phrase), false, `${relPath} must not contain generated doc boilerplate: ${label}`);
}

const requiredPhrases = [
    ['i32 to f32 numeric semantics', 'cast: i32 \u3092 f32 \u306b\u6570\u5024\u5909\u63db\u3059\u308b'],
    ['f32 to i32 numeric semantics', 'cast: f32 \u3092 i32 \u306b\u6570\u5024\u5909\u63db\u3059\u308b'],
    ['bool to i32 truth mapping', '`true` \u3092 1\u3001`false` \u3092 0'],
    ['i32 to u8 mask semantics', '0xff \u30de\u30b9\u30af\u76f8\u5f53\u306e\u7e2e\u5c0f\u5909\u63db'],
    ['bitcast i32 to f32 semantics', 'bitcast_i32_to_f32: i32 \u306e\u30d3\u30c3\u30c8\u5217\u3092 f32 \u3068\u3057\u3066\u518d\u89e3\u91c8\u3059\u308b'],
    ['bitcast f32 to i32 semantics', 'bitcast_f32_to_i32: f32 \u306e\u30d3\u30c3\u30c8\u5217\u3092 i32 \u3068\u3057\u3066\u518d\u89e3\u91c8\u3059\u308b'],
];

for (const [label, phrase] of requiredPhrases) {
    assert.equal(src.includes(phrase), true, `${relPath} must document ${label}`);
}

console.log('stdlib core/cast doc boilerplate regression passed');
