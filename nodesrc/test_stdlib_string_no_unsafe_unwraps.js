#!/usr/bin/env node

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const {
    stripNeplComments,
    assertStringBuilderOwnerBoundary,
} = require('./source_policy/stdlib_builder_owner');

const repoRoot = path.resolve(__dirname, '..');
const relPath = 'stdlib/alloc/string.nepl';
const src = fs.readFileSync(path.join(repoRoot, relPath), 'utf8');
const utf8RelPath = 'stdlib/alloc/string/utf8.nepl';
const utf8Src = fs.readFileSync(path.join(repoRoot, utf8RelPath), 'utf8');

const code = stripNeplComments(src);
const utf8Code = stripNeplComments(utf8Src);
const fromU128Radix = code.match(/fn\s+from_u128_radix[\s\S]*?(?=\nfn\s+to_u128|\nfn\s+parse_u128|\n\/\/ to_u128|$)/)?.[0] ?? '';
const stringFinish = code.match(/fn\s+string_finish\s+<\(RegionToken<u8>,i32\)->str>\s+\(region,\s*byte_len\):[\s\S]*?(?=\nfn\s+string_from_addr_unchecked\s+)/)?.[0] ?? '';
const codeWithoutStringFinish = stringFinish ? code.replace(stringFinish, '') : code;
const fromF64Result = code.match(/fn\s+from_f64_result[\s\S]*?(?=\nfn\s+from_f64\s+<|$)/)?.[0] ?? '';

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
    assert.doesNotMatch(utf8Code, pattern, `${utf8RelPath} must not use unsafe unwrap helpers in implementation code`);
}

assert.match(utf8Code, /enum\s+StringUtf8LeadKind:/, 'alloc/string/utf8 must classify UTF-8 leading bytes with an enum');
assert.match(utf8Code, /fn\s+string_utf8_validate_mem\s+/, 'alloc/string/utf8 must own raw UTF-8 memory validation');
assert.match(code, /fn\s+string_from_utf8_mem_result\s+/, 'alloc/string must expose a checked UTF-8 construction API');
assert.match(code, /fn\s+str_utf8_is_boundary\s+/, 'alloc/string must validate UTF-8 slice boundaries');
assert.match(code, /fn\s+concat_result\s+/, 'alloc/string must keep allocation-bearing concat available as Result');
assert.match(code, /fn\s+str_slice_result\s+/, 'alloc/string must keep allocation-bearing slice available as Result');
assert.match(code, /fn\s+sb_build_result\s+/, 'StringBuilder build must have a Result-returning path');
assertStringBuilderOwnerBoundary(code);
assert.match(code, /fn\s+from_f64_result\s+/, 'from_f64 must have a Result-returning implementation path');
assert.notEqual(fromF64Result, '', 'from_f64_result body must be available for source policy checks');
assert.doesNotMatch(fromF64Result, /\b(?:scratch_raw|alloc_ptr<u8>\s+6|string_from_mem_unchecked_result)\b/, 'from_f64_result must not route fractional digits through raw scratch string construction');
assert.match(code, /fn\s+from_f64_build_fixed_result[\s\S]*string_builder_with_capacity_result[\s\S]*sb_build_result/, 'from_f64_result must build output through StringBuilder ownership APIs');
assert.doesNotMatch(code, /fn\s+str_split_result\s+<\(str,str\)->Result<Vec<str>,str>>/, 'alloc/string must not expose owned Vec<str> split until element cleanup is typed');
assert.doesNotMatch(code, /fn\s+str_split_ranges_result\s+<\(str,str\)->Result<Vec<i32>,str>>/, 'alloc/string must not expose allocation-bearing split range vectors while returned Vec owner summaries are incomplete');
assert.match(code, /enum\s+StrSplitStepKind:[\s\S]*Part[\s\S]*Done/, 'alloc/string must expose allocation-free split scanner state as an enum');
assert.match(code, /fn\s+str_split_next\s+<\(str,str,i32\)->StrSplitStep>/, 'alloc/string must expose allocation-free split scanning instead of owned Vec<str> split');
assert.match(code, /fn\s+sb_append_slice_result\s+/, 'StringBuilder must support appending source string ranges without allocating owned substrings');
assert.match(code, /fn\s+string_finish_base[\s\S]*store_i32\s+mem_ptr_addr\s+base\s+byte_len/, 'string_finish_base must use owned raw header store');
assert.match(stringFinish, /\bget\s+region\s+"ptr"/, 'string_finish must consume RegionToken at the final str ownership boundary');
assert.match(stringFinish, /\bstring_finish_base\s+base\s+byte_len\b/, 'string_finish must delegate raw header finalization to string_finish_base');
assert.match(fromU128Radix, /digit_count[\s\S]*string_alloc_region\s+digit_count[\s\S]*set\s+pos\s+sub\s+pos\s+1/, 'from_u128_radix must count digits before allocating and write output from the end');
assert.doesNotMatch(fromU128Radix, /scratch_raw/, 'from_u128_radix must not use scratch raw storage for digit reversal');
assert.match(code, /fn\s+str_slice_result[\s\S]*str_utf8_is_boundary\s+s\s+s0[\s\S]*str_utf8_is_boundary\s+s\s+e0/, 'str_slice_result must reject non-boundary UTF-8 byte ranges');
assert.doesNotMatch(codeWithoutStringFinish, /\bget\s+(?:region|out_region)\s+"ptr"/, 'alloc/string must read RegionToken ptr through get_ref except at the final string_finish ownership boundary');
assert.doesNotMatch(code, /\bstring_region_data_ptr\s+(?:region|out_region)\b/, 'alloc/string must pass RegionToken projections by reference');
assert.match(code, /fn\s+string_region_data_ptr\s+<\(&RegionToken<u8>\)->MemPtr<u8>>/, 'string_region_data_ptr must project from a RegionToken reference');

console.log('alloc/string unsafe unwrap regression passed');
