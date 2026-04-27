#!/usr/bin/env node

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');

const repoRoot = path.resolve(__dirname, '..');
const relPath = 'stdlib/alloc/collections/ringbuffer.nepl';
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
    assert.doesNotMatch(code, pattern, `${relPath} must not use unsafe unwrap helpers in implementation code`);
}

assert.match(code, /fn\s+ringbuffer_store_header_i32\s+/, 'RingBuffer must centralize owned header writes');
assert.match(code, /fn\s+free\s+<\.T>\s+<\(RingBuffer<\.T>\)\*>\(\)>[\s\S]*dealloc_raw\s+mem_ptr_addr\s+data[\s\S]*dealloc_raw\s+mem_ptr_addr\s+hdr\s+16/, 'RingBuffer.free must use raw owner cleanup for data and header storage');
assert.doesNotMatch(code, /dealloc_ptr/, 'RingBuffer must not use checked deallocation for owned internals');

console.log('ringbuffer unsafe unwrap regression passed');
