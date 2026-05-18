#!/usr/bin/env node

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const {
    stripNeplComments,
    implementationLineCount,
    assertStringBuilderOwnerBoundary,
} = require('./source_policy/stdlib_builder_owner');

const repoRoot = path.resolve(__dirname, '..');
const relPath = 'stdlib/alloc/string.nepl';
const src = fs.readFileSync(path.join(repoRoot, relPath), 'utf8');
const utf8RelPath = 'stdlib/alloc/string/utf8.nepl';
const utf8Src = fs.readFileSync(path.join(repoRoot, utf8RelPath), 'utf8');
const storageRelPath = 'stdlib/alloc/string/storage.nepl';
const storageSrc = fs.readFileSync(path.join(repoRoot, storageRelPath), 'utf8');
const accessRelPath = 'stdlib/alloc/string/access.nepl';
const accessSrc = fs.readFileSync(path.join(repoRoot, accessRelPath), 'utf8');
const builderRelPath = 'stdlib/alloc/string/builder.nepl';
const builderSrc = fs.readFileSync(path.join(repoRoot, builderRelPath), 'utf8');
const builderTypesRelPath = 'stdlib/alloc/string/builder/types.nepl';
const builderTypesSrc = fs.readFileSync(path.join(repoRoot, builderTypesRelPath), 'utf8');
const builderReserveRelPath = 'stdlib/alloc/string/builder/reserve.nepl';
const builderReserveSrc = fs.readFileSync(path.join(repoRoot, builderReserveRelPath), 'utf8');
const builderAppendRelPath = 'stdlib/alloc/string/builder/append.nepl';
const builderAppendSrc = fs.readFileSync(path.join(repoRoot, builderAppendRelPath), 'utf8');
const builderBuildRelPath = 'stdlib/alloc/string/builder/build.nepl';
const builderBuildSrc = fs.readFileSync(path.join(repoRoot, builderBuildRelPath), 'utf8');
const searchRelPath = 'stdlib/alloc/string/search.nepl';
const searchSrc = fs.readFileSync(path.join(repoRoot, searchRelPath), 'utf8');
const searchCompareRelPath = 'stdlib/alloc/string/search/compare.nepl';
const searchCompareSrc = fs.readFileSync(path.join(repoRoot, searchCompareRelPath), 'utf8');
const searchBoundaryRelPath = 'stdlib/alloc/string/search/boundary.nepl';
const searchBoundarySrc = fs.readFileSync(path.join(repoRoot, searchBoundaryRelPath), 'utf8');
const searchByteFindRelPath = 'stdlib/alloc/string/search/byte_find.nepl';
const searchByteFindSrc = fs.readFileSync(path.join(repoRoot, searchByteFindRelPath), 'utf8');
const sliceRelPath = 'stdlib/alloc/string/slice.nepl';
const sliceSrc = fs.readFileSync(path.join(repoRoot, sliceRelPath), 'utf8');
const sliceByteRelPath = 'stdlib/alloc/string/slice/byte.nepl';
const sliceByteSrc = fs.readFileSync(path.join(repoRoot, sliceByteRelPath), 'utf8');
const sliceCharRelPath = 'stdlib/alloc/string/slice/char.nepl';
const sliceCharSrc = fs.readFileSync(path.join(repoRoot, sliceCharRelPath), 'utf8');
const sliceTrimRelPath = 'stdlib/alloc/string/slice/trim.nepl';
const sliceTrimSrc = fs.readFileSync(path.join(repoRoot, sliceTrimRelPath), 'utf8');
const splitRelPath = 'stdlib/alloc/string/split.nepl';
const splitSrc = fs.readFileSync(path.join(repoRoot, splitRelPath), 'utf8');
const integerRelPath = 'stdlib/alloc/string/integer.nepl';
const integerSrc = fs.readFileSync(path.join(repoRoot, integerRelPath), 'utf8');
const integerFormatRelPath = 'stdlib/alloc/string/integer/format.nepl';
const integerFormatSrc = fs.readFileSync(path.join(repoRoot, integerFormatRelPath), 'utf8');
const integerParseRelPath = 'stdlib/alloc/string/integer/parse.nepl';
const integerParseSrc = fs.readFileSync(path.join(repoRoot, integerParseRelPath), 'utf8');
const floatRelPath = 'stdlib/alloc/string/float.nepl';
const floatSrc = fs.readFileSync(path.join(repoRoot, floatRelPath), 'utf8');
const floatFormatRelPath = 'stdlib/alloc/string/float/format.nepl';
const floatFormatSrc = fs.readFileSync(path.join(repoRoot, floatFormatRelPath), 'utf8');
const floatParseRelPath = 'stdlib/alloc/string/float/parse.nepl';
const floatParseSrc = fs.readFileSync(path.join(repoRoot, floatParseRelPath), 'utf8');
const concatRelPath = 'stdlib/alloc/string/concat.nepl';
const concatSrc = fs.readFileSync(path.join(repoRoot, concatRelPath), 'utf8');
const builderExtRelPath = 'stdlib/alloc/string/builder_ext.nepl';
const builderExtSrc = fs.readFileSync(path.join(repoRoot, builderExtRelPath), 'utf8');
const findRelPath = 'stdlib/alloc/string/find.nepl';
const findSrc = fs.readFileSync(path.join(repoRoot, findRelPath), 'utf8');

