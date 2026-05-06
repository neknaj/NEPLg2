#!/usr/bin/env node

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');

const repoRoot = path.resolve(__dirname, '..');
const relPath = 'stdlib/nm/parser.nepl';
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
    ['Document source-view contract', 'Document: NM document \u306e source view'],
    ['inline JSON contract', 'nm_inline_to_json: inline markup \u3092 JSON array \u306b\u5909\u63db\u3059\u308b'],
    ['document JSON contract', 'document_to_json: `Document` \u3092 JSON \u6587\u5b57\u5217\u3078\u5909\u63db\u3059\u308b'],
];

for (const [label, phrase] of requiredPhrases) {
    assert.equal(src.includes(phrase), true, `${relPath} must document ${label}`);
}

console.log('stdlib nm/parser doc boilerplate regression passed');
