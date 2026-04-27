#!/usr/bin/env node

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');

const repoRoot = path.resolve(__dirname, '..');
const relPaths = [
    'stdlib/alloc/collections/vec.nepl',
    'stdlib/alloc/collections/vec/sort.nepl',
];

const codeByPath = new Map();

for (const relPath of relPaths) {
    const src = fs.readFileSync(path.join(repoRoot, relPath), 'utf8');
    const code = src
        .split(/\r?\n/)
        .filter((line) => !/^\s*\/\//.test(line))
        .join('\n');
    codeByPath.set(relPath, code);
}

const forbidden = [
    /\bunwrap\b/,
    /\bunwrap_ok\b/,
    /\bunwrap_err\b/,
    /\buwok\b/,
    /\buwerr\b/,
    /#intrinsic\s+"unreachable"/,
    /dealloc_ptr/,
];

for (const [relPath, code] of codeByPath) {
    for (const pattern of forbidden) {
        assert.doesNotMatch(code, pattern, `${relPath} must not use unsafe unwrap or checked deallocation helpers in implementation code`);
    }
}

const vecCode = codeByPath.get('stdlib/alloc/collections/vec.nepl');
const sortCode = codeByPath.get('stdlib/alloc/collections/vec/sort.nepl');

assert.match(vecCode, /fn\s+free\s+<\.T>\s+<\(Vec<\.T>\)->\(\)>\s+\(v\):[\s\S]*dealloc_raw\s+mem_ptr_addr\s+v_data\s+mul\s+v_cap\s+size_of<\.T>/, 'Vec.free must use raw owner cleanup for data storage');
assert.match(vecCode, /fn\s+push\s+<\.T>\s+<\(Vec<\.T>,\.T\)->Result<Vec<\.T>,\s*StdErrorKind>>\s+\(v,\s*item\):[\s\S]*match\s+realloc_ptr<\.T>\s+v_data\s+old_bytes\s+new_bytes:[\s\S]*Result::Err\s+_e:[\s\S]*dealloc_raw\s+mem_ptr_addr\s+v_data\s+old_bytes[\s\S]*Result::Err<Vec<\.T>,\s*StdErrorKind>\s+StdErrorKind::OutOfMemory/, 'Vec.push must release the consumed old buffer when grow fails');
assert.match(vecCode, /dealloc_raw\s+mem_ptr_addr\s+left0_data\s+mul\s+left0_cap\s+size_of<\.T>/, 'Vec.partition cleanup must use raw owner cleanup for the left buffer after right allocation failure');
assert.match(sortCode, /fn\s+sort_merge\s+<\.T:\s+Ord>[\s\S]*dealloc_raw\s+mem_ptr_addr\s+buf\s+mul\s+n\s+size_of<\.T>[\s\S]*Result<\(\),\s*StdErrorKind>::Ok\s+\(\)/, 'sort_merge must release scratch buffer with raw owner cleanup');
assert.match(sortCode, /fn\s+sort_merge_ret\s+<\.T:\s+Ord>[\s\S]*dealloc_raw\s+mem_ptr_addr\s+buf\s+mul\s+n\s+size_of<\.T>[\s\S]*Result<Vec<\.T>,\s*StdErrorKind>::Ok\s+Vec<\.T>\s+n\s+cap\s+data_ptr/, 'sort_merge_ret must release scratch buffer with raw owner cleanup');

console.log('vec unsafe unwrap regression passed');
