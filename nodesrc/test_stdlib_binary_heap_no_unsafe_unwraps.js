#!/usr/bin/env node

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');

const repoRoot = path.resolve(__dirname, '..');
const relPath = 'stdlib/alloc/collections/binary_heap.nepl';
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

assert.match(code, /struct\s+BinaryHeap<\.T>:[\s\S]*len\s+<i32>[\s\S]*cap\s+<i32>[\s\S]*items\s+<Vec<Option<\.T>>>/, 'BinaryHeap must keep typed Vec<Option<T>> storage in its public owner struct');
assert.match(code, /struct\s+BinaryHeapPop<\.T>:[\s\S]*heap\s+<BinaryHeap<\.T>>[\s\S]*item\s+<Option<\.T>>/, 'BinaryHeap must expose an owner-preserving pop result');
assert.match(code, /fn\s+heap_item_at\s+<\.T:\s*Copy>\s+<\(&Vec<Option<\.T>>,i32\)->Option<\.T>>/, 'BinaryHeap must read initialized slot state through Option<T>');
assert.match(code, /fn\s+heap_store_slot\s+<\.T:\s*Copy>\s+<\(&Vec<Option<\.T>>,i32,Option<\.T>\)\*>\(\)>[\s\S]*vec::replace<Option<\.T>>/, 'BinaryHeap must update slot state through Vec<Option<T>> replacement');
assert.match(code, /fn\s+heap_alloc_slots\s+<\.T:\s*Copy>\s+<\(i32\)\*>Result<Vec<Option<\.T>>, Diag>>[\s\S]*vec::filled<Option<\.T>>\s+cap\s+none<\.T>/, 'BinaryHeap allocation must initialize every slot as None and report allocation failure as Diag');
assert.match(code, /fn\s+new\s+<\.T:\s*Copy>\s+<\(\)\*>Result<BinaryHeap<\.T>, Diag>>/, 'BinaryHeap.new must expose allocation as an impure Result<BinaryHeap<T>, Diag>');
assert.match(code, /fn\s+push\s+<\.T:\s*Ord&Copy>\s+<\(BinaryHeap<\.T>,\.T\)\*>Result<BinaryHeap<\.T>, Diag>>/, 'BinaryHeap.push must expose heap mutation as an impure Result<BinaryHeap<T>, Diag>');
assert.match(code, /fn\s+free\s+<\.T:\s*Copy>\s+<\(BinaryHeap<\.T>\)->\(\)>[\s\S]*vec::free<Option<\.T>>\s+field::get\s+hp\s+"items"/, 'BinaryHeap.free must close the Vec<Option<T>> owner through the Copy storage fast path');
assert.doesNotMatch(code, /\bMemPtr\b|\balloc_ptr\b|\balloc_raw\b|\brealloc_ptr\b|\bdealloc_raw\b|\bload_i32\b|\bstore_i32\b|\bmem_ptr_addr\b|dealloc_ptr/, 'BinaryHeap must not reintroduce raw header or raw element storage');

console.log('binary heap unsafe unwrap regression passed');
