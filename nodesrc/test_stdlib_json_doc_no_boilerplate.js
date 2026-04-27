#!/usr/bin/env node

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');

const repoRoot = path.resolve(__dirname, '..');
const relPath = 'stdlib/alloc/encoding/json.nepl';
const src = fs.readFileSync(path.join(repoRoot, relPath), 'utf8');

const forbiddenPhrases = [
    ['generic main-use title', '\u4e3b\u306a\u7528\u9014'],
    ['predefined-process placeholder', '\u5b9a\u7fa9\u6e08\u307f\u51e6\u7406'],
    ['thin-wrapper placeholder', '\u8584\u3044\u30e9\u30c3\u30d1'],
    ['move-and-rebind placeholder', '\u518d\u5229\u7528\u6642\u306f\u675f\u7e1b\u3057\u76f4'],
    ['generic enum overview', '\u5217\u6319\u578b\u306e\u6982\u8981'],
];

for (const [label, phrase] of forbiddenPhrases) {
    assert.equal(src.includes(phrase), false, `${relPath} must not contain generated doc boilerplate: ${label}`);
}

const requiredPhrases = [
    ['JsonValue variant contract', 'JsonValue: JSON \u306e\u5024\u8868\u73fe'],
    ['null constructor contract', 'json_null: JSON null \u5024\u3092\u4f5c\u308b'],
    ['bool constructor contract', 'json_bool: bool \u5024\u3092 JSON Bool \u306b\u5909\u63db\u3059\u308b'],
    ['array owner transfer contract', '`arr` \u306e\u6240\u6709\u6a29\u306f\u8fd4\u308a\u5024\u306b\u79fb\u308a\u307e\u3059'],
    ['object owner transfer contract', '`obj` \u306e\u6240\u6709\u6a29\u306f\u8fd4\u308a\u5024\u306b\u79fb\u308a\u307e\u3059'],
    ['bool accessor contract', 'json_as_bool: JSON Bool payload \u3092\u53d6\u308a\u51fa\u3059'],
    ['number accessor contract', 'json_as_number: JSON Number payload \u3092\u53d6\u308a\u51fa\u3059'],
    ['string accessor contract', 'json_as_string: JSON String payload \u3092\u53d6\u308a\u51fa\u3059'],
];

for (const [label, phrase] of requiredPhrases) {
    assert.equal(src.includes(phrase), true, `${relPath} must document ${label}`);
}

console.log('stdlib json doc boilerplate regression passed');
