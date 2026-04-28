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
const fromU128Radix = code.match(/fn\s+from_u128_radix[\s\S]*?(?=\nfn\s+to_u128|\nfn\s+parse_u128|\n\/\/ to_u128|$)/)?.[0] ?? '';

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
assert.doesNotMatch(code, /struct\s+StringBuilder:[\s\S]*parts\s+<Vec<str>>/, 'StringBuilder must not store non-Copy str payloads in Vec raw storage');
assert.match(code, /struct\s+StringBuilder:[\s\S]*data\s+<MemPtr<u8>>[\s\S]*len\s+<i32>[\s\S]*cap\s+<i32>/, 'StringBuilder must use owned byte storage');
assert.match(code, /fn\s+from_f64_result\s+/, 'from_f64 must have a Result-returning implementation path');
assert.match(code, /fn\s+string_finish_base[\s\S]*store_i32\s+mem_ptr_addr\s+base\s+byte_len/, 'string_finish_base must use owned raw header store');
assert.match(fromU128Radix, /digit_count[\s\S]*string_alloc_region\s+digit_count[\s\S]*set\s+pos\s+sub\s+pos\s+1/, 'from_u128_radix must count digits before allocating and write output from the end');
assert.doesNotMatch(fromU128Radix, /scratch_raw/, 'from_u128_radix must not use scratch raw storage for digit reversal');
assert.match(code, /fn\s+str_slice_result[\s\S]*str_utf8_is_boundary\s+s\s+s0[\s\S]*str_utf8_is_boundary\s+s\s+e0/, 'str_slice_result must reject non-boundary UTF-8 byte ranges');
assert.doesNotMatch(code, /\bget\s+(?:region|out_region)\s+"ptr"/, 'alloc/string must read RegionToken ptr through get_ref so string construction does not move the token');
assert.doesNotMatch(code, /\bstring_region_data_ptr\s+(?:region|out_region)\b/, 'alloc/string must pass RegionToken projections by reference');
assert.match(code, /fn\s+string_region_data_ptr\s+<\(&RegionToken<u8>\)->MemPtr<u8>>/, 'string_region_data_ptr must project from a RegionToken reference');

console.log('alloc/string unsafe unwrap regression passed');
