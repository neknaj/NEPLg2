#!/usr/bin/env node

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');

const repoRoot = path.resolve(__dirname, '..');
const relPath = 'stdlib/alloc/hash/sha256.nepl';
const src = fs.readFileSync(path.join(repoRoot, relPath), 'utf8');

const code = src
    .split(/\r?\n/)
    .filter((line) => !/^\s*\/\//.test(line))
    .join('\n');

const forbidden = [
    /\bunwrap\b/,
    /\bunwrap_ok\b/,
    /\bunwrap_err\b/,
    /\buwok\b/,
    /\buwerr\b/,
    /#intrinsic\s+"unreachable"/,
];

for (const pattern of forbidden) {
    assert.doesNotMatch(code, pattern, `${relPath} must propagate errors without ${pattern}`);
}

const roundsLoop = code.match(
    /fn sha256_rounds_loop[\s\S]*?(?=\nfn sha256_compress_block)/
);
assert.ok(roundsLoop, 'sha256_rounds_loop must exist');
assert.doesNotMatch(
    roundsLoop[0],
    /Result::Err\s+e:/,
    'sha256_rounds_loop must not shadow the working variable e with an error payload binding'
);

console.log('sha256 unsafe unwrap regression passed');
