#!/usr/bin/env node

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');

const repoRoot = path.resolve(__dirname, '..');

function read(relPath) {
    return fs.readFileSync(path.join(repoRoot, relPath), 'utf8');
}

const htmlGenPath = 'stdlib/nm/html_gen.nepl';
const htmlInlinePath = 'stdlib/nm/html_inline.nepl';
const htmlGen = read(htmlGenPath);
const htmlInline = read(htmlInlinePath);

assert.doesNotMatch(
    htmlGen,
    /^(?:pub\s+)?fn\s+nm_inline_to_html\s+/m,
    `${htmlGenPath} must not own nm_inline_to_html; inline HTML serialization belongs to ${htmlInlinePath}`
);
assert.match(
    htmlInline,
    /^pub\s+fn\s+nm_inline_to_html\s+<\(str\)->str>/m,
    `${htmlInlinePath} must expose nm_inline_to_html`
);
assert.match(
    htmlGen,
    /^#import "\.\/html_inline" as inline$/m,
    `${htmlGenPath} must import the dedicated HTML inline module`
);
assert.match(
    htmlGen,
    /\binline::nm_inline_to_html\b/,
    `${htmlGenPath} must call the HTML inline module for inline content`
);
assert.match(
    htmlInline,
    /match\s+ch:/,
    `${htmlInlinePath} must keep marker dispatch as a match`
);
assert.match(
    htmlInline,
    /\bnm_inline_to_html\s+main\b/,
    `${htmlInlinePath} must keep recursive gloss main rendering inside the inline module`
);

console.log('stdlib nm html inline boundary regression passed');
