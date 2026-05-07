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
];

for (const pattern of forbidden) {
    assert.doesNotMatch(code, pattern, `${relPath} must not use unsafe unwrap helpers in implementation code`);
}

assert.match(
    code,
    /struct\s+List<\.T>:[\s\S]*items\s+<Vec<\.T>>/,
    'List must keep typed Vec<T> storage instead of a raw node pointer',
);
assert.match(
    code,
    /#import\s+"alloc\/collections\/vec"\s+as\s+vec/,
    'List must delegate storage ownership to the Vec abstraction',
);
assert.match(
    code,
    /fn\s+cons\s+<\.T:\s*Copy>\s+<\(\.T,List<\.T>\)\*>Result<List<\.T>,\s*Diag>>[\s\S]*vec::push<\.T>\s+items\s+head/,
    'List.cons must add the logical head through Vec.push and expose the Copy payload contract',
);
assert.match(
    code,
    /fn\s+tail\s+<\.T:\s*Copy>\s+<\(List<\.T>\)->Option<List<\.T>>>[\s\S]*vec::pop<\.T>\s+items[\s\S]*some<List<\.T>>\s+List<\.T>\s+next_items/,
    'List.tail must remove the logical head by returning the Vec.pop owner',
);
assert.match(
    code,
    /fn\s+get\s+<\.T:\s*Copy>\s+<\(List<\.T>,i32\)->Option<\.T>>[\s\S]*list_physical_index\s+n\s+idx[\s\S]*vec::free<\.T>\s+items/,
    'List.get must translate logical index to storage index and close the consumed owner',
);
assert.match(
    code,
    /fn\s+reverse\s+<\.T:\s*Copy>\s+<\(List<\.T>\)\*>List<\.T>>[\s\S]*list_reverse_items<\.T>\s+&items\s+0\s+sub\s+n\s+1[\s\S]*List<\.T>\s+items/,
    'List.reverse must reverse storage in place without raw node relinking',
);
assert.match(
    code,
    /fn\s+free\s+<\.T>\s+<\(List<\.T>\)->\(\)>[\s\S]*list_free_items<\.T>\s+lst/,
    'List.free must close the Vec<T> owner',
);
assert.match(
    code,
    /fn\s+map\s+<\.T:\s*Copy,\.U:\s*Copy>[\s\S]*vec::with_capacity<\.U>\s+n[\s\S]*vec::push<\.U>\s+out\s+mapped[\s\S]*vec::free<\.T>\s+items/,
    'List.map must build a typed Vec<U> output and close the input storage owner',
);
assert.match(
    code,
    /fn\s+filter\s+<\.T:\s*Copy>[\s\S]*vec::with_capacity<\.T>\s+n[\s\S]*vec::push<\.T>\s+out\s+value[\s\S]*vec::free<\.T>\s+items/,
    'List.filter must build a typed Vec<T> output and close the input storage owner',
);
assert.doesNotMatch(
    code,
    /\bMemPtr\b|\balloc_ptr\b|\balloc_raw\b|\brealloc_ptr\b|\bdealloc_raw\b|\bload_i32\b|\bstore_i32\b|\bmem_ptr_addr\b|dealloc_ptr|\bptr\s+<i32>/,
    'List must not reintroduce raw node storage, raw headers, or raw pointer sentinels',
);

console.log('list unsafe unwrap regression passed');
