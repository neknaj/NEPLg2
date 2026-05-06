#!/usr/bin/env node

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');

const repoRoot = path.resolve(__dirname, '..');

function read(relPath) {
    return fs.readFileSync(path.join(repoRoot, relPath), 'utf8');
}

const parserPath = 'stdlib/nm/parser.nepl';
const scannerPath = 'stdlib/nm/parser/scanner.nepl';
const htmlPath = 'stdlib/nm/html_gen.nepl';
const parser = read(parserPath);
const scanner = read(scannerPath);
const html = read(htmlPath);

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
    assert.match(
        scanner,
        new RegExp(`^pub\\s+fn\\s+${name}\\s+`, 'm'),
        `${scannerPath} must expose ${name}`
    );
    assert.doesNotMatch(
        html,
        new RegExp(`^(?:pub\\s+)?fn\\s+${name}\\s+`, 'm'),
        `${htmlPath} must not own ${name}; NM scan helpers belong to ${scannerPath}`
    );
}

assert.match(
    parser,
    /^#import "\.\/parser\/scanner" as scan$/m,
    `${parserPath} must import the dedicated NM parser scanner module`
);
assert.match(
    html,
    /^#import "\.\/parser\/scanner" as scan$/m,
    `${htmlPath} must import the dedicated NM parser scanner module`
);
assert.match(
    parser,
    /\bscan::nm_find_math_end\b/,
    `${parserPath} must use scanner module for math delimiter search`
);
assert.match(
    html,
    /\bscan::nm_find_math_end\b/,
    `${htmlPath} must use scanner module for math delimiter search`
);
assert.match(
    parser,
    /\bscan::nm_heading_level\b/,
    `${parserPath} must use scanner module for heading classification`
);
assert.match(
    html,
    /\bscan::nm_heading_level\b/,
    `${htmlPath} must use scanner module for heading classification`
);
assert.match(
    scanner,
    /nm\/parser\/scanner: NM parser の line \/ delimiter scanner helper/,
    `${scannerPath} must document its scanner responsibility`
);

console.log('stdlib nm parser scanner boundary regression passed');
