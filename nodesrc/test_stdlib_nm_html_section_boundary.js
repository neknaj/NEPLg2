#!/usr/bin/env node

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const { legacyTypeSyntaxView } = require('./source_policy/nepl_source_view');

const repoRoot = path.resolve(__dirname, '..');

function read(relPath) {
    return fs.readFileSync(path.join(repoRoot, relPath), 'utf8');
}

const htmlGenPath = 'stdlib/nm/html_gen.nepl';
const htmlSectionPath = 'stdlib/nm/html_section.nepl';
const htmlGen = legacyTypeSyntaxView(read(htmlGenPath));
const htmlSection = legacyTypeSyntaxView(read(htmlSectionPath));

assert.match(
    htmlSection,
    /pub\s+struct\s+NmHtmlSectionState:/,
    `${htmlSectionPath} must own the HTML section stack state`,
);
assert.match(
    htmlSection,
    /pub\s+fn\s+nm_section_state_close_current\s+<\(NmHtmlSectionState\)->NmHtmlSectionState>/,
    `${htmlSectionPath} must expose close-current state transition`,
);
assert.match(
    htmlSection,
    /pub\s+fn\s+nm_append_section_open\s+<\(StringBuilder,i32\)->StringBuilder>/,
    `${htmlSectionPath} must expose section open tag helper`,
);
assert.match(
    htmlSection,
    /pub\s+fn\s+nm_append_section_close\s+<\(StringBuilder\)->StringBuilder>/,
    `${htmlSectionPath} must expose section close tag helper`,
);
assert.match(
    htmlSection,
    /match\s+state\.current_level:/,
    `${htmlSectionPath} must keep section close dispatch explicit`,
);

assert.match(
    htmlGen,
    /^#import "\.\/html_section" as \*$/m,
    `${htmlGenPath} must import the dedicated section helper module`,
);
assert.match(
    htmlGen,
    /\bnm_section_state_new\b/,
    `${htmlGenPath} must initialize section state through html_section`,
);
assert.match(
    htmlGen,
    /\bnm_section_state_close_current\b/,
    `${htmlGenPath} must close sections through html_section`,
);
assert.match(
    htmlGen,
    /\bnm_append_section_open\b/,
    `${htmlGenPath} must append section open tags through html_section`,
);
assert.doesNotMatch(
    htmlGen,
    /\blet\s+mut\s+open[1-6]\s+<bool>/,
    `${htmlGenPath} must not reintroduce ad hoc section open flags`,
);
assert.doesNotMatch(
    htmlGen,
    /match\s+current_level:/,
    `${htmlGenPath} must not reintroduce duplicated current_level close dispatch`,
);

console.log('stdlib nm html section boundary regression passed');