const code = stripNeplComments(src);
const utf8Code = stripNeplComments(utf8Src);
const storageCode = stripNeplComments(storageSrc);
const accessCode = stripNeplComments(accessSrc);
const builderCode = stripNeplComments(builderSrc);
const builderTypesCode = stripNeplComments(builderTypesSrc);
const builderReserveCode = stripNeplComments(builderReserveSrc);
const builderAppendCode = stripNeplComments(builderAppendSrc);
const builderBuildCode = stripNeplComments(builderBuildSrc);
const builderCombinedCode = [
    builderTypesCode,
    builderReserveCode,
    builderAppendCode,
    builderBuildCode,
].join('\n');
const searchCode = stripNeplComments(searchSrc);
const searchCompareCode = stripNeplComments(searchCompareSrc);
const searchBoundaryCode = stripNeplComments(searchBoundarySrc);
const searchByteFindCode = stripNeplComments(searchByteFindSrc);
const sliceCode = stripNeplComments(sliceSrc);
const sliceByteCode = stripNeplComments(sliceByteSrc);
const sliceCharCode = stripNeplComments(sliceCharSrc);
const sliceTrimCode = stripNeplComments(sliceTrimSrc);
const splitCode = stripNeplComments(splitSrc);
const integerCode = stripNeplComments(integerSrc);
const integerFormatCode = stripNeplComments(integerFormatSrc);
const integerParseCode = stripNeplComments(integerParseSrc);
const floatCode = stripNeplComments(floatSrc);
const floatFormatCode = stripNeplComments(floatFormatSrc);
const floatParseCode = stripNeplComments(floatParseSrc);
const concatCode = stripNeplComments(concatSrc);
const builderExtCode = stripNeplComments(builderExtSrc);
const findCode = stripNeplComments(findSrc);
const fromU128Radix = integerFormatCode.match(/(?:pub\s+)?fn\s+from_u128_radix[\s\S]*?(?=\n(?:pub\s+)?fn\s+from_i128|\n\/\/ from_i128|$)/)?.[0] ?? '';
const stringFinish = storageCode.match(/(?:pub\s+)?fn\s+string_finish\s+<\(RegionToken<u8>,i32\)->str>\s+\(region,\s*byte_len\):[\s\S]*?(?=\n(?:pub\s+)?fn\s+string_from_addr_unchecked\s+)/)?.[0] ?? '';
const storageCodeWithoutStringFinish = stringFinish ? storageCode.replace(stringFinish, '') : storageCode;
const fromF64Result = floatFormatCode.match(/(?:pub\s+)?fn\s+from_f64_result[\s\S]*?(?=\n(?:pub\s+)?fn\s+from_f64\s+<|$)/)?.[0] ?? '';

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
    assert.doesNotMatch(storageCode, pattern, `${storageRelPath} must not use unsafe unwrap helpers in implementation code`);
    assert.doesNotMatch(accessCode, pattern, `${accessRelPath} must not use unsafe unwrap helpers in implementation code`);
    assert.doesNotMatch(builderCode, pattern, `${builderRelPath} must not use unsafe unwrap helpers in implementation code`);
    assert.doesNotMatch(builderTypesCode, pattern, `${builderTypesRelPath} must not use unsafe unwrap helpers in implementation code`);
    assert.doesNotMatch(builderReserveCode, pattern, `${builderReserveRelPath} must not use unsafe unwrap helpers in implementation code`);
    assert.doesNotMatch(builderAppendCode, pattern, `${builderAppendRelPath} must not use unsafe unwrap helpers in implementation code`);
    assert.doesNotMatch(builderBuildCode, pattern, `${builderBuildRelPath} must not use unsafe unwrap helpers in implementation code`);
    assert.doesNotMatch(searchCode, pattern, `${searchRelPath} must not use unsafe unwrap helpers in implementation code`);
    assert.doesNotMatch(searchCompareCode, pattern, `${searchCompareRelPath} must not use unsafe unwrap helpers in implementation code`);
    assert.doesNotMatch(searchBoundaryCode, pattern, `${searchBoundaryRelPath} must not use unsafe unwrap helpers in implementation code`);
    assert.doesNotMatch(searchByteFindCode, pattern, `${searchByteFindRelPath} must not use unsafe unwrap helpers in implementation code`);
    assert.doesNotMatch(sliceCode, pattern, `${sliceRelPath} must not use unsafe unwrap helpers in implementation code`);
    assert.doesNotMatch(sliceByteCode, pattern, `${sliceByteRelPath} must not use unsafe unwrap helpers in implementation code`);
    assert.doesNotMatch(sliceCharCode, pattern, `${sliceCharRelPath} must not use unsafe unwrap helpers in implementation code`);
    assert.doesNotMatch(sliceTrimCode, pattern, `${sliceTrimRelPath} must not use unsafe unwrap helpers in implementation code`);
    assert.doesNotMatch(splitCode, pattern, `${splitRelPath} must not use unsafe unwrap helpers in implementation code`);
    assert.doesNotMatch(integerCode, pattern, `${integerRelPath} must not use unsafe unwrap helpers in implementation code`);
    assert.doesNotMatch(integerFormatCode, pattern, `${integerFormatRelPath} must not use unsafe unwrap helpers in implementation code`);
    assert.doesNotMatch(integerParseCode, pattern, `${integerParseRelPath} must not use unsafe unwrap helpers in implementation code`);
    assert.doesNotMatch(floatCode, pattern, `${floatRelPath} must not use unsafe unwrap helpers in implementation code`);
    assert.doesNotMatch(floatFormatCode, pattern, `${floatFormatRelPath} must not use unsafe unwrap helpers in implementation code`);
    assert.doesNotMatch(floatParseCode, pattern, `${floatParseRelPath} must not use unsafe unwrap helpers in implementation code`);
    assert.doesNotMatch(concatCode, pattern, `${concatRelPath} must not use unsafe unwrap helpers in implementation code`);
    assert.doesNotMatch(builderExtCode, pattern, `${builderExtRelPath} must not use unsafe unwrap helpers in implementation code`);
    assert.doesNotMatch(findCode, pattern, `${findRelPath} must not use unsafe unwrap helpers in implementation code`);
}

