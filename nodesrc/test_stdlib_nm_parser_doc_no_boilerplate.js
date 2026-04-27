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
    ['fence parser return contract', 'FenceRes: fenced code block parser \u306e\u623b\u308a\u5024'],
    ['paragraph parser return contract', 'ParaRes: paragraph parser \u306e\u623b\u308a\u5024'],
    ['heading predicate contract', 'is_heading_start: ATX heading \u884c\u3092\u5224\u5b9a\u3059\u308b'],
    ['section close contract', 'close_one_section: \u958b\u3044\u3066\u3044\u308b section \u3092 1 \u3064\u9589\u3058\u308b'],
    ['fenced block parser contract', 'parse_fence: fenced code block \u3092\u8aad\u307f\u53d6\u308b'],
    ['paragraph parser contract', 'parse_paragraph: paragraph block \u3092\u8aad\u307f\u53d6\u308b'],
    ['JSON escape contract', 'json_escape: JSON \u6587\u5b57\u5217\u7528\u306b byte \u3092 escape \u3059\u308b'],
    ['inline JSON contract', 'inlines_to_json: inline AST \u5217\u3092 JSON \u914d\u5217\u306b\u5909\u63db\u3059\u308b'],
];

for (const [label, phrase] of requiredPhrases) {
    assert.equal(src.includes(phrase), true, `${relPath} must document ${label}`);
}

console.log('stdlib nm/parser doc boilerplate regression passed');
