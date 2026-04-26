#!/usr/bin/env node

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');

const repoRoot = path.resolve(__dirname, '..');
const relPaths = [
    'stdlib/alloc/collections/queue.nepl',
    'stdlib/alloc/collections/deque.nepl',
];

const forbidden = [
    /\bunwrap\b/,
    /\bunwrap_ok\b/,
    /\bunwrap_err\b/,
    /\buwok\b/,
    /\buwerr\b/,
    /#intrinsic\s+"unreachable"/,
];

function implementationCode(relPath) {
    const src = fs.readFileSync(path.join(repoRoot, relPath), 'utf8');
    return src
        .split(/\r?\n/)
        .filter((line) => !/^\s*\/\//.test(line))
        .join('\n');
}

for (const relPath of relPaths) {
    const code = implementationCode(relPath);
    for (const pattern of forbidden) {
        assert.doesNotMatch(code, pattern, `${relPath} must not use unsafe unwrap helpers in implementation code`);
    }
    assert.match(code, /dealloc_raw\s+mem_ptr_addr/, `${relPath} must use raw deallocation for owned circular-buffer storage`);
}

const queue = implementationCode('stdlib/alloc/collections/queue.nepl');
assert.match(queue, /fn queue_store_header_i32 /, 'Queue must keep owned header writes explicit');

const deque = implementationCode('stdlib/alloc/collections/deque.nepl');
assert.match(deque, /fn deque_store_header_i32 /, 'Deque must keep owned header writes explicit');

console.log('queue/deque unsafe unwrap regression passed');
