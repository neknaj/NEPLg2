#!/usr/bin/env node

const assert = require('node:assert/strict');

function stripNeplComments(src) {
    return src
        .split(/\r?\n/)
        .filter((line) => !/^\s*\/\//.test(line))
        .join('\n');
}

function assertByteBuilderOwnerBoundary(code) {
    assert.match(
        code,
        /struct\s+ByteBuilder:\s+ptr\s+<Option<MemPtr<u8>>>\s+len\s+<i32>\s+cap\s+<i32>/,
        'ByteBuilder must encode empty storage as Option::None instead of a null owning pointer',
    );

    assert.match(
        code,
        /\bfn\s+byte_builder_from_owned_ptr\s+<\(MemPtr<u8>,i32,i32\)->ByteBuilder>/,
        'ByteBuilder owned pointer construction must be centralized',
    );

    assert.doesNotMatch(
        code,
        /\bByteBuilder\s+mem_ptr_wrap\b/,
        'ByteBuilder must not encode empty storage as mem_ptr_wrap 0',
    );

    assert.doesNotMatch(
        code,
        /\bResult<ByteBuilder,[^>]+>::Ok\s+ByteBuilder\s+(?!some<MemPtr<u8>>)/,
        'ByteBuilder Result return paths must use the centralized owned pointer constructor',
    );

    assert.doesNotMatch(
        code,
        /\b(realloc_ptr|mem_copy|store_u8)<u8>[^;\n]*\bget\s+(?:builder|reserved)\s+"ptr"/,
        'ByteBuilder storage access must first match the Option pointer',
    );

    assert.doesNotMatch(
        code,
        /\bmatch\s+get\s+reserved\s+"ptr"/,
        'ByteBuilder append paths must borrow the reserved pointer and return the same owner',
    );

    assert.match(
        code,
        /\bmatch\s+\*get_ref\s+&reserved\s+"ptr"/,
        'ByteBuilder append paths must inspect storage through a field reference',
    );

    assert.match(
        code,
        /\bfn\s+byte_builder_with_len\s+<\(ByteBuilder,i32\)->ByteBuilder>/,
        'ByteBuilder must centralize len updates that preserve the storage owner field',
    );

    assert.match(
        code,
        /\bbyte_builder_with_len\s+reserved\s+add\s+len0\b/,
        'ByteBuilder append paths must update len through the owner-preserving helper',
    );
}

function assertStringBuilderOwnerBoundary(code) {
    assert.doesNotMatch(
        code,
        /struct\s+StringBuilder:[\s\S]*parts\s+<Vec<str>>/,
        'StringBuilder must not store non-Copy str payloads in Vec raw storage',
    );

    assert.match(
        code,
        /struct\s+StringBuilder:\s+data\s+<Option<MemPtr<u8>>>\s+len\s+<i32>\s+cap\s+<i32>/,
        'StringBuilder must encode empty storage as Option::None instead of a null owning pointer',
    );

    assert.match(
        code,
        /\bfn\s+string_builder_from_owned_ptr\s+<\(MemPtr<u8>,i32,i32\)->StringBuilder>/,
        'StringBuilder owned pointer construction must be centralized',
    );

    assert.doesNotMatch(
        code,
        /\bStringBuilder\s+mem_ptr_wrap\b/,
        'StringBuilder must not encode empty storage as mem_ptr_wrap 0',
    );

    assert.doesNotMatch(
        code,
        /\bResult<StringBuilder,[^>]+>::Ok\s+StringBuilder\s+(?!some<MemPtr<u8>>)/,
        'StringBuilder Result return paths must use the centralized owned pointer constructor',
    );

    assert.doesNotMatch(
        code,
        /\b(realloc_ptr|mem_copy|store_u8)<u8>[^;\n]*\bget\s+(?:sb|reserved)\s+"data"/,
        'StringBuilder storage access must first match the Option pointer',
    );

    assert.doesNotMatch(
        code,
        /\bmatch\s+get\s+reserved\s+"data"/,
        'StringBuilder append paths must borrow the reserved pointer and return the same owner',
    );

    assert.match(
        code,
        /\bmatch\s+\*get_ref\s+&reserved\s+"data"/,
        'StringBuilder append paths must inspect storage through a field reference',
    );

    assert.match(
        code,
        /\bfn\s+string_builder_with_len\s+<\(StringBuilder,i32\)->StringBuilder>/,
        'StringBuilder must centralize len updates that preserve the storage owner field',
    );

    assert.match(
        code,
        /\bstring_builder_with_len\s+reserved\s+add\s+len0\b/,
        'StringBuilder append paths must update len through the owner-preserving helper',
    );
}

module.exports = {
    stripNeplComments,
    assertByteBuilderOwnerBoundary,
    assertStringBuilderOwnerBoundary,
};
