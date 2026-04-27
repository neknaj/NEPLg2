#!/usr/bin/env node

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');

const repoRoot = path.resolve(__dirname, '..');
const relPath = 'stdlib/std/env/cliarg.nepl';
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

assert.match(code, /fn\s+cli_load_u8_result\s+<\(MemPtr<u8>,i32\)->Result<i32,i32>>\s+\(base,\s*off\):[\s\S]*Option::None:[\s\S]*Result<i32,i32>::Err\s+1/, 'cliarg must map invalid byte loads to shim errno 1');
assert.match(code, /fn\s+cstr_len_result\s+<\(MemPtr<u8>\)\*>Result<i32,str>>\s+\(p\):[\s\S]*Option::None:[\s\S]*set\s+ok\s+0[\s\S]*Result<i32,str>::Err\s+"cliarg\.cstr_len invalid pointer"/, 'cstr_len_result must return Err on invalid C string pointer');
assert.match(code, /fn\s+cliarg_count\s+<\(\)\*>i32>\s+\(\):[\s\S]*match\s+load_i32\s+argc_ptr:[\s\S]*Option::None:[\s\S]*0/, 'cliarg_count must avoid unsafe metadata unwrap');
assert.match(code, /fn\s+cliarg_get\s+<\(i32\)\*>Option<str>>\s+\(idx\):[\s\S]*match\s+load_i32\s+argc_ptr:[\s\S]*Option::None:[\s\S]*none<str>[\s\S]*match\s+load_i32\s+buf_ptr:[\s\S]*Option::None:[\s\S]*none<str>/, 'cliarg_get must avoid unsafe metadata unwraps');

console.log('stdlib cliarg unsafe unwrap regression passed');
