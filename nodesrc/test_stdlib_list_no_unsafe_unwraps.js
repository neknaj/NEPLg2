#!/usr/bin/env node

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');

const repoRoot = path.resolve(__dirname, '..');
const relPath = 'stdlib/alloc/collections/list.nepl';
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
    /dealloc_ptr/,
];

for (const pattern of forbidden) {
    assert.doesNotMatch(code, pattern, `${relPath} must not use unsafe unwrap or checked deallocation helpers in implementation code`);
}

const reverseMatch = code.match(/fn\s+reverse\s+<\.T>[\s\S]*?(?=\nfn\s+list_map_impl\b)/);
assert(reverseMatch, 'List.reverse implementation must be present');
const reverseCode = reverseMatch[0];

assert.match(reverseCode, /fn\s+reverse\s+<\.T>\s+<\(List<\.T>\)\*>List<\.T>>/, 'reverse must be an infallible owner-moving operation');
assert.match(reverseCode, /let\s+mut\s+prev\s+<i32>\s+0[\s\S]*let\s+mut\s+cur\s+<i32>\s+field::get\s+lst\s+"ptr"/, 'reverse must consume the input owner pointer');
assert.match(reverseCode, /let\s+next_ptr\s+<i32>\s+load_i32\s+add\s+cur\s+size_of<\.T>[\s\S]*store_i32\s+add\s+cur\s+size_of<\.T>\s+prev[\s\S]*set\s+prev\s+cur[\s\S]*set\s+cur\s+next_ptr/, 'reverse must relink existing nodes instead of copying payloads');
assert.match(reverseCode, /List\s+prev/, 'reverse must return the relinked owner');
assert.doesNotMatch(reverseCode, /list_alloc_node|load<\.T>\s+cur|Result::Err|diag_err/, 'reverse must not allocate, duplicate payloads, or expose an allocation failure path');

assert.match(code, /fn\s+list_alloc_node\s+<\.T>\s+<\(\.T,i32\)\*>Result<i32,\s*Diag>>/, 'list_alloc_node must keep the checked node allocation boundary centralized');
assert.match(code, /fn\s+cons\s+<\.T>[\s\S]*match\s+list_alloc_node<\.T>\s+head\s+tail_ptr/, 'cons must share the checked node allocator');
assert.match(code, /fn\s+list_map_impl\s+<\.T,\.U>[\s\S]*match\s+list_alloc_node<\.U>\s+mapped_head\s+mapped_tail_ptr[\s\S]*Result::Err\s+e:[\s\S]*free<\.U>\s+mapped_tail[\s\S]*err<List<\.U>,\s*Diag>\s+e/, 'list_map_impl must free the partial mapped tail if final node allocation fails');
assert.match(code, /fn\s+list_filter_impl\s+<\.T>[\s\S]*match\s+list_alloc_node<\.T>\s+load<\.T>\s+lst_ptr\s+filtered_tail_ptr[\s\S]*Result::Err\s+e:[\s\S]*free<\.T>\s+filtered_tail[\s\S]*err<List<\.T>,\s*Diag>\s+e/, 'list_filter_impl must free the partial filtered tail if final node allocation fails');

console.log('list unsafe unwrap regression passed');
