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
const ioSrc = fs.readFileSync(path.join(repoRoot, 'stdlib/alloc/io.nepl'), 'utf8');
const stringRootSrc = fs.readFileSync(path.join(repoRoot, 'stdlib/alloc/string.nepl'), 'utf8');
const stringBuilderSrc = fs.readFileSync(path.join(repoRoot, 'stdlib/alloc/string/builder.nepl'), 'utf8');

const ioCode = stripNeplComments(ioSrc);
const stringRootCode = stripNeplComments(stringRootSrc);
const stringBuilderCode = stripNeplComments(stringBuilderSrc);

assertByteBuilderOwnerBoundary(ioCode);
assert.match(stringRootCode, /pub\s+#import\s+"\.\/string\/builder"\s+as\s+\*/, 'alloc/string root must re-export StringBuilder APIs');
assert.doesNotMatch(stringRootCode, /struct\s+StringBuilder:/, 'alloc/string root must not own StringBuilder storage state');
assert.doesNotMatch(stringRootCode, /fn\s+string_builder_reserve_result\b/, 'alloc/string root must not own StringBuilder grow logic');
assert.doesNotMatch(stringRootCode, /fn\s+sb_append_result\b/, 'alloc/string root must not own StringBuilder str append logic');
assert.doesNotMatch(stringRootCode, /fn\s+sb_build_result\b/, 'alloc/string root must not own StringBuilder finalization logic');
assertStringBuilderOwnerBoundary(stringBuilderCode);
assert.ok(stringBuilderSrc.split(/\r?\n/).length <= 560, 'alloc/string/builder should stay narrowly scoped');

console.log('stdlib builder owner boundary regression passed');
