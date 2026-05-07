#!/usr/bin/env node

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');

const repoRoot = path.resolve(__dirname, '..');
const relPath = 'stdlib/std/env/cliarg.nepl';
const rawRelPath = 'stdlib/std/env/cliarg/raw.nepl';
const cstrRelPath = 'stdlib/std/env/cliarg/cstr.nepl';
const src = fs.readFileSync(path.join(repoRoot, relPath), 'utf8');
const rawSrc = fs.readFileSync(path.join(repoRoot, rawRelPath), 'utf8');
const cstrSrc = fs.readFileSync(path.join(repoRoot, cstrRelPath), 'utf8');
const loaderSrc = fs.readFileSync(path.join(repoRoot, 'nepl-core/src/loader.rs'), 'utf8');

const code = src
    .split(/\r?\n/)
    .filter((line) => !/^\s*\/\//.test(line))
    .join('\n');
const rawCode = rawSrc
    .split(/\r?\n/)
    .filter((line) => !/^\s*\/\//.test(line))
    .join('\n');
const cstrCode = cstrSrc
    .split(/\r?\n/)
    .filter((line) => !/^\s*\/\//.test(line))
    .join('\n');
const allCode = `${code}\n${rawCode}\n${cstrCode}`;

const forbidden = [
    /\bunwrap\b/,
    /\bunwrap_ok\b/,
    /\bunwrap_err\b/,
    /\buwok\b/,
    /\buwerr\b/,
    /#intrinsic\s+"unreachable"/,
];

for (const pattern of forbidden) {
    assert.doesNotMatch(allCode, pattern, `${relPath} must not use unsafe unwrap helpers in implementation code`);
}

assert.match(code, /#import\s+"std\/env\/cliarg\/raw"\s+as\s+\*/, 'cliarg root must import the raw argv boundary');
assert.match(code, /#import\s+"std\/env\/cliarg\/cstr"\s+as\s+\*/, 'cliarg root must import the C string conversion boundary');
for (const pattern of [
    /\bfn\s+cli_load_u8_result\b/,
    /\bfn\s+cli_args_sizes_result\b/,
    /\bfn\s+args_get\b/,
]) {
    assert.doesNotMatch(code, pattern, 'cliarg root must not keep raw argv boundary implementation bodies');
    assert.match(rawCode, pattern, 'cliarg raw module must own raw argv boundary implementation bodies');
}
for (const pattern of [
    /\bfn\s+cstr_len_result\b/,
    /\bfn\s+cstr_to_str\b/,
]) {
    assert.doesNotMatch(code, pattern, 'cliarg root must not keep C string implementation bodies');
    assert.match(cstrCode, pattern, 'cliarg cstr module must own C string implementation bodies');
}

assert.match(rawCode, /fn\s+cli_load_u8_result\s+<\(MemPtr<u8>,i32\)->Result<i32,i32>>\s+\(base,\s*off\):[\s\S]*Option::None:[\s\S]*Result<i32,i32>::Err\s+1/, 'cliarg must map invalid byte loads to shim errno 1');
assert.match(cstrCode, /fn\s+cstr_len_result\s+<\(MemPtr<u8>\)\*>Result<i32,str>>\s+\(p\):[\s\S]*Option::None:[\s\S]*set\s+ok\s+0[\s\S]*Result<i32,str>::Err\s+"cliarg\.cstr_len invalid pointer"/, 'cstr_len_result must return Err on invalid C string pointer');
assert.match(cstrCode, /fn\s+cstr_to_str\s+<\(MemPtr<u8>\)\*>str>\s+\(p\):[\s\S]*string_from_mem_unchecked_result\s+p\s+len/, 'cstr_to_str must delegate string allocation and owner transfer to alloc/string');
assert.doesNotMatch(allCode, /\bfn\s+cli_i32_ptr\b/, 'cliarg must not reintroduce MemPtr<i32> out-pointer projections across WASI boundaries');
assert.match(loaderSrc, /&\["std",\s*"env",\s*"cliarg",\s*"raw\.nepl"\]/, 'cliarg raw argv implementation must be an exact raw-memory boundary');
assert.doesNotMatch(loaderSrc, /&\["std",\s*"env",\s*"cliarg\.nepl"\]/, 'cliarg root facade must not receive raw-memory boundary capability');
assert.doesNotMatch(loaderSrc, /&\["std",\s*"env",\s*"cliarg",\s*"cstr\.nepl"\]/, 'cliarg cstr conversion module must remain raw-memory-free');
assert.match(rawCode, /fn\s+cli_args_sizes_result\s+<\(MemPtr<u8>\)\*>Result<CliArgSizes,i32>>\s+\(meta\):[\s\S]*store_i32\s+meta_raw\s+0[\s\S]*store_i32\s+add\s+meta_raw\s+4\s+0[\s\S]*args_sizes_get\s+meta_raw\s+add\s+meta_raw\s+4[\s\S]*load_i32\s+meta_raw[\s\S]*load_i32\s+add\s+meta_raw\s+4/, 'cliarg sizes must initialize and read WASI out pointers in one raw-address boundary');
assert.match(code, /fn\s+cliarg_count\s+<\(\)\*>i32>\s+\(\):[\s\S]*cli_args_sizes_result\s+meta[\s\S]*get\s+sizes\s+"argc"/, 'cliarg_count must use the raw-address args_sizes boundary');
assert.match(code, /fn\s+cliarg_get\s+<\(i32\)\*>Option<str>>\s+\(idx\):[\s\S]*cli_args_sizes_result\s+meta[\s\S]*cli_zero_i32_slots_result\s+argv\s+argv_size[\s\S]*cli_zero_u8_buffer_result\s+argv_buf\s+buf_size[\s\S]*store_i32\s+arg_slot_raw\s+0[\s\S]*args_get\s+argv_raw\s+argv_buf_raw[\s\S]*load_i32\s+arg_slot_raw/, 'cliarg_get must initialize argv scratch and read arg slots in one raw-address boundary');

console.log('stdlib cliarg unsafe unwrap regression passed');
