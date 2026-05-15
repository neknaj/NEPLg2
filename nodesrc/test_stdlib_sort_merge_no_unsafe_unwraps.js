#!/usr/bin/env node

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');

const repoRoot = path.resolve(__dirname, '..');
const facadePath = 'stdlib/alloc/collections/vec/sort/merge.nepl';
const apiPath = 'stdlib/alloc/collections/vec/sort/merge/api.nepl';
const bufferPath = 'stdlib/alloc/collections/vec/sort/merge/buffer.nepl';
const rangePath = 'stdlib/alloc/collections/vec/sort/merge/range.nepl';
const facadeSrc = sourceWithoutComments(facadePath);
const apiSrc = sourceWithoutComments(apiPath);
const bufferSrc = sourceWithoutComments(bufferPath);
const rangeSrc = sourceWithoutComments(rangePath);
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
];

for (const name of ['sort_merge', 'sort_merge_ret']) {
    const code = extractFunction(name);
    for (const pattern of forbidden) {
        assert.doesNotMatch(code, pattern, `${name} must propagate errors without ${pattern}`);
    }
    assert.deepEqual(
        unexpectedUnreachableLines(code),
        [],
        `${name} must report allocation and cleanup failures through Result without unreachable`,
    );
}

assert.doesNotMatch(facadeSrc, /\bfn\s+/, 'sort/merge facade must not keep implementation bodies');
assert.match(
    facadeSrc,
    /pub\s+#import\s+"\.\/merge\/api"\s+as\s+\*/,
    'sort/merge facade must re-export only merge/api',
);
assert.doesNotMatch(
    facadeSrc,
    /pub\s+#import\s+"\.\/merge\/(?:buffer|range)"\s+as\s+\*/,
    'sort/merge facade must not re-export raw merge buffer/range helpers',
);

assert.match(bufferSrc, /fn\s+sort_buf_get\s+<\.T:\s*Copy>/, 'merge/buffer must own Copy-only scratch buffer reads');
assert.match(bufferSrc, /fn\s+sort_buf_set\s+<\.T:\s*Copy>\s+<\(MemPtr<\.T>,i32,\.T\)\*>\(\)>/, 'merge/buffer must own Copy-only scratch buffer writes');
assert.match(rangeSrc, /fn\s+sort_merge_range_data\s+<\.T:\s*Ord&Copy>/, 'merge/range must own Copy-only merge range traversal');
assert.match(apiSrc, /#import\s+"\.\/range"\s+as\s+\*/, 'merge/api must import raw range traversal explicitly');

console.log('sort merge unsafe unwrap regression passed');

function sourceWithoutComments(file) {
    return fs.readFileSync(path.join(repoRoot, file), 'utf8')
        .split(/\r?\n/)
        .filter((line) => !/^\s*\/\//.test(line))
        .join('\n');
}

function unexpectedUnreachableLines(code) {
    const functionLines = code.split(/\r?\n/);
    const unexpected = [];
    for (let i = 0; i < functionLines.length; i += 1) {
        if (!/#intrinsic\s+"unreachable"/.test(functionLines[i])) continue;
        unexpected.push(`${i + 1}: ${functionLines[i].trim()}`);
    }
    return unexpected;
}
