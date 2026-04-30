#!/usr/bin/env node

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');

const repoRoot = path.resolve(__dirname, '..');
const relPath = 'stdlib/std/fs.nepl';
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

assert.doesNotMatch(code, /\bstr_split_result\b/, 'fs_normalize_relative must not use owned Vec<str> split');
assert.doesNotMatch(code, /\bstr_split_ranges_result\b/, 'fs_normalize_relative must not allocate split range vectors');
assert.match(code, /fn\s+fs_normalize_relative\s+<\(str\)->Result<str,i32>>\s+\(path\):[\s\S]*str_split_next\s+path\s+"\/"\s+cursor[\s\S]*match\s+get\s+step\s+"kind":[\s\S]*StrSplitStepKind::Part:/, 'fs_normalize_relative must scan path components with allocation-free split steps');
assert.match(code, /fn\s+fs_normalize_relative\s+<\(str\)->Result<str,i32>>\s+\(path\):[\s\S]*match\s+fs_normalize_range_push\s+stack\s+part_start\s+part_end:[\s\S]*Result::Err\s+e:[\s\S]*set\s+stack\s+v::Vec<i32>\s+0\s+0\s+mem_ptr_wrap\s+0[\s\S]*set\s+err\s+e/, 'fs_normalize_relative must store component ranges as Copy i32 pairs and map push failure to errno');
assert.match(code, /fn\s+fs_read_dir_fd\s+<\(i32\)\*>Result<Vec<str>,i32>>\s+\(fd\):[\s\S]*match\s+v::push<str>\s+entries\s+name:[\s\S]*Result::Err\s+_e:[\s\S]*set\s+entries\s+v::Vec<str>\s+0\s+0\s+mem_ptr_wrap\s+0[\s\S]*set\s+err\s+12/, 'fs_read_dir_fd must map entry accumulation push failure to errno 12');

console.log('stdlib fs unsafe unwrap regression passed');
