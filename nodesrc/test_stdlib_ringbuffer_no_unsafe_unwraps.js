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

assert.match(code, /struct\s+RingBuffer<\.T>:[\s\S]*len\s+<i32>[\s\S]*cap\s+<i32>[\s\S]*head\s+<i32>[\s\S]*items\s+<Vec<Option<\.T>>>/, 'RingBuffer must keep typed Vec<Option<T>> storage in its public owner struct');
assert.match(code, /struct\s+RingBufferPop<\.T>:[\s\S]*buffer\s+<RingBuffer<\.T>>[\s\S]*item\s+<Option<\.T>>/, 'RingBuffer must expose an owner-preserving pop result');
assert.match(code, /fn\s+ringbuffer_item_at\s+<\.T:\s*Copy>\s+<\(&Vec<Option<\.T>>,i32\)->Option<\.T>>/, 'RingBuffer must read initialized slot state through Option<T>');
assert.match(code, /fn\s+ringbuffer_store_slot\s+<\.T:\s*Copy>\s+<\(&Vec<Option<\.T>>,i32,Option<\.T>\)\*>\(\)>[\s\S]*vec::replace<Option<\.T>>/, 'RingBuffer must update slot state through Vec<Option<T>> replacement');
assert.match(code, /fn\s+ringbuffer_alloc_slots\s+<\.T:\s*Copy>[\s\S]*vec::filled<Option<\.T>>\s+cap\s+none<\.T>/, 'RingBuffer allocation must initialize every slot as None');
assert.match(code, /fn\s+pop_front\s+<\.T:\s*Copy>\s+<\(RingBuffer<\.T>\)\*>RingBufferPop<\.T>>[\s\S]*ringbuffer_store_slot<\.T>\s+&items\s+head0\s+none<\.T>[\s\S]*RingBufferPop<\.T>/, 'RingBuffer pop_front must clear the consumed slot and return the updated owner');
assert.match(code, /fn\s+free\s+<\.T>\s+<\(RingBuffer<\.T>\)->\(\)>[\s\S]*vec::free<Option<\.T>>\s+field::get\s+rb\s+"items"/, 'RingBuffer.free must close the Vec<Option<T>> owner');
assert.doesNotMatch(code, /\bMemPtr\b|\balloc_ptr\b|\balloc_raw\b|\bdealloc_raw\b|\bload_i32\b|\bstore_i32\b|\bmem_ptr_addr\b|dealloc_ptr/, 'RingBuffer must not reintroduce raw header or raw element storage');

console.log('ringbuffer unsafe unwrap regression passed');
