#!/usr/bin/env node

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');

const repoRoot = path.resolve(__dirname, '..');
const relPath = 'stdlib/alloc/collections/vec/sort.nepl';
const src = fs.readFileSync(path.join(repoRoot, relPath), 'utf8');
const lines = src.split(/\r?\n/);

function extractFunction(name) {
    const start = lines.findIndex((line) => line.startsWith(`fn ${name} `));
    assert.notEqual(start, -1, `${name} must exist`);

    let end = lines.length;
    for (let i = start + 1; i < lines.length; i += 1) {
        if (lines[i].startsWith('fn ')) {
            end = i;
            break;
        }
    }
    return lines.slice(start, end).join('\n');
}

const forbidden = [
    /\bunwrap\b/,
    /\bunwrap_ok\b/,
    /\bunwrap_err\b/,
    /\buwok\b/,
    /\buwerr\b/,
    /#intrinsic\s+"unreachable"/,
];

for (const name of ['sort_merge', 'sort_merge_ret']) {
    const code = extractFunction(name);
    for (const pattern of forbidden) {
        assert.doesNotMatch(code, pattern, `${name} must propagate errors without ${pattern}`);
    }
}

console.log('sort merge unsafe unwrap regression passed');
