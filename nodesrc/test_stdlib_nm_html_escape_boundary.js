#!/usr/bin/env node

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');

const repoRoot = path.resolve(__dirname, '..');

function read(relPath) {
    return fs.readFileSync(path.join(repoRoot, relPath), 'utf8');
}

const htmlGenPath = 'stdlib/nm/html_gen.nepl';
const htmlEscapePath = 'stdlib/nm/html_escape.nepl';
const htmlGen = read(htmlGenPath);
const htmlEscape = read(htmlEscapePath);

for (const name of ['html_escape_kind', 'escape_html']) {
    assert.doesNotMatch(
        htmlGen,
        new RegExp(`^(?:pub\\s+)?fn\\s+${name}\\s+`, 'm'),
        `${htmlGenPath} must not own ${name}; HTML escaping belongs to ${htmlEscapePath}`
    );
    assert.match(
        htmlEscape,
        new RegExp(`^(?:pub\\s+)?fn\\s+${name}\\s+`, 'm'),
        `${htmlEscapePath} must expose ${name}`
    );
}

assert.doesNotMatch(
    htmlGen,
    /^(?:pub\s+)?enum\s+HtmlEscapeKind:/m,
    `${htmlGenPath} must not own HtmlEscapeKind`
);
assert.match(
    htmlEscape,
    /^(?:pub\s+)?enum\s+HtmlEscapeKind:/m,
    `${htmlEscapePath} must own HtmlEscapeKind`
);
assert.match(
    htmlGen,
    /^#import "\.\/html_escape" as html$/m,
    `${htmlGenPath} must import the dedicated HTML escape module`
);
assert.match(
    htmlGen,
    /\bhtml::escape_html\b/,
    `${htmlGenPath} must call the HTML escape module for text/attribute output`
);

console.log('stdlib nm html escape boundary regression passed');
