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
assert.match(code, /fn\s+len\s+<\.T>\s+<\(&BinaryHeap<\.T>\)->i32>\s+\(hp\):/, 'BinaryHeap.len must borrow the owner');
assert.match(code, /fn\s+cap\s+<\.T>\s+<\(&BinaryHeap<\.T>\)->i32>\s+\(hp\):/, 'BinaryHeap.cap must borrow the owner');
assert.match(code, /fn\s+is_empty\s+<\.T>\s+<\(&BinaryHeap<\.T>\)->bool>\s+\(hp\):/, 'BinaryHeap.is_empty must borrow the owner');
assert.match(code, /fn\s+peek\s+<\.T:\s*Copy>\s+<\(&BinaryHeap<\.T>\)->Option<\.T>>\s+\(hp\):/, 'BinaryHeap.peek must borrow the owner');
assert.doesNotMatch(code, /fn\s+(?:len_ref|cap_ref|is_empty_ref|peek_ref)\b/, 'BinaryHeap must not keep duplicate *_ref observer surfaces');
assert.doesNotMatch(code, /fn\s+(?:len|cap|is_empty|peek)\s+<[^>]+>\s+<\(BinaryHeap<\.T>\)/, 'BinaryHeap observers must not consume the owner');
assert.match(code, /fn\s+free\s+<\.T>\s+<\(BinaryHeap<\.T>\)->\(\)>[\s\S]*vec::free<Option<\.T>>\s+field::get\s+hp\s+"items"/, 'BinaryHeap.free must close the Vec<Option<T>> owner');
assert.doesNotMatch(code, /\bMemPtr\b|\balloc_ptr\b|\balloc_raw\b|\brealloc_ptr\b|\bdealloc_raw\b|\bload_i32\b|\bstore_i32\b|\bmem_ptr_addr\b|dealloc_ptr/, 'BinaryHeap must not reintroduce raw header or raw element storage');

for (const testPath of [
    'stdlib/tests/binary_heap.n.md',
    'tests/stdlib/binary_heap_collections.n.md',
]) {
    const testSrc = fs.readFileSync(path.join(repoRoot, testPath), 'utf8');
    assert.doesNotMatch(testSrc, /\b(?:len_ref|cap_ref|is_empty_ref|peek_ref)<i32>/, `${testPath} must not use removed BinaryHeap *_ref observers`);
    assert.doesNotMatch(testSrc, /\b(?:len|cap|is_empty|peek)(?:<i32>)?\s+hp[0-9]?\b/, `${testPath} must not call BinaryHeap observers by value`);
    assert.doesNotMatch(testSrc, /\bhp[0-9]?\s+\|>\s+peek(?:<i32>)?\b/, `${testPath} must not pipe BinaryHeap owners into peek`);
}

console.log('binary heap unsafe unwrap regression passed');
