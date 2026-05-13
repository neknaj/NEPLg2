#!/usr/bin/env node

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');

const repoRoot = path.resolve(__dirname, '..');
const facadePath = 'stdlib/alloc/collections/vec/sort/merge.nepl';
const apiPath = 'stdlib/alloc/collections/vec/sort/merge/api.nepl';
const facadeSrc = sourceWithoutComments(facadePath);
const apiSrc = sourceWithoutComments(apiPath);
const lines = apiSrc.split(/\r?\n/);

function extractFunction(name) {
    const declaration = new RegExp(`^(?:pub\\s+)?fn\\s+${name}\\s+`);
    const start = lines.findIndex((line) => declaration.test(line));
    assert.notEqual(start, -1, `${name} must exist`);

    let end = lines.length;
    for (let i = start + 1; i < lines.length; i += 1) {
        if (/^(?:pub\s+)?fn\s+/.test(lines[i])) {
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

assert.doesNotMatch(facadeSrc, /\bfn\s+/, 'sort/merge facade must not keep implementation bodies');
for (const submodule of ['buffer', 'range', 'api']) {
    assert.match(
        facadeSrc,
        new RegExp(`pub\\s+#import\\s+"\\.\\/merge\\/${submodule}"\\s+as\\s+\\*`),
        `sort/merge facade must re-export merge/${submodule}`,
    );
}

console.log('sort merge unsafe unwrap regression passed');

function sourceWithoutComments(file) {
    return fs.readFileSync(path.join(repoRoot, file), 'utf8')
        .split(/\r?\n/)
        .filter((line) => !/^\s*\/\//.test(line))
        .join('\n');
}
