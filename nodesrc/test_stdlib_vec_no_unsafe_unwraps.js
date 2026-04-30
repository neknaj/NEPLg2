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

function between(code, start, end) {
    const startIdx = code.indexOf(start);
    assert.notEqual(startIdx, -1, `missing section start: ${start}`);
    const endIdx = code.indexOf(end, startIdx + start.length);
    assert.notEqual(endIdx, -1, `missing section end: ${end}`);
    return code.slice(startIdx, endIdx);
}

const pushSection = between(vecCode, 'fn push ', 'fn get ');
const withCapacitySection = between(vecCode, 'fn with_capacity ', 'fn filled ');
const popSection = between(vecCode, 'fn pop ', 'fn clear ');
const clearSection = between(vecCode, 'fn clear ', 'fn vec_read_at ');
const mapSection = between(vecCode, 'fn map ', 'fn filter ');
const freeSection = vecCode.slice(vecCode.indexOf('fn free '));

assert.doesNotMatch(vecCode, /\bfield::get\s+\w+\s+"(?:len|cap)"/, 'Vec implementation must read Copy len/cap header fields through field::get_ref so owner-consuming helpers do not move them');
assert.match(withCapacitySection, /if:\s+lt\s+cap\s+0\s+then:\s+Result::Err<Vec<\.T>,\s*StdErrorKind>\s+StdErrorKind::InvalidOperation[\s\S]*eq\s+cap\s+0[\s\S]*alloc_ptr<\.T>\s+mul\s+cap\s+size_of<\.T>/, 'Vec.with_capacity must reject negative capacity before calling the allocator');
assert.match(pushSection, /let\s+v_data\s+<MemPtr<\.T>>\s+field::get\s+v\s+"data"/, 'Vec.push must explicitly move the data owner from the consumed input Vec');
assert.match(popSection, /let\s+v_data\s+<MemPtr<\.T>>\s+field::get\s+v\s+"data"/, 'Vec.pop must explicitly move the data owner into the returned Vec');
assert.match(clearSection, /let\s+v_data\s+<MemPtr<\.T>>\s+field::get\s+v\s+"data"/, 'Vec.clear must explicitly move the data owner into the returned Vec');
assert.match(freeSection, /let\s+v_data\s+<MemPtr<\.T>>\s+field::get\s+v\s+"data"[\s\S]*dealloc_raw\s+mem_ptr_addr\s+v_data\s+mul\s+v_cap\s+size_of<\.T>/, 'Vec.free must explicitly move the data owner before deallocating it');
assert.match(mapSection, /let\s+out_data\s+<MemPtr<\.U>>\s+field::get\s+out0\s+"data"/, 'Vec.map must explicitly move the output data owner from the allocated Vec into the returned Vec');
assert.match(vecCode, /fn\s+free\s+<\.T>\s+<\(Vec<\.T>\)->\(\)>\s+\(v\):[\s\S]*dealloc_raw\s+mem_ptr_addr\s+v_data\s+mul\s+v_cap\s+size_of<\.T>/, 'Vec.free must use raw owner cleanup for data storage');
assert.match(vecCode, /fn\s+push\s+<\.T>\s+<\(Vec<\.T>,\.T\)->Result<Vec<\.T>,\s*StdErrorKind>>\s+\(v,\s*item\):[\s\S]*match\s+realloc_ptr<\.T>\s+v_data\s+old_bytes\s+new_bytes:[\s\S]*Result::Err\s+_e:[\s\S]*dealloc_raw\s+mem_ptr_addr\s+v_data\s+old_bytes[\s\S]*Result::Err<Vec<\.T>,\s*StdErrorKind>\s+StdErrorKind::OutOfMemory/, 'Vec.push must release the consumed old buffer when grow fails');
assert.match(vecCode, /dealloc_raw\s+mem_ptr_addr\s+left0_data\s+mul\s+left0_cap\s+size_of<\.T>/, 'Vec.partition cleanup must use raw owner cleanup for the left buffer after right allocation failure');
assert.match(sortCode, /fn\s+sort_merge\s+<\.T:\s+Ord>[\s\S]*dealloc_raw\s+mem_ptr_addr\s+buf\s+mul\s+n\s+size_of<\.T>[\s\S]*Result<\(\),\s*StdErrorKind>::Ok\s+\(\)/, 'sort_merge must release scratch buffer with raw owner cleanup');
assert.match(sortCode, /fn\s+sort_merge_ret\s+<\.T:\s+Ord>[\s\S]*dealloc_raw\s+mem_ptr_addr\s+buf\s+mul\s+n\s+size_of<\.T>[\s\S]*Result<Vec<\.T>,\s*StdErrorKind>::Ok\s+Vec<\.T>\s+n\s+cap\s+data_ptr/, 'sort_merge_ret must release scratch buffer with raw owner cleanup');

console.log('vec unsafe unwrap regression passed');
