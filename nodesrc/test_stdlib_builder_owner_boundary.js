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
const ioByteBuilderSrc = fs.readFileSync(path.join(repoRoot, 'stdlib/alloc/io/bytebuilder.nepl'), 'utf8');
const stringRootSrc = fs.readFileSync(path.join(repoRoot, 'stdlib/alloc/string.nepl'), 'utf8');
const stringBuilderSrc = fs.readFileSync(path.join(repoRoot, 'stdlib/alloc/string/builder.nepl'), 'utf8');

const ioRootCode = stripNeplComments(ioRootSrc);
const ioByteBuilderCode = stripNeplComments(ioByteBuilderSrc);
const stringRootCode = stripNeplComments(stringRootSrc);
const stringBuilderCode = stripNeplComments(stringBuilderSrc);

assert.match(ioRootCode, /pub\s+#import\s+"\.\/io\/bytebuilder"\s+as\s+\*/, 'alloc/io root must re-export ByteBuilder APIs');
assert.doesNotMatch(ioRootCode, /struct\s+ByteBuilder:/, 'alloc/io root must not own ByteBuilder storage state');
assert.doesNotMatch(ioRootCode, /fn\s+byte_builder_reserve\b/, 'alloc/io root must not own ByteBuilder grow logic');
assert.doesNotMatch(ioRootCode, /fn\s+byte_builder_finish\b/, 'alloc/io root must not own ByteBuilder finalization logic');
assertByteBuilderOwnerBoundary(ioByteBuilderCode);
assert.ok(ioByteBuilderSrc.split(/\r?\n/).length <= 460, 'alloc/io/bytebuilder should stay narrowly scoped');
assert.match(stringRootCode, /pub\s+#import\s+"\.\/string\/builder"\s+as\s+\*/, 'alloc/string root must re-export StringBuilder APIs');
assert.doesNotMatch(stringRootCode, /struct\s+StringBuilder:/, 'alloc/string root must not own StringBuilder storage state');
assert.doesNotMatch(stringRootCode, /fn\s+string_builder_reserve_result\b/, 'alloc/string root must not own StringBuilder grow logic');
assert.doesNotMatch(stringRootCode, /fn\s+sb_append_result\b/, 'alloc/string root must not own StringBuilder str append logic');
assert.doesNotMatch(stringRootCode, /fn\s+sb_build_result\b/, 'alloc/string root must not own StringBuilder finalization logic');
assertStringBuilderOwnerBoundary(stringBuilderCode);
assert.ok(stringBuilderSrc.split(/\r?\n/).length <= 560, 'alloc/string/builder should stay narrowly scoped');

console.log('stdlib builder owner boundary regression passed');
