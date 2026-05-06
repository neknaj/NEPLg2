#!/usr/bin/env node

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');

const repoRoot = path.resolve(__dirname, '..');

function read(relPath) {
    return fs.readFileSync(path.join(repoRoot, relPath), 'utf8');
}

const parserPath = 'stdlib/nm/parser.nepl';
const jsonEscapePath = 'stdlib/nm/json_escape.nepl';
const parser = read(parserPath);
const jsonEscape = read(jsonEscapePath);

for (const name of [
    'json_escape_byte_into',
    'json_escape_mem_into',
    'json_escape_into',
    'json_escape_builder_into',
    'json_escape',
]) {
    assert.doesNotMatch(
        parser,
        new RegExp(`^(?:pub\\s+)?fn\\s+${name}\\s+`, 'm'),
        `${parserPath} must not own ${name}; JSON escaping belongs to ${jsonEscapePath}`
    );
    assert.match(
        jsonEscape,
        new RegExp(`^pub\\s+fn\\s+${name}\\s+`, 'm'),
        `${jsonEscapePath} must expose ${name}`
    );
}

assert.match(
    parser,
    /^#import "\.\/json_escape" as json$/m,
    `${parserPath} must import the dedicated JSON escape module`
);
assert.match(
    parser,
    /\bjson::json_escape_into\b/,
    `${parserPath} must call the JSON escape module for string segments`
);
assert.match(
    parser,
    /\bjson::json_escape_builder_into\b/,
    `${parserPath} must call the JSON escape module for code block builders`
);

console.log('stdlib nm json escape boundary regression passed');
