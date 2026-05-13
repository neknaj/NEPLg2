#!/usr/bin/env node

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');

const repoRoot = path.resolve(__dirname, '..');
const relPaths = [
    'stdlib/alloc/hash/sha256.nepl',
    'stdlib/alloc/hash/sha256/types.nepl',
    'stdlib/alloc/hash/sha256/round.nepl',
    'stdlib/alloc/hash/sha256/padding.nepl',
    'stdlib/alloc/hash/sha256/schedule.nepl',
    'stdlib/alloc/hash/sha256/compress.nepl',
    'stdlib/alloc/hash/sha256/digest.nepl',
    'stdlib/alloc/hash/sha256/api.nepl',
];

const sources = new Map(
    relPaths.map((relPath) => [
        relPath,
        fs.readFileSync(path.join(repoRoot, relPath), 'utf8'),
    ])
);

const code = [...sources.values()]
    .join('\n')
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
    assert.doesNotMatch(code, pattern, `sha256 modules must propagate errors without ${pattern}`);
}

const facade = sources.get('stdlib/alloc/hash/sha256.nepl');
assert.doesNotMatch(
    facade,
    /^\s*(struct|fn|impl)\s/m,
    'sha256 root must stay a facade without implementation bodies'
);

for (const required of relPaths.slice(1)) {
    const importPath = required.replace(/^stdlib\//, '').replace(/\.nepl$/, '');
    assert.match(
        facade,
        new RegExp(`#import\\s+"${importPath.replace(/\//g, '\\/')}"\\s+as\\s+\\*`),
        `sha256 facade must re-export ${required}`
    );
}

const roundsLoop = code.match(
    /fn\s+sha256_rounds_loop[\s\S]*?(?=\n(?:pub\s+)?fn\s+sha256_compress_block)/
);
assert.ok(roundsLoop, 'sha256_rounds_loop must exist');
assert.doesNotMatch(
    roundsLoop[0],
    /Result::Err\s+e:/,
    'sha256_rounds_loop must not shadow the working variable e with an error payload binding'
);

console.log('sha256 unsafe unwrap regression passed');
