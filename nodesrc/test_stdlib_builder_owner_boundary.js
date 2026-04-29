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
const stringSrc = fs.readFileSync(path.join(repoRoot, 'stdlib/alloc/string.nepl'), 'utf8');

const ioCode = stripNeplComments(ioSrc);
const stringCode = stripNeplComments(stringSrc);

assertByteBuilderOwnerBoundary(ioCode);
assertStringBuilderOwnerBoundary(stringCode);

console.log('stdlib builder owner boundary regression passed');
