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

assert.match(code, /#import\s+"std\/env\/cliarg\/raw"\s+as\s+cli_raw\b/, 'cliarg root must import the raw argv boundary only through a qualified namespace');
assert.doesNotMatch(code, /#import\s+"std\/env\/cliarg\/cstr"/, 'cliarg root facade must not import or re-export the C string conversion boundary directly');
assert.match(rawCode, /#import\s+"std\/env\/cliarg\/cstr"\s+as\s+cstr\b/, 'cliarg raw boundary must explicitly own the C string conversion dependency');
assert.doesNotMatch(code, /#import\s+"core\/mem\/(?:raw|internal)"/, 'cliarg root facade must not import raw memory or raw address conversion modules directly');
for (const pattern of [
    /\bmem_ptr_addr\b/,
    /\bmem_ptr_wrap\b/,
    /\bstore_i32\b/,
    /\bload_i32\b/,
    /\bargs_get\b/,
    /\bcli_zero_i32_slots_result\b/,
    /\bcli_zero_u8_buffer_result\b/,
]) {
    assert.doesNotMatch(code, pattern, 'cliarg root facade must not perform argv raw scratch operations directly');
}
for (const pattern of [
    /\bfn\s+cli_load_u8_result\b/,
    /\bfn\s+cli_args_sizes_result\b/,
    /\bfn\s+cliarg_count_result\b/,
    /\bfn\s+cliarg_get_checked\b/,
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
assert.match(rawCode, /\b(?:store_i32|load_i32|store_u8|load_u8|mem_ptr_addr)\b/, 'cliarg raw argv implementation must carry source-level raw memory evidence');
assert.match(cstrCode, /\bload_u8\b/, 'cliarg cstr conversion must carry source-level checked byte-load evidence');
assert.match(rawCode, /fn\s+cli_args_sizes_result\s+<\(MemPtr<u8>\)\*>Result<CliArgSizes,i32>>\s+\(meta\):[\s\S]*store_i32\s+meta_raw\s+0[\s\S]*store_i32\s+add\s+meta_raw\s+4\s+0[\s\S]*args_sizes_get\s+meta_raw\s+add\s+meta_raw\s+4[\s\S]*load_i32\s+meta_raw[\s\S]*load_i32\s+add\s+meta_raw\s+4/, 'cliarg sizes must initialize and read WASI out pointers in one raw-address boundary');
assert.match(rawCode, /fn\s+cliarg_get_checked\s+<\(i32\)\*>Option<str>>\s+\(idx\):[\s\S]*cli_args_sizes_result\s+meta[\s\S]*cli_zero_i32_slots_result\s+argv\s+argv_size[\s\S]*cli_zero_u8_buffer_result\s+argv_buf\s+buf_size[\s\S]*store_i32\s+arg_slot_raw\s+0[\s\S]*args_get\s+argv_raw\s+argv_buf_raw[\s\S]*load_i32\s+arg_slot_raw/, 'cliarg_get_checked must initialize argv scratch and read arg slots in one raw-address boundary');
assert.match(rawCode, /fn\s+cliarg_get_checked\s+<\(i32\)\*>Option<str>>\s+\(idx\):[\s\S]*\bor\s+lt\s+idx\s+0\s+or\s+ge\s+idx\s+argc\s+le\s+buf_size\s+0[\s\S]*let\s+arg_slot_raw\s+<i32>\s+add\s+argv_raw\s+mul\s+idx\s+4/, 'cliarg_get_checked must reject negative indexes before computing the argv slot address');
assert.match(rawCode, /fn\s+cliarg_count_result\b[\s\S]*\balloc_region<u8>\s+8[\s\S]*\blet\s+meta\s+<MemPtr<u8>>\s+region_ptr\s+&meta_region[\s\S]*\bdealloc_region<u8>\s+meta_region/, 'cliarg_count_result must own argc metadata scratch through RegionToken');
assert.match(rawCode, /fn\s+cliarg_get_checked\b[\s\S]*\balloc_region<u8>\s+8[\s\S]*\blet\s+meta\s+<MemPtr<u8>>\s+region_ptr\s+&meta_region[\s\S]*\balloc_region<u8>\s+argv_size[\s\S]*\blet\s+argv\s+<MemPtr<u8>>\s+region_ptr\s+&argv_region[\s\S]*\balloc_region<u8>\s+buf_size[\s\S]*\blet\s+argv_buf\s+<MemPtr<u8>>\s+region_ptr\s+&argv_buf_region[\s\S]*\bdealloc_region<u8>\s+argv_buf_region[\s\S]*\bdealloc_region<u8>\s+argv_region[\s\S]*\bdealloc_region<u8>\s+meta_region/, 'cliarg_get_checked must own argv metadata and byte buffers through RegionToken');
assert.match(rawCode, /fn\s+__cli_copy_to_cstr\s+<\(str\)\*>Result<RegionToken<u8>,i32>>[\s\S]*\balloc_region<u8>\s+size[\s\S]*\blet\s+dst\s+<MemPtr<u8>>\s+region_ptr\s+&dst_region[\s\S]*Result<RegionToken<u8>,i32>::Ok\s+dst_region[\s\S]*\blet\s+cpath\s+<MemPtr<u8>>\s+region_ptr\s+&cpath_region[\s\S]*\bdealloc_region<u8>\s+cpath_region/, 'LLVM cliarg cmdline C string scratch must be owned through RegionToken');
assert.match(rawCode, /fn\s+args_sizes_get\b[\s\S]*\balloc_region<u8>\s+cap[\s\S]*\blet\s+tmp\s+<MemPtr<u8>>\s+region_ptr\s+&tmp_region[\s\S]*\bdealloc_region<u8>\s+tmp_region/, 'LLVM args_sizes_get temporary cmdline buffer must be owned through RegionToken');
assert.match(rawCode, /fn\s+args_get\b[\s\S]*\balloc_region<u8>\s+cap[\s\S]*\blet\s+tmp\s+<MemPtr<u8>>\s+region_ptr\s+&tmp_region[\s\S]*\bdealloc_region<u8>\s+tmp_region/, 'LLVM args_get temporary cmdline buffer must be owned through RegionToken');
assert.doesNotMatch(rawCode, /#import\s+"core\/mem\/pointer\/alloc"\s+as\s+\*/, 'cliarg raw boundary must not import low-level MemPtr owner allocation wrappers');
assert.doesNotMatch(rawCode, /\b(?:alloc_ptr|realloc_ptr|dealloc_ptr)\b/, 'cliarg raw boundary must not use MemPtr as an argv scratch free-obligation owner');
assert.match(code, /fn\s+cliarg_count\s+<\(\)\*>i32>\s+\(\):[\s\S]*cli_raw::cliarg_count_result/, 'cliarg_count must delegate to the raw argv boundary helper');
assert.match(code, /fn\s+cliarg_get\s+<\(i32\)\*>Option<str>>\s+\(idx\):[\s\S]*cli_raw::cliarg_get_checked\s+idx/, 'cliarg_get must delegate argv scratch handling to the raw boundary helper');

console.log('stdlib cliarg unsafe unwrap regression passed');
