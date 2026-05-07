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
assert.match(code, /fn\s+len\s+<\.T>\s+<\(&Stack<\.T>\)->i32>\s+\(stk\):/, 'Stack.len must borrow the owner');
assert.match(code, /fn\s+is_empty\s+<\.T>\s+<\(&Stack<\.T>\)->bool>\s+\(stk\):/, 'Stack.is_empty must borrow the owner');
assert.match(code, /fn\s+peek\s+<\.T:\s*Copy>\s+<\(&Stack<\.T>\)->Option<\.T>>\s+\(stk\):/, 'Stack.peek must borrow the owner');
assert.match(code, /fn\s+get\s+<\.T:\s*Copy>\s+<\(&Stack<\.T>,i32\)->Option<\.T>>\s+\(stk,\s*idx\):/, 'Stack.get must borrow the owner');
assert.doesNotMatch(code, /fn\s+(?:len_ref|is_empty_ref|peek_ref|get_ref)\b/, 'Stack must not keep duplicate *_ref observer surfaces');
assert.doesNotMatch(code, /fn\s+(?:len|is_empty|peek|get)\s+<[^>]+>\s+<\(Stack<\.T>\)/, 'Stack observers must not consume the owner');
assert.match(code, /fn\s+free\s+<\.T>\s+<\(Stack<\.T>\)->\(\)>[\s\S]*vec::free<Option<\.T>>\s+field::get\s+stk\s+"items"/, 'Stack.free must close the Vec<Option<T>> owner');
assert.doesNotMatch(code, /\bMemPtr\b|\balloc_ptr\b|\balloc_raw\b|\brealloc_ptr\b|\bdealloc_raw\b|\bload_i32\b|\bstore_i32\b|\bmem_ptr_addr\b|dealloc_ptr/, 'Stack must not reintroduce raw header or raw element storage');

for (const testPath of [
    'stdlib/tests/stack.n.md',
    'tests/stdlib/stack_collections.n.md',
]) {
    const testSrc = fs.readFileSync(path.join(repoRoot, testPath), 'utf8');
    assert.doesNotMatch(testSrc, /\b(?:len_ref|is_empty_ref|peek_ref|get_ref)<i32>/, `${testPath} must not use removed Stack *_ref observers`);
    assert.doesNotMatch(testSrc, /\b(?:len|is_empty|peek|get)(?:<i32>)?\s+s[0-9]?\b/, `${testPath} must not call Stack observers by value`);
    assert.doesNotMatch(testSrc, /\bs[0-9]?\s+\|>\s+(?:len|is_empty|peek|get)(?:<i32>)?\b/, `${testPath} must not pipe Stack owners into observers`);
    assert.match(testSrc, /\b(?:len|peek|get)(?:<i32>)?\s+&s[0-9]?\b/, `${testPath} must exercise borrowed Stack observers through primary names`);
}

for (const rel of [
    'examples/rpn.nepl',
    'examples/rpn_legacy.nepl',
    'examples/bf.nepl',
]) {
    const exampleSrc = fs.readFileSync(path.join(repoRoot, rel), 'utf8');
    assert.doesNotMatch(exampleSrc, /\bstk::(?:len_ref|is_empty_ref|peek_ref|get_ref)\b/, `${rel} must use primary borrowed Stack observer names`);
}

const pipeCollections = fs.readFileSync(path.join(repoRoot, 'tests/stdlib/pipe_collections.n.md'), 'utf8');
assert.match(pipeCollections, /\blen<i32>\s+&s0\b/, 'pipe_collections stack case must borrow Stack.len');
assert.doesNotMatch(pipeCollections, /\blen<i32>\s+s0\b/, 'pipe_collections stack case must not consume Stack.len');

const overloadTests = fs.readFileSync(path.join(repoRoot, 'tests/compiler/overload.n.md'), 'utf8');
assert.doesNotMatch(overloadTests, /\blen_ref<i32>\s+&st\b/, 'overload tests must not use removed Stack.len_ref');

console.log('stack unsafe unwrap regression passed');