assert.match(utf8Code, /enum\s+StringUtf8LeadKind:/, 'alloc/string/utf8 must classify UTF-8 leading bytes with an enum');
assert.match(utf8Code, /fn\s+string_utf8_validate_mem\s+/, 'alloc/string/utf8 must own raw UTF-8 memory validation');
assert.match(storageCode, /fn\s+string_from_utf8_mem_result\s+/, 'alloc/string/storage must expose a checked UTF-8 construction API');
assert.match(searchCode, /pub\s+#import\s+"\.\/search\/compare"\s+as\s+@merge/, 'alloc/string/search facade must merge byte comparison APIs for qualified imports');
assert.match(searchCode, /pub\s+#import\s+"\.\/search\/boundary"\s+as\s+@merge/, 'alloc/string/search facade must merge UTF-8 boundary APIs for qualified imports');
assert.match(searchCode, /pub\s+#import\s+"\.\/search\/byte_find"\s+as\s+@merge/, 'alloc/string/search facade must merge byte find APIs for qualified imports');
assert.doesNotMatch(searchCode, /\bfn\s+/, 'alloc/string/search facade must not own implementation function bodies');
assert.match(searchCompareCode, /fn\s+str_eq\s+/, 'alloc/string/search/compare must own byte equality');
assert.match(searchCompareCode, /fn\s+str_starts_with_at\s+/, 'alloc/string/search/compare must own bounded prefix checks');
assert.match(searchCompareCode, /fn\s+str_range_eq\s+/, 'alloc/string/search/compare must own range equality');
assert.match(searchBoundaryCode, /fn\s+str_utf8_is_boundary\s+/, 'alloc/string/search/boundary must validate UTF-8 slice boundaries');
assert.match(searchByteFindCode, /fn\s+str_find\s+/, 'alloc/string/search/byte_find must own byte find loops');
assert.ok(implementationLineCount(searchSrc) <= 35, 'alloc/string/search facade should stay small');
assert.ok(implementationLineCount(searchCompareSrc) <= 270, 'alloc/string/search/compare should stay narrowly scoped');
assert.ok(implementationLineCount(searchBoundarySrc) <= 75, 'alloc/string/search/boundary should stay narrowly scoped');
assert.ok(implementationLineCount(searchByteFindSrc) <= 125, 'alloc/string/search/byte_find should stay narrowly scoped');
assert.match(code, /pub\s+#import\s+"\.\/string\/concat"\s+as\s+\*/, 'alloc/string must re-export concat APIs');
assert.match(code, /pub\s+#import\s+"\.\/string\/builder_ext"\s+as\s+\*/, 'alloc/string must re-export StringBuilder extension APIs');
assert.match(code, /pub\s+#import\s+"\.\/string\/find"\s+as\s+\*/, 'alloc/string must re-export Option-returning find API');
assert.match(concatCode, /fn\s+concat_result\s+/, 'alloc/string/concat must keep allocation-bearing concat available as Result');
assert.match(sliceCode, /pub\s+#import\s+"\.\/slice\/byte"\s+as\s+@merge/, 'alloc/string/slice facade must merge byte slice APIs for qualified imports');
assert.match(sliceCode, /pub\s+#import\s+"\.\/slice\/char"\s+as\s+@merge/, 'alloc/string/slice facade must merge char traversal APIs for qualified imports');
assert.match(sliceCode, /pub\s+#import\s+"\.\/slice\/trim"\s+as\s+@merge/, 'alloc/string/slice facade must merge trim APIs for qualified imports');
assert.doesNotMatch(sliceCode, /\bfn\s+/, 'alloc/string/slice facade must not own implementation function bodies');
assert.match(sliceByteCode, /fn\s+str_slice_result\s+/, 'alloc/string/slice/byte must keep allocation-bearing slice available as Result');
assert.match(builderCode, /pub\s+#import\s+"\.\/builder\/types"\s+as\s+@merge/, 'alloc/string/builder facade must merge StringBuilder type APIs');
assert.match(builderCode, /pub\s+#import\s+"\.\/builder\/reserve"\s+as\s+@merge/, 'alloc/string/builder facade must merge reserve APIs');
assert.match(builderCode, /pub\s+#import\s+"\.\/builder\/append"\s+as\s+@merge/, 'alloc/string/builder facade must merge append APIs');
assert.match(builderCode, /pub\s+#import\s+"\.\/builder\/build"\s+as\s+@merge/, 'alloc/string/builder facade must merge build APIs');
assert.doesNotMatch(builderCode, /\b(?:fn|struct|enum)\s+/, 'alloc/string/builder facade must not own implementation bodies');
assert.match(builderBuildCode, /fn\s+sb_build_result\s+/, 'StringBuilder build must have a Result-returning path');
assertStringBuilderOwnerBoundary(builderCombinedCode);
assert.match(code, /pub\s+#import\s+"\.\/string\/float"\s+as\s+\*/, 'alloc/string must re-export float conversion APIs');
assert.match(floatCode, /pub\s+#import\s+"\.\/float\/format"\s+as\s+\*/, 'alloc/string/float facade must re-export float formatting APIs');
assert.match(floatCode, /pub\s+#import\s+"\.\/float\/parse"\s+as\s+\*/, 'alloc/string/float facade must re-export float parsing APIs');
assert.doesNotMatch(floatCode, /\bfn\s+/, 'alloc/string/float facade must not own implementation function bodies');
assert.match(floatFormatCode, /fn\s+from_f64_result\s+/, 'from_f64 must have a Result-returning implementation path');
assert.notEqual(fromF64Result, '', 'from_f64_result body must be available for source policy checks');
assert.doesNotMatch(fromF64Result, /\b(?:scratch_raw|alloc_ptr<u8>\s+6|string_from_mem_unchecked_result)\b/, 'from_f64_result must not route fractional digits through raw scratch string construction');
assert.match(floatFormatCode, /fn\s+from_f64_build_fixed_result[\s\S]*string_alloc_region[\s\S]*string_finish/, 'from_f64_result must build output through the fixed-size string storage boundary');
assert.doesNotMatch(floatFormatCode, /\bstring_builder_with_capacity_result\b/, 'from_f64_result must not route fixed-size output through growable StringBuilder owner chains');
assert.doesNotMatch(code, /fn\s+str_split_result\s+<\(str,str\)->Result<Vec<str>,str>>/, 'alloc/string must not expose owned Vec<str> split until element cleanup is typed');
assert.doesNotMatch(code, /fn\s+str_split_ranges_result\s+<\(str,str\)->Result<Vec<i32>,str>>/, 'alloc/string must not expose allocation-bearing split range vectors while returned Vec owner summaries are incomplete');
assert.match(code, /pub\s+#import\s+"\.\/string\/split"\s+as\s+\*/, 'alloc/string must re-export allocation-free split scanner APIs');
assert.match(splitCode, /enum\s+StrSplitStepKind:[\s\S]*Part[\s\S]*Done/, 'alloc/string/split must expose allocation-free split scanner state as an enum');
assert.match(splitCode, /fn\s+str_split_next\s+<\(str,str,i32\)->StrSplitStep>/, 'alloc/string/split must expose allocation-free split scanning instead of owned Vec<str> split');
assert.doesNotMatch(code, /enum\s+StrSplitStepKind:/, 'alloc/string root must not own split scanner state');
assert.doesNotMatch(code, /fn\s+str_split_next\s+<\(str,str,i32\)->StrSplitStep>/, 'alloc/string root must not own allocation-free split scanning');
assert.match(code, /pub\s+#import\s+"\.\/string\/integer"\s+as\s+\*/, 'alloc/string must re-export integer conversion APIs');
assert.match(integerCode, /pub\s+#import\s+"\.\/integer\/format"\s+as\s+\*/, 'alloc/string/integer facade must re-export integer formatting APIs');
assert.match(integerCode, /pub\s+#import\s+"\.\/integer\/parse"\s+as\s+\*/, 'alloc/string/integer facade must re-export integer parsing APIs');
assert.doesNotMatch(integerCode, /\bfn\s+/, 'alloc/string/integer facade must not own implementation function bodies');
assert.match(integerFormatCode, /fn\s+from_i32_radix\s+/, 'alloc/string/integer/format must own i32 formatting');
assert.match(integerParseCode, /fn\s+to_i128_radix\s+/, 'alloc/string/integer/parse must own i128 parsing');
assert.doesNotMatch(code, /fn\s+to_i128_radix\s+/, 'alloc/string root must not own integer parsing');
assert.match(builderExtCode, /fn\s+sb_append_slice_result\s+/, 'StringBuilder must support appending source string ranges without allocating owned substrings');
assert.match(findCode, /fn\s+find\s+<\(str,str\)->Option<i32>>/, 'alloc/string/find must expose Option-returning byte search');
assert.doesNotMatch(code, /\bfn\s+/, 'alloc/string root facade must not own implementation function bodies');
assert.doesNotMatch(storageCode, /\bfn\s+string_finish_base\b/, 'alloc/string/storage must not keep a MemPtr-based string finalizer');
assert.doesNotMatch(storageCode, /\bpub\s+fn\s+string_addr\b/, 'string_addr must stay private to string_data_ptr');
assert.doesNotMatch(storageCode, /\bpub\s+fn\s+string_from_addr_unchecked\b/, 'string_from_addr_unchecked must stay private to string_finish');
assert.match(stringFinish, /\blet\s+base_raw\s+<i32>\s+get\s+region\s+"raw"/, 'string_finish must consume RegionToken raw owner identity at the final str ownership boundary');
assert.match(stringFinish, /\bstore_i32\s+base_raw\s+byte_len\b[\s\S]*\bstring_from_addr_unchecked\s+base_raw\b/, 'string_finish must finalize the header and transfer the same raw owner directly into str');
assert.match(fromU128Radix, /digit_count[\s\S]*string_alloc_region\s+digit_count[\s\S]*set\s+pos\s+sub\s+pos\s+1/, 'from_u128_radix must count digits before allocating and write output from the end');
assert.doesNotMatch(fromU128Radix, /scratch_raw/, 'from_u128_radix must not use scratch raw storage for digit reversal');
assert.match(sliceByteCode, /fn\s+str_slice_result[\s\S]*str_utf8_is_boundary\s+s\s+s0[\s\S]*str_utf8_is_boundary\s+s\s+e0/, 'str_slice_result must reject non-boundary UTF-8 byte ranges');
assert.doesNotMatch(storageCodeWithoutStringFinish, /\bget\s+(?:region|out_region)\s+"raw"/, 'alloc/string/storage must not consume RegionToken raw owner identity except at the final string_finish ownership boundary');
assert.doesNotMatch(storageCode, /\bstring_region_data_ptr\s+(?:region|out_region)\b/, 'alloc/string/storage must pass RegionToken projections by reference');
assert.match(storageCode, /fn\s+string_region_data_ptr\s+<\(&RegionToken<u8>\)->MemPtr<u8>>/, 'string_region_data_ptr must project from a RegionToken reference');

console.log('alloc/string unsafe unwrap regression passed');
