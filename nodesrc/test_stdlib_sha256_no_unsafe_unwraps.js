#!/usr/bin/env node

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const { legacyTypeSyntaxView } = require('./source_policy/nepl_source_view');

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

const code = legacyTypeSyntaxView([...sources.values()].join('\n'));

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

const api = legacyTypeSyntaxView(sources.get('stdlib/alloc/hash/sha256/api.nepl'));
assert.doesNotMatch(
    api,
    /\bctx\.buffer\b/,
    'sha256 api must use explicit owner aggregate field accessors so source proof is visible'
);

const types = legacyTypeSyntaxView(sources.get('stdlib/alloc/hash/sha256/types.nepl'));
assert.match(
    types,
    /pub\s+fn\s+sha256_update_error_kind\s+<\(&Sha256UpdateError\)->StdErrorKind>/,
    'Sha256UpdateError must expose a borrowed error-kind accessor'
);
assert.match(
    types,
    /pub\s+fn\s+sha256_update_error_ctx\s+<\(Sha256UpdateError\)->Sha256>/,
    'Sha256UpdateError must expose an owner-consuming ctx accessor'
);

console.log('sha256 unsafe unwrap regression passed');
