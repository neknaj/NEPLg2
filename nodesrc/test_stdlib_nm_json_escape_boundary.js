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
const jsonEscapePath = 'stdlib/nm/json_escape.nepl';
const parser = read(parserPath);
const document = read(documentPath);
const jsonEscape = read(jsonEscapePath);

for (const name of [
    'json_escape_byte_into',
    'json_escape_into',
    'json_escape_builder_into',
    'json_escape',
]) {
    assert.doesNotMatch(
        parser,
        new RegExp(`^(?:pub\\s+)?fn\\s+${name}\\s+`, 'm'),
        `${parserPath} must not own ${name}; JSON escaping belongs to ${jsonEscapePath}`
    );
    assert.doesNotMatch(
        document,
        new RegExp(`^(?:pub\\s+)?fn\\s+${name}\\s+`, 'm'),
        `${documentPath} must not own ${name}; JSON escaping belongs to ${jsonEscapePath}`
    );
    assert.match(
        jsonEscape,
        new RegExp(`^pub\\s+fn\\s+${name}\\s+`, 'm'),
        `${jsonEscapePath} must expose ${name}`
    );
}

assert.match(
    document,
    /^#import "\.\.\/json_escape" as json$/m,
    `${documentPath} must import the dedicated JSON escape module`
);
assert.match(
    document,
    /\bjson::json_escape_into\b/,
    `${documentPath} must call the JSON escape module for string segments`
);
assert.match(
    document,
    /\bjson::json_escape_builder_into\b/,
    `${documentPath} must call the JSON escape module for code block builders`
);

assert.doesNotMatch(
    jsonEscape,
    /\bMemPtr\b|\bload_u8\b|\bstring_data_ptr\b|\bmem_ptr_addr\b/,
    `${jsonEscapePath} must not expose or own raw memory traversal; use alloc/string byte access boundaries`
);

assert.doesNotMatch(
    jsonEscape,
    /^#import "core\/mem" as \*$/m,
    `${jsonEscapePath} must not import core/mem directly`
);

console.log('stdlib nm json escape boundary regression passed');
