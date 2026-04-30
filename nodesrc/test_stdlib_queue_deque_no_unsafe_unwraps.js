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
assert.match(queue, /fn\s+queue_store_slot\s+<\.T:\s*Copy>\s+<\(&Vec<Option<\.T>>,i32,Option<\.T>\)\*>\(\)>[\s\S]*vec::replace<Option<\.T>>/, 'Queue must update slot state through Vec<Option<T>> replacement');
assert.match(queue, /fn\s+queue_alloc_slots\s+<\.T:\s*Copy>[\s\S]*vec::filled<Option<\.T>>\s+cap\s+none<\.T>/, 'Queue allocation must initialize every slot as None');
assert.match(queue, /fn\s+pop_front\s+<\.T:\s*Copy>\s+<\(Queue<\.T>\)\*>QueuePop<\.T>>[\s\S]*queue_store_slot<\.T>\s+&items\s+head0\s+none<\.T>[\s\S]*QueuePop<\.T>/, 'Queue pop_front must clear the consumed slot and return the updated owner');
assert.match(queue, /fn\s+free\s+<\.T>\s+<\(Queue<\.T>\)->\(\)>[\s\S]*vec::free<Option<\.T>>\s+field::get\s+q\s+"items"/, 'Queue.free must close the Vec<Option<T>> owner');
assert.doesNotMatch(queue, /\bMemPtr\b|\balloc_ptr\b|\balloc_raw\b|\bdealloc_raw\b|\bload_i32\b|\bstore_i32\b|\bmem_ptr_addr\b/, 'Queue must not reintroduce raw header or raw element storage');

const deque = implementationCode('stdlib/alloc/collections/deque.nepl');
assert.match(deque, /struct\s+Deque<\.T>:[\s\S]*len\s+<i32>[\s\S]*cap\s+<i32>[\s\S]*head\s+<i32>[\s\S]*items\s+<Vec<Option<\.T>>>/, 'Deque must keep typed Vec<Option<T>> storage in its public owner struct');
assert.match(deque, /fn\s+deque_item_at\s+<\.T:\s*Copy>\s+<\(&Vec<Option<\.T>>,i32\)->Option<\.T>>/, 'Deque must read initialized slot state through Option<T>');
assert.match(deque, /fn\s+deque_store_slot\s+<\.T:\s*Copy>\s+<\(&Vec<Option<\.T>>,i32,Option<\.T>\)\*>\(\)>[\s\S]*vec::replace<Option<\.T>>/, 'Deque must update slot state through Vec<Option<T>> replacement');
assert.match(deque, /fn\s+deque_alloc_slots\s+<\.T:\s*Copy>[\s\S]*vec::filled<Option<\.T>>\s+cap\s+none<\.T>/, 'Deque allocation must initialize every slot as None');
assert.match(deque, /fn\s+push_front\s+<\.T:\s*Copy>[\s\S]*deque_prev_index[\s\S]*deque_store_slot<\.T>\s+&items\s+head1\s+some<\.T>\s+item/, 'Deque push_front must write a typed Some slot at the new head');
assert.match(deque, /fn\s+push_back\s+<\.T:\s*Copy>[\s\S]*deque_tail_index[\s\S]*deque_store_slot<\.T>\s+&items\s+tail\s+some<\.T>\s+item/, 'Deque push_back must write a typed Some slot at the tail');
assert.match(deque, /fn\s+free\s+<\.T>\s+<\(Deque<\.T>\)->\(\)>[\s\S]*vec::free<Option<\.T>>\s+field::get\s+dq\s+"items"/, 'Deque.free must close the Vec<Option<T>> owner');
assert.doesNotMatch(deque, /\bMemPtr\b|\balloc_ptr\b|\balloc_raw\b|\bdealloc_raw\b|\bload_i32\b|\bstore_i32\b|\bmem_ptr_addr\b/, 'Deque must not reintroduce raw header or raw element storage');

console.log('queue/deque unsafe unwrap regression passed');
