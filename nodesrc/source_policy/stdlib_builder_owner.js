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
        /struct\s+ByteBuilder:\s+region\s+<RegionToken<u8>>\s+len\s+<i32>\s+cap\s+<i32>/,
        'ByteBuilder must keep byte storage ownership in a RegionToken field',
    );

    assert.match(
        code,
        /\bfn\s+byte_builder_from_owned_region\s+<\(RegionToken<u8>,i32,i32\)->ByteBuilder>/,
        'ByteBuilder owned region construction must be centralized',
    );

    assert.doesNotMatch(
        code,
        /\bByteBuilder\s+region_new\s+mem_ptr_wrap\b/,
        'ByteBuilder must not inline null raw pointer ownership in constructor call sites',
    );

    assert.doesNotMatch(
        code,
        /\bResult<ByteBuilder,[^>]+>::Ok\s+ByteBuilder\s+(?!byte_builder_empty_region\b|region\b)/,
        'ByteBuilder Result return paths must use the centralized owned region constructor',
    );

    assert.doesNotMatch(
        code,
        /\b(?:mem_copy|store_u8)<u8>[^;\n]*\bget\s+(?:builder|reserved)\s+"region"/,
        'ByteBuilder raw writes must project a non-owning pointer view from a RegionToken reference',
    );

    assert.doesNotMatch(
        code,
        /\b(realloc_ptr|dealloc_ptr)<u8>[^;\n]*\bmem_ptr_addr\s+get\s+(?:builder|reserved)\s+"region"/,
        'ByteBuilder must not recover ownership from a raw address extracted out of RegionToken',
    );

    assert.match(
        code,
        /\bget_ref\s+&reserved\s+"region"[\s\S]*\bregion_ptr\s+region_ref\b/,
        'ByteBuilder append paths must borrow RegionToken and write through a non-owning pointer view',
    );

    assert.match(
        code,
        /\bfn\s+byte_builder_with_len\s+<\(ByteBuilder,i32\)->ByteBuilder>/,
        'ByteBuilder must centralize len updates that preserve the storage owner field',
    );

    for (const [name, pattern] of [
        ['byte_builder_free', /\bfn\s+byte_builder_free\s+<\(ByteBuilder\)->\(\)>/],
        ['byte_builder_reserve', /\bfn\s+byte_builder_reserve\s+<\(ByteBuilder,i32\)->Result<ByteBuilder,\s*StdErrorKind>>/],
        ['byte_builder_push_u8', /\bfn\s+byte_builder_push_u8\s+<\(ByteBuilder,i32\)->Result<ByteBuilder,\s*StdErrorKind>>/],
        ['byte_builder_push_bytes_ref', /\bfn\s+byte_builder_push_bytes_ref\s+<\(ByteBuilder,&MemPtr<u8>,i32\)->Result<ByteBuilder,\s*StdErrorKind>>/],
        ['byte_builder_finish', /\bfn\s+byte_builder_finish\s+<\(ByteBuilder\)->Result<ByteBuf,\s*StdErrorKind>>/],
    ]) {
        assert.match(
            code,
            pattern,
            `${name} must expose a pure safe surface; raw memory effects remain checked inside the source boundary`,
        );
    }

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
        /struct\s+StringBuilder:\s+bytes\s+<ByteBuilder>/,
        'StringBuilder must delegate byte storage ownership to ByteBuilder instead of duplicating raw MemPtr state',
    );

    assert.doesNotMatch(
        code,
        /struct\s+StringBuilder:[\s\S]*\b(?:data|ptr)\s+<Option<MemPtr<u8>>>/,
        'StringBuilder must not keep a direct Option<MemPtr<u8>> owner field',
    );

    assert.doesNotMatch(
        code,
        /\bStringBuilder\s+mem_ptr_wrap\b/,
        'StringBuilder must not encode empty storage as mem_ptr_wrap 0',
    );

    assert.doesNotMatch(
        code,
        /\b(?:alloc_ptr|realloc_ptr|dealloc_ptr)<u8>/,
        'StringBuilder must not allocate or free raw byte storage directly; use ByteBuilder',
    );

    assert.doesNotMatch(
        code,
        /\b(?:mem_copy|store_u8)<u8>/,
        'StringBuilder append/build must not perform raw byte mutation directly; use ByteBuilder',
    );

    assert.doesNotMatch(
        code,
        /\bget(?:_ref)?\s+(?:&)?(?:sb|reserved)\s+"(?:data|ptr|cap|len)"/,
        'StringBuilder must not inspect a duplicated raw byte layout',
    );

    assert.match(
        code,
        /\bfn\s+string_builder_from_byte_builder\s+<\(ByteBuilder\)->StringBuilder>/,
        'StringBuilder must centralize wrapping a ByteBuilder owner',
    );

    assert.match(
        code,
        /\bfn\s+string_builder_into_byte_builder\s+<\(StringBuilder\)->ByteBuilder>/,
        'StringBuilder must centralize consuming conversion back to ByteBuilder',
    );

    assert.match(
        code,
        /\bbyte_builder_push_bytes_ref\b[\s\S]*\bbyte_builder_push_char_utf8\b[\s\S]*\bbyte_builder_push_ascii\b[\s\S]*\bbyte_builder_push_u8\b/,
        'StringBuilder append APIs must delegate all byte writes to ByteBuilder',
    );

    assert.match(
        code,
        /\bbyte_builder_finish\b[\s\S]*\bio_bytebuf_to_str_result\b/,
        'StringBuilder build must finalize through ByteBuilder and ByteBuf typed owner boundaries',
    );
}

module.exports = {
    stripNeplComments,
    assertByteBuilderOwnerBoundary,
    assertStringBuilderOwnerBoundary,
};
