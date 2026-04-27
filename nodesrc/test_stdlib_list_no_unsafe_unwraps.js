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

assert.match(reverseCode, /let\s+mut\s+new_head\s+<i32>\s+0/, 'reverse must track the partial result by raw head pointer until ownership is returned');
assert.match(reverseCode, /match\s+list_alloc_node<\.T>\s+value\s+new_head\s+"list_reverse\(node\)"/, 'reverse must allocate nodes through the checked node allocator');
assert.match(reverseCode, /Result::Err\s+e:[\s\S]*free<\.T>\s+List\s+new_head[\s\S]*set\s+result\s+err<List<\.T>,\s*Diag>\s+e/, 'reverse must free the partial reversed list before returning allocation failure');
assert.match(reverseCode, /if\s+done\s+result\s+ok<List<\.T>,\s*Diag>\s+List\s+new_head/, 'reverse must return the accumulated Result without unsafe unwraps');

assert.match(code, /fn\s+cons\s+<\.T>[\s\S]*match\s+list_alloc_node<\.T>\s+head\s+tail_ptr\s+"list_cons\(node\)"/, 'cons must share the checked node allocator');
assert.match(code, /fn\s+list_map_impl\s+<\.T,\.U>[\s\S]*match\s+list_alloc_node<\.U>\s+mapped_head\s+mapped_tail_ptr\s+"list_map\(node\)"[\s\S]*Result::Err\s+e:[\s\S]*free<\.U>\s+mapped_tail[\s\S]*err<List<\.U>,\s*Diag>\s+e/, 'list_map_impl must free the partial mapped tail if final node allocation fails');
assert.match(code, /fn\s+list_filter_impl\s+<\.T>[\s\S]*match\s+list_alloc_node<\.T>\s+load<\.T>\s+lst_ptr\s+filtered_tail_ptr\s+"list_filter\(node\)"[\s\S]*Result::Err\s+e:[\s\S]*free<\.T>\s+filtered_tail[\s\S]*err<List<\.T>,\s*Diag>\s+e/, 'list_filter_impl must free the partial filtered tail if final node allocation fails');

console.log('list unsafe unwrap regression passed');
