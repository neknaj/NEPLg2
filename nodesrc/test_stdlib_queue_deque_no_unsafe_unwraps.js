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
}

const queue = implementationCode('stdlib/alloc/collections/queue.nepl');
assert.match(queue, /struct\s+Queue<\.T>:[\s\S]*len\s+<i32>[\s\S]*cap\s+<i32>[\s\S]*head\s+<i32>[\s\S]*items\s+<Vec<Option<\.T>>>/, 'Queue must keep typed Vec<Option<T>> storage in its public owner struct');
assert.match(queue, /struct\s+QueuePop<\.T>:[\s\S]*queue\s+<Queue<\.T>>[\s\S]*item\s+<Option<\.T>>/, 'Queue must expose an owner-preserving pop result');
assert.match(queue, /fn\s+queue_item_at\s+<\.T:\s*Copy>\s+<\(&Vec<Option<\.T>>,i32\)->Option<\.T>>/, 'Queue must read initialized slot state through Option<T>');
assert.match(queue, /fn\s+queue_store_slot\s+<\.T:\s*Copy>\s+<\(&Vec<Option<\.T>>,i32,Option<\.T>\)\*>\(\)>[\s\S]*vec::replace_ref<Option<\.T>>/, 'Queue must update slot state through Vec<Option<T>> replacement');
assert.match(queue, /fn\s+queue_alloc_slots\s+<\.T:\s*Copy>[\s\S]*vec::filled<Option<\.T>>\s+cap\s+none<\.T>/, 'Queue allocation must initialize every slot as None');
assert.match(queue, /fn\s+pop_front\s+<\.T:\s*Copy>\s+<\(Queue<\.T>\)\*>QueuePop<\.T>>[\s\S]*queue_store_slot<\.T>\s+&items\s+head0\s+none<\.T>[\s\S]*QueuePop<\.T>/, 'Queue pop_front must clear the consumed slot and return the updated owner');
assert.match(queue, /fn\s+free\s+<\.T>\s+<\(Queue<\.T>\)->\(\)>[\s\S]*vec::free<Option<\.T>>\s+field::get\s+q\s+"items"/, 'Queue.free must close the Vec<Option<T>> owner');
assert.doesNotMatch(queue, /\bMemPtr\b|\balloc_ptr\b|\balloc_raw\b|\bdealloc_raw\b|\bload_i32\b|\bstore_i32\b|\bmem_ptr_addr\b/, 'Queue must not reintroduce raw header or raw element storage');

const deque = implementationCode('stdlib/alloc/collections/deque.nepl');
assert.match(deque, /dealloc_raw\s+mem_ptr_addr/, 'Deque must use raw deallocation for its current owned circular-buffer storage until it is migrated');
assert.match(deque, /fn deque_store_header_i32 /, 'Deque must keep owned header writes explicit');

console.log('queue/deque unsafe unwrap regression passed');
