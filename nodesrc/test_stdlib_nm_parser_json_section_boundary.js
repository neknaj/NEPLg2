#!/usr/bin/env node

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const { legacyTypeSyntaxView } = require('./source_policy/nepl_source_view');

const repoRoot = path.resolve(__dirname, '..');

function read(relPath) {
    return fs.readFileSync(path.join(repoRoot, relPath), 'utf8');
}

const parserPath = 'stdlib/nm/parser.nepl';
const documentPath = 'stdlib/nm/parser/document.nepl';
const jsonSectionPath = 'stdlib/nm/parser/json_section.nepl';
const parser = legacyTypeSyntaxView(read(parserPath));
const document = legacyTypeSyntaxView(read(documentPath));
const jsonSection = legacyTypeSyntaxView(read(jsonSectionPath));

assert.match(
    jsonSection,
    /pub\s+struct\s+NmJsonSectionState:/,
    `${jsonSectionPath} must own JSON section state`,
);
assert.match(
    jsonSection,
    /pub\s+fn\s+nm_json_section_needs_comma\s+<\(NmJsonSectionState\)->bool>/,
    `${jsonSectionPath} must expose comma state query`,
);
assert.match(
    jsonSection,
    /pub\s+fn\s+nm_json_section_mark_current_has\s+<\(NmJsonSectionState\)->NmJsonSectionState>/,
    `${jsonSectionPath} must expose current-container mark transition`,
);
assert.match(
    jsonSection,
    /pub\s+fn\s+nm_json_section_close_current\s+<\(NmJsonSectionState\)->NmJsonSectionState>/,
    `${jsonSectionPath} must expose close-current transition`,
);
assert.match(
    jsonSection,
    /match\s+state\.current_level:/,
    `${jsonSectionPath} must keep section state dispatch explicit`,
);

assert.match(
    document,
    /^#import "\.\/json_section" as \*$/m,
    `${documentPath} must import the dedicated JSON section state module`,
);
for (const symbol of [
    'nm_json_section_state_new',
    'nm_json_section_current_level',
    'nm_json_section_needs_comma',
    'nm_json_section_mark_current_has',
    'nm_json_section_open_level',
    'nm_json_section_close_current',
]) {
    assert.match(document, new RegExp(`\\b${symbol}\\b`), `${documentPath} must call ${symbol}`);
}

for (const pattern of [
    /\blet\s+mut\s+current_level\s+<i32>/,
    /\blet\s+mut\s+open[1-6]\s+<bool>/,
    /\blet\s+mut\s+root_has\s+<bool>/,
    /\blet\s+mut\s+has[1-6]\s+<bool>/,
    /\bfn\s+nm_json_needs_comma\b/,
    /match\s+current_level:/,
]) {
    assert.doesNotMatch(parser, pattern, `${parserPath} must not reintroduce ad hoc JSON section state`);
    assert.doesNotMatch(document, pattern, `${documentPath} must not reintroduce ad hoc JSON section state`);
}

console.log('stdlib nm parser JSON section boundary regression passed');
