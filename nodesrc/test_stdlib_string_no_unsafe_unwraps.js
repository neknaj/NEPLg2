#!/usr/bin/env node

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');

const repoRoot = path.resolve(__dirname, '..');
const relPath = 'stdlib/alloc/string.nepl';
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

assert.match(code, /enum\s+StringUtf8LeadKind:/, 'alloc/string must classify UTF-8 leading bytes with an enum');
assert.match(code, /fn\s+string_from_utf8_mem_result\s+/, 'alloc/string must expose a checked UTF-8 construction API');
assert.match(code, /fn\s+str_utf8_is_boundary\s+/, 'alloc/string must validate UTF-8 slice boundaries');
assert.match(code, /fn\s+concat_result\s+/, 'alloc/string must keep allocation-bearing concat available as Result');
assert.match(code, /fn\s+str_slice_result\s+/, 'alloc/string must keep allocation-bearing slice available as Result');
assert.match(code, /fn\s+sb_build_result\s+/, 'StringBuilder build must have a Result-returning path');
assert.match(code, /fn\s+from_f64_result\s+/, 'from_f64 must have a Result-returning implementation path');
assert.match(code, /fn\s+string_finish_base[\s\S]*store_i32\s+mem_ptr_addr\s+base\s+byte_len/, 'string_finish_base must use owned raw header store');
assert.match(code, /fn\s+from_u128_radix[\s\S]*alloc_ptr<u8>\s+132[\s\S]*dealloc_raw\s+scratch_raw\s+132/, 'from_u128_radix must manage scratch storage without unsafe unwraps');
assert.match(code, /fn\s+str_slice_result[\s\S]*str_utf8_is_boundary\s+s\s+s0[\s\S]*str_utf8_is_boundary\s+s\s+e0/, 'str_slice_result must reject non-boundary UTF-8 byte ranges');

console.log('alloc/string unsafe unwrap regression passed');
