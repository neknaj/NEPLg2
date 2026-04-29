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

function between(code, start, end) {
    const startIdx = code.indexOf(start);
    assert.notEqual(startIdx, -1, `missing section start: ${start}`);
    const endIdx = code.indexOf(end, startIdx + start.length);
    assert.notEqual(endIdx, -1, `missing section end: ${end}`);
    return code.slice(startIdx, endIdx);
}

const newSection = between(code, 'fn new ', 'fn push ');
const pushSection = between(code, 'fn push ', 'fn push_ref ');

assert.match(code, /struct\s+Stack<\.T>:\s+hdr\s+<MemPtr<u8>>\s+data\s+<MemPtr<\.T>>/, 'Stack must carry both header and data owners as fields');
assert.doesNotMatch(newSection, /\blet\s+(?:len_ptr|cap_ptr|data_ptr_ptr)\b/, 'Stack.new must initialize raw header fields directly without helper MemPtr owner locals');
assert.match(newSection, /let\s+header_addr\s+<i32>\s+mem_ptr_addr\s+header[\s\S]*let\s+data_addr\s+<i32>\s+\*field::get_ref\s+&data\s+"raw"[\s\S]*store_i32\s+header_addr\s+0[\s\S]*store_i32\s+add\s+header_addr\s+4\s+cap[\s\S]*store_i32\s+add\s+header_addr\s+8\s+data_addr[\s\S]*Result::Ok<Stack<\.T>,\s*Diag>\s+Stack<\.T>\s+header\s+data/, 'Stack.new must initialize header cells directly and return both header and data owners exactly once');
assert.doesNotMatch(pushSection, /\blet\s+(?:len_ptr|cap_ptr|data_ptr_ptr)\b/, 'Stack.push must update raw header fields directly without helper MemPtr owner locals');
assert.match(pushSection, /let\s+hdr\s+<MemPtr<u8>>\s+field::get\s+stk\s+"hdr"[\s\S]*let\s+data\s+<MemPtr<\.T>>\s+field::get\s+stk\s+"data"[\s\S]*let\s+hdr_addr\s+<i32>\s+mem_ptr_addr\s+hdr[\s\S]*let\s+data_addr\s+<i32>\s+\*field::get_ref\s+&data\s+"raw"[\s\S]*let\s+len\s+<i32>\s+load_i32\s+hdr_addr[\s\S]*let\s+cap\s+<i32>\s+load_i32\s+add\s+hdr_addr\s+4/, 'Stack.push must derive header metadata from the consumed header owner and data address from the carried data owner');
assert.match(pushSection, /Result::Err\s+_e:[\s\S]*dealloc_raw\s+data_addr\s+mul\s+cap\s+size_of<\.T>[\s\S]*dealloc_raw\s+hdr_addr\s+12[\s\S]*diag_err<Stack<\.T>>\s+diag_out_of_memory\s+"push\(realloc_raw\)"/, 'Stack.push realloc failure must release consumed storage before returning Err');
assert.match(pushSection, /Result::Ok\s+new_data:[\s\S]*let\s+new_data_addr\s+<i32>\s+\*field::get_ref\s+&new_data\s+"raw"[\s\S]*Result::Ok<Stack<\.T>,\s*Diag>\s+Stack<\.T>\s+hdr\s+new_data/, 'Stack.push must transfer the grown data owner into the returned Stack');
assert.match(pushSection, /else:[\s\S]*store<\.T>\s+add\s+data_addr\s+off\s+item[\s\S]*Result::Ok<Stack<\.T>,\s*Diag>\s+Stack<\.T>\s+hdr\s+data/, 'Stack.push non-grow path must transfer the original data owner into the returned Stack');
assert.match(code, /fn\s+free\s+<\.T>\s+<\(Stack<\.T>\)\*>\(\)>\s+\(stk\):[\s\S]*dealloc_raw\s+mem_ptr_addr\s+data[\s\S]*dealloc_raw\s+mem_ptr_addr\s+hdr\s+12/, 'Stack.free must use raw owner cleanup for data and header storage');
assert.doesNotMatch(code, /dealloc_ptr/, 'Stack must not use checked deallocation for owned internals');

console.log('stack unsafe unwrap regression passed');
