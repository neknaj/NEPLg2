#!/usr/bin/env node

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');

const repoRoot = path.resolve(__dirname, '..');

function read(relPath) {
    return fs.readFileSync(path.join(repoRoot, relPath), 'utf8');
}

function codeOnly(src) {
    return src
        .split(/\r?\n/)
        .filter((line) => !/^\s*\/\//.test(line))
        .join('\n');
}

const facadePath = 'stdlib/nm/parser.nepl';
const documentPath = 'stdlib/nm/parser/document.nepl';
const facadeSrc = read(facadePath);
const documentSrc = read(documentPath);
const facadeCode = codeOnly(facadeSrc);
const documentCode = codeOnly(documentSrc);

assert.match(
    facadeSrc,
    /^pub #import "\.\/parser\/document" as @merge$/m,
    `${facadePath} must re-export parser/document through the public facade`,
);

for (const pattern of [
    /pub\s+struct\s+Document:/,
    /pub\s+fn\s+parse_markdown\b/,
    /pub\s+fn\s+document_to_json\b/,
    /\bNmJsonSectionState\b/,
    /\bStringBuilder\b/,
    /\bwhile\s+lt\s+pos\s+n:/,
]) {
    assert.doesNotMatch(facadeCode, pattern, `${facadePath} must not reintroduce document parser implementation`);
}

assert.match(documentCode, /pub\s+struct\s+Document:[\s\S]*source\s+<str>/, `${documentPath} must own Document`);
assert.match(documentCode, /pub\s+fn\s+parse_markdown\s+<\(str\)->Document>/, `${documentPath} must own parse_markdown`);
assert.match(documentCode, /pub\s+fn\s+document_to_json\s+<\(Document\)->str>/, `${documentPath} must own document_to_json`);

for (const importLine of [
    '#import "../json_escape" as json',
    '#import "./json_inline" as json_inline',
    '#import "./json_section" as *',
    '#import "./scanner" as scan',
]) {
    assert.equal(documentSrc.includes(importLine), true, `${documentPath} must keep ${importLine}`);
}

assert.ok(facadeSrc.split(/\r?\n/).length <= 60, `${facadePath} must stay within the public facade boundary`);
assert.ok(documentSrc.split(/\r?\n/).length <= 240, `${documentPath} must stay within the document serializer boundary`);

console.log('stdlib nm parser document boundary regression passed');
