#!/usr/bin/env node

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');

const repoRoot = path.resolve(__dirname, '..');
const relPath = 'stdlib/alloc/collections/stack.nepl';
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

assert.match(code, /struct\s+Stack<\.T>:[\s\S]*len\s+<i32>[\s\S]*cap\s+<i32>[\s\S]*items\s+<Vec<Option<\.T>>>/, 'Stack must keep typed Vec<Option<T>> storage in its public owner struct');
assert.match(code, /struct\s+StackPop<\.T>:[\s\S]*stack\s+<Stack<\.T>>[\s\S]*item\s+<Option<\.T>>/, 'Stack must expose an owner-preserving pop result');
assert.match(code, /fn\s+stack_item_at\s+<\.T:\s*Copy>\s+<\(&Vec<Option<\.T>>,i32\)->Option<\.T>>/, 'Stack must read initialized slot state through Option<T>');
assert.match(code, /fn\s+stack_store_slot\s+<\.T:\s*Copy>\s+<\(&Vec<Option<\.T>>,i32,Option<\.T>\)\*>\(\)>[\s\S]*vec::replace<Option<\.T>>/, 'Stack must update slot state through Vec<Option<T>> replacement');
assert.match(code, /fn\s+stack_alloc_slots\s+<\.T:\s*Copy>[\s\S]*vec::filled<Option<\.T>>\s+cap\s+none<\.T>/, 'Stack allocation must initialize every slot as None');
assert.match(code, /fn\s+pop_top\s+<\.T:\s*Copy>\s+<\(Stack<\.T>\)\*>StackPop<\.T>>[\s\S]*stack_store_slot<\.T>\s+&items\s+next_len\s+none<\.T>[\s\S]*StackPop<\.T>/, 'Stack pop_top must clear the consumed slot and return the updated owner');
assert.match(code, /fn\s+free\s+<\.T>\s+<\(Stack<\.T>\)->\(\)>[\s\S]*vec::free<Option<\.T>>\s+field::get\s+stk\s+"items"/, 'Stack.free must close the Vec<Option<T>> owner');
assert.doesNotMatch(code, /\bMemPtr\b|\balloc_ptr\b|\balloc_raw\b|\brealloc_ptr\b|\bdealloc_raw\b|\bload_i32\b|\bstore_i32\b|\bmem_ptr_addr\b|dealloc_ptr/, 'Stack must not reintroduce raw header or raw element storage');

console.log('stack unsafe unwrap regression passed');
