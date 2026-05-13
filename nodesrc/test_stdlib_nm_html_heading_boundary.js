#!/usr/bin/env node

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');

const repoRoot = path.resolve(__dirname, '..');

function read(relPath) {
    return fs.readFileSync(path.join(repoRoot, relPath), 'utf8');
}

const htmlGenPath = 'stdlib/nm/html_gen.nepl';
const htmlHeadingPath = 'stdlib/nm/html_heading.nepl';
const htmlGen = read(htmlGenPath);
const htmlHeading = read(htmlHeadingPath);

for (const name of ['html_heading_kind', 'nm_append_heading_open', 'nm_append_heading_close']) {
    assert.doesNotMatch(
        htmlGen,
        new RegExp(`^(?:pub\\s+)?fn\\s+${name}\\s+`, 'm'),
        `${htmlGenPath} must not own ${name}; heading tag helpers belong to ${htmlHeadingPath}`
    );
    assert.match(
        htmlHeading,
        new RegExp(`^(?:pub\\s+)?fn\\s+${name}\\s+`, 'm'),
        `${htmlHeadingPath} must expose ${name}`
    );
}

assert.doesNotMatch(
    htmlGen,
    /^(?:pub\s+)?enum\s+HtmlHeadingKind:/m,
    `${htmlGenPath} must not own HtmlHeadingKind`
);
assert.match(
    htmlHeading,
    /^(?:pub\s+)?enum\s+HtmlHeadingKind:/m,
    `${htmlHeadingPath} must own HtmlHeadingKind`
);
assert.match(
    htmlGen,
    /^#import "\.\/html_heading" as heading$/m,
    `${htmlGenPath} must import the dedicated HTML heading module`
);
assert.match(
    htmlGen,
    /\bheading::nm_append_heading_open\b/,
    `${htmlGenPath} must call the heading module for heading open tags`
);
assert.match(
    htmlGen,
    /\bheading::nm_append_heading_close\b/,
    `${htmlGenPath} must call the heading module for heading close tags`
);

console.log('stdlib nm html heading boundary regression passed');
