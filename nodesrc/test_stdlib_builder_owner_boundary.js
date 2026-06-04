#!/usr/bin/env node

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const {
    stripNeplComments,
    assertByteBuilderOwnerBoundary,
    assertStringBuilderOwnerBoundary,
} = require('./source_policy/stdlib_builder_owner');

const repoRoot = path.resolve(__dirname, '..');
const ioRootSrc = fs.readFileSync(path.join(repoRoot, 'stdlib/alloc/io.nepl'), 'utf8');
const ioByteBuilderFacadeSrc = fs.readFileSync(path.join(repoRoot, 'stdlib/alloc/io/bytebuilder.nepl'), 'utf8');
const ioByteBuilderTypesSrc = fs.readFileSync(path.join(repoRoot, 'stdlib/alloc/io/bytebuilder/types.nepl'), 'utf8');
const ioByteBuilderStorageSrc = fs.readFileSync(path.join(repoRoot, 'stdlib/alloc/io/bytebuilder/storage.nepl'), 'utf8');
const ioByteBuilderAppendSrc = fs.readFileSync(path.join(repoRoot, 'stdlib/alloc/io/bytebuilder/append.nepl'), 'utf8');
const ioByteBuilderBuildSrc = fs.readFileSync(path.join(repoRoot, 'stdlib/alloc/io/bytebuilder/build.nepl'), 'utf8');
const stringRootSrc = fs.readFileSync(path.join(repoRoot, 'stdlib/alloc/string.nepl'), 'utf8');
const stringBuilderSrc = fs.readFileSync(path.join(repoRoot, 'stdlib/alloc/string/builder.nepl'), 'utf8');
const stringBuilderTypesSrc = fs.readFileSync(path.join(repoRoot, 'stdlib/alloc/string/builder/types.nepl'), 'utf8');
const stringBuilderReserveSrc = fs.readFileSync(path.join(repoRoot, 'stdlib/alloc/string/builder/reserve.nepl'), 'utf8');
const stringBuilderAppendSrc = fs.readFileSync(path.join(repoRoot, 'stdlib/alloc/string/builder/append.nepl'), 'utf8');
const stringBuilderBuildSrc = fs.readFileSync(path.join(repoRoot, 'stdlib/alloc/string/builder/build.nepl'), 'utf8');
const stringBuilderExtSrc = fs.readFileSync(path.join(repoRoot, 'stdlib/alloc/string/builder_ext.nepl'), 'utf8');

const ioRootCode = stripNeplComments(ioRootSrc);
const ioByteBuilderFacadeCode = stripNeplComments(ioByteBuilderFacadeSrc);
const ioByteBuilderCode = stripNeplComments([
    ioByteBuilderTypesSrc,
    ioByteBuilderStorageSrc,
    ioByteBuilderAppendSrc,
    ioByteBuilderBuildSrc,
].join('\n'));
const stringRootCode = stripNeplComments(stringRootSrc);
const stringBuilderCode = stripNeplComments([
    stringBuilderTypesSrc,
    stringBuilderReserveSrc,
    stringBuilderAppendSrc,
    stringBuilderBuildSrc,
    stringBuilderExtSrc,
].join('\n'));

assert.match(ioRootCode, /pub\s+#import\s+"\.\/io\/bytebuilder"\s+as\s+\*/, 'alloc/io root must re-export ByteBuilder APIs');
assert.doesNotMatch(ioRootCode, /struct\s+ByteBuilder:/, 'alloc/io root must not own ByteBuilder storage state');
assert.doesNotMatch(ioRootCode, /fn\s+byte_builder_reserve\b/, 'alloc/io root must not own ByteBuilder grow logic');
assert.doesNotMatch(ioRootCode, /fn\s+byte_builder_finish\b/, 'alloc/io root must not own ByteBuilder finalization logic');
assert.match(ioByteBuilderFacadeSrc, /pub\s+#import\s+"\.\/bytebuilder\/types"\s+as\s+@merge/, 'alloc/io/bytebuilder facade must merge ByteBuilder type APIs');
assert.match(ioByteBuilderFacadeSrc, /pub\s+#import\s+"\.\/bytebuilder\/storage"\s+as\s+@merge/, 'alloc/io/bytebuilder facade must merge storage APIs');
assert.match(ioByteBuilderFacadeSrc, /pub\s+#import\s+"\.\/bytebuilder\/append"\s+as\s+@merge/, 'alloc/io/bytebuilder facade must merge append APIs');
assert.match(ioByteBuilderFacadeSrc, /pub\s+#import\s+"\.\/bytebuilder\/build"\s+as\s+@merge/, 'alloc/io/bytebuilder facade must merge build APIs');
assert.doesNotMatch(ioByteBuilderFacadeCode, /\b(?:fn|struct|enum)\s+/, 'alloc/io/bytebuilder facade must not own implementation bodies');
assertByteBuilderOwnerBoundary(ioByteBuilderCode);
assert.match(stringRootCode, /pub\s+#import\s+"\.\/string\/builder"\s+as\s+\*/, 'alloc/string root must re-export StringBuilder APIs');
assert.doesNotMatch(stringRootCode, /struct\s+StringBuilder:/, 'alloc/string root must not own StringBuilder storage state');
assert.doesNotMatch(stringRootCode, /fn\s+string_builder_reserve_result\b/, 'alloc/string root must not own StringBuilder grow logic');
assert.doesNotMatch(stringRootCode, /fn\s+sb_append_result\b/, 'alloc/string root must not own StringBuilder str append logic');
assert.doesNotMatch(stringRootCode, /fn\s+sb_build_result\b/, 'alloc/string root must not own StringBuilder finalization logic');
assert.match(stringBuilderSrc, /pub\s+#import\s+"\.\/builder\/types"\s+as\s+@merge/, 'alloc/string/builder facade must merge StringBuilder type APIs');
assert.match(stringBuilderSrc, /pub\s+#import\s+"\.\/builder\/reserve"\s+as\s+@merge/, 'alloc/string/builder facade must merge reserve APIs');
assert.match(stringBuilderSrc, /pub\s+#import\s+"\.\/builder\/append"\s+as\s+@merge/, 'alloc/string/builder facade must merge append APIs');
assert.match(stringBuilderSrc, /pub\s+#import\s+"\.\/builder\/build"\s+as\s+@merge/, 'alloc/string/builder facade must merge build APIs');
assert.doesNotMatch(stripNeplComments(stringBuilderSrc), /\b(?:fn|struct|enum)\s+/, 'alloc/string/builder facade must not own implementation bodies');
assertStringBuilderOwnerBoundary(stringBuilderCode);

console.log('stdlib builder owner boundary regression passed');
