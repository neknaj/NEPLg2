#!/usr/bin/env node

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');

const repoRoot = path.resolve(__dirname, '..');

function read(relPath) {
    return fs.readFileSync(path.join(repoRoot, relPath), 'utf8');
}

const parserPath = 'stdlib/nm/parser.nepl';
const documentPath = 'stdlib/nm/parser/document.nepl';
const scannerPath = 'stdlib/nm/parser/scanner.nepl';
const jsonInlinePath = 'stdlib/nm/parser/json_inline.nepl';
const jsonSectionPath = 'stdlib/nm/parser/json_section.nepl';
const htmlPath = 'stdlib/nm/html_gen.nepl';
const htmlInlinePath = 'stdlib/nm/html_inline.nepl';
const htmlSectionPath = 'stdlib/nm/html_section.nepl';
const parser = read(parserPath);
const document = read(documentPath);
const scanner = read(scannerPath);
const jsonInline = read(jsonInlinePath);
const jsonSection = read(jsonSectionPath);
const html = read(htmlPath);
const htmlInline = read(htmlInlinePath);
const htmlSection = read(htmlSectionPath);

for (const name of [
    'nm_read_line',
    'nm_heading_level',
    'nm_heading_text',
    'is_fence_start',
    'is_hr',
    'is_section_break',
    'nm_is_block_boundary',
    'nm_deepest_level',
    'nm_find_math_end',
    'nm_find_gloss_slash',
]) {
    assert.doesNotMatch(
        parser,
        new RegExp(`^(?:pub\\s+)?fn\\s+${name}\\s+`, 'm'),
        `${parserPath} must not own ${name}; NM scan helpers belong to ${scannerPath}`
    );
    assert.doesNotMatch(
        document,
        new RegExp(`^(?:pub\\s+)?fn\\s+${name}\\s+`, 'm'),
        `${documentPath} must not own ${name}; NM scan helpers belong to ${scannerPath}`
    );
    assert.match(
        scanner,
        new RegExp(`^pub\\s+fn\\s+${name}\\s+`, 'm'),
        `${scannerPath} must expose ${name}`
    );
    assert.doesNotMatch(
        jsonInline,
        new RegExp(`^(?:pub\\s+)?fn\\s+${name}\\s+`, 'm'),
        `${jsonInlinePath} must not own ${name}; NM scan helpers belong to ${scannerPath}`
    );
    assert.doesNotMatch(
        jsonSection,
        new RegExp(`^(?:pub\\s+)?fn\\s+${name}\\s+`, 'm'),
        `${jsonSectionPath} must not own ${name}; NM scan helpers belong to ${scannerPath}`
    );
    assert.doesNotMatch(
        html,
        new RegExp(`^(?:pub\\s+)?fn\\s+${name}\\s+`, 'm'),
        `${htmlPath} must not own ${name}; NM scan helpers belong to ${scannerPath}`
    );
    assert.doesNotMatch(
        htmlInline,
        new RegExp(`^(?:pub\\s+)?fn\\s+${name}\\s+`, 'm'),
        `${htmlInlinePath} must not own ${name}; NM scan helpers belong to ${scannerPath}`
    );
    assert.doesNotMatch(
        htmlSection,
        new RegExp(`^(?:pub\\s+)?fn\\s+${name}\\s+`, 'm'),
        `${htmlSectionPath} must not own ${name}; NM scan helpers belong to ${scannerPath}`
    );
}

assert.match(
    parser,
    /^pub #import "\.\/parser\/document" as @merge$/m,
    `${parserPath} must re-export the dedicated document parser module`
);
assert.match(
    document,
    /^#import "\.\/scanner" as scan$/m,
    `${documentPath} must import the dedicated NM parser scanner module`
);
assert.match(
    jsonInline,
    /^#import "\.\/scanner" as scan$/m,
    `${jsonInlinePath} must import the dedicated NM parser scanner module`
);
assert.match(
    jsonSection,
    /^#import "\.\/scanner" as scan$/m,
    `${jsonSectionPath} must import the dedicated NM parser scanner module`
);
assert.match(
    html,
    /^#import "\.\/parser\/scanner" as scan$/m,
    `${htmlPath} must import the dedicated NM parser scanner module`
);
assert.match(
    htmlInline,
    /^#import "\.\/parser\/scanner" as scan$/m,
    `${htmlInlinePath} must import the dedicated NM parser scanner module`
);
assert.match(
    htmlSection,
    /^#import "\.\/parser\/scanner" as scan$/m,
    `${htmlSectionPath} must import the dedicated NM parser scanner module`
);
assert.match(
    htmlInline,
    /\bscan::nm_find_math_end\b/,
    `${htmlInlinePath} must use scanner module for math delimiter search`
);
assert.match(
    htmlInline,
    /\bscan::nm_find_gloss_slash\b/,
    `${htmlInlinePath} must use scanner module for gloss slash search`
);
assert.match(
    document,
    /\bscan::nm_heading_level\b/,
    `${documentPath} must use scanner module for heading classification`
);
assert.match(
    html,
    /\bscan::nm_heading_level\b/,
    `${htmlPath} must use scanner module for heading classification`
);
assert.match(
    jsonInline,
    /\bscan::nm_find_math_end\b/,
    `${jsonInlinePath} must use scanner module for math delimiter search`
);
assert.match(
    jsonInline,
    /\bscan::nm_find_gloss_slash\b/,
    `${jsonInlinePath} must use scanner module for gloss slash search`
);
assert.match(
    jsonSection,
    /\bscan::nm_deepest_level\b/,
    `${jsonSectionPath} must use scanner module for section depth calculation`
);
assert.match(
    htmlSection,
    /\bscan::nm_deepest_level\b/,
    `${htmlSectionPath} must use scanner module for section depth calculation`
);
assert.match(
    scanner,
    /nm\/parser\/scanner: NM parser の line \/ delimiter scanner helper/,
    `${scannerPath} must document its scanner responsibility`
);

console.log('stdlib nm parser scanner boundary regression passed');
