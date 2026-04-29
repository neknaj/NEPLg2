#!/usr/bin/env node

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');

const repoRoot = path.resolve(__dirname, '..');
const ioSrc = fs.readFileSync(path.join(repoRoot, 'stdlib/alloc/io.nepl'), 'utf8');
const stringSrc = fs.readFileSync(path.join(repoRoot, 'stdlib/alloc/string.nepl'), 'utf8');

const stripComments = (src) => src
    .split(/\r?\n/)
    .filter((line) => !/^\s*\/\//.test(line))
    .join('\n');

const ioCode = stripComments(ioSrc);
const stringCode = stripComments(stringSrc);

assert.match(
    ioCode,
    /struct\s+ByteBuilder:\s+ptr\s+<Option<MemPtr<u8>>>\s+len\s+<i32>\s+cap\s+<i32>/,
    'ByteBuilder must encode empty storage as Option::None instead of a null owning pointer',
);

assert.match(
    stringCode,
    /struct\s+StringBuilder:\s+data\s+<Option<MemPtr<u8>>>\s+len\s+<i32>\s+cap\s+<i32>/,
    'StringBuilder must encode empty storage as Option::None instead of a null owning pointer',
);

assert.match(
    ioCode,
    /\bfn\s+byte_builder_from_owned_ptr\s+<\(MemPtr<u8>,i32,i32\)->ByteBuilder>/,
    'ByteBuilder owned pointer construction must be centralized',
);

assert.match(
    stringCode,
    /\bfn\s+string_builder_from_owned_ptr\s+<\(MemPtr<u8>,i32,i32\)->StringBuilder>/,
    'StringBuilder owned pointer construction must be centralized',
);

assert.doesNotMatch(
    ioCode,
    /\bByteBuilder\s+mem_ptr_wrap\b/,
    'ByteBuilder must not encode empty storage as mem_ptr_wrap 0',
);

assert.doesNotMatch(
    stringCode,
    /\bStringBuilder\s+mem_ptr_wrap\b/,
    'StringBuilder must not encode empty storage as mem_ptr_wrap 0',
);

assert.doesNotMatch(
    ioCode,
    /\bResult<ByteBuilder,[^>]+>::Ok\s+ByteBuilder\s+(?!some<MemPtr<u8>>)/,
    'ByteBuilder Result return paths must use the centralized owned pointer constructor',
);

assert.doesNotMatch(
    stringCode,
    /\bResult<StringBuilder,[^>]+>::Ok\s+StringBuilder\s+(?!some<MemPtr<u8>>)/,
    'StringBuilder Result return paths must use the centralized owned pointer constructor',
);

assert.doesNotMatch(
    ioCode,
    /\b(realloc_ptr|mem_copy|store_u8)<u8>[^;\n]*\bget\s+(?:builder|reserved)\s+"ptr"/,
    'ByteBuilder storage access must first match the Option pointer',
);

assert.doesNotMatch(
    ioCode,
    /\bmatch\s+get\s+reserved\s+"ptr"/,
    'ByteBuilder append paths must borrow the reserved pointer and return the same owner',
);

assert.match(
    ioCode,
    /\bmatch\s+\*get_ref\s+&reserved\s+"ptr"/,
    'ByteBuilder append paths must inspect storage through a field reference',
);

assert.match(
    ioCode,
    /\bfn\s+byte_builder_with_len\s+<\(ByteBuilder,i32\)->ByteBuilder>/,
    'ByteBuilder must centralize len updates that preserve the storage owner field',
);

assert.match(
    ioCode,
    /\bbyte_builder_with_len\s+reserved\s+add\s+len0\b/,
    'ByteBuilder append paths must update len through the owner-preserving helper',
);

assert.doesNotMatch(
    stringCode,
    /\b(realloc_ptr|mem_copy|store_u8)<u8>[^;\n]*\bget\s+(?:sb|reserved)\s+"data"/,
    'StringBuilder storage access must first match the Option pointer',
);

assert.doesNotMatch(
    stringCode,
    /\bmatch\s+get\s+reserved\s+"data"/,
    'StringBuilder append paths must borrow the reserved pointer and return the same owner',
);

assert.match(
    stringCode,
    /\bmatch\s+\*get_ref\s+&reserved\s+"data"/,
    'StringBuilder append paths must inspect storage through a field reference',
);

assert.match(
    stringCode,
    /\bfn\s+string_builder_with_len\s+<\(StringBuilder,i32\)->StringBuilder>/,
    'StringBuilder must centralize len updates that preserve the storage owner field',
);

assert.match(
    stringCode,
    /\bstring_builder_with_len\s+reserved\s+add\s+len0\b/,
    'StringBuilder append paths must update len through the owner-preserving helper',
);

console.log('stdlib builder owner boundary regression passed');
