#!/usr/bin/env node

const assert = require('node:assert/strict');

function stripNeplComments(src) {
    return src
        .split(/\r?\n/)
        .filter((line) => !/^\s*\/\//.test(line))
        .join('\n');
}

function implementationLineCount(src) {
    return stripNeplComments(src)
        .split(/\r?\n/)
        .filter((line) => line.trim().length > 0)
        .length;
}

function assertByteBuilderOwnerBoundary(code) {
    assert.match(
        code,
        /enum\s+ByteBuilderStorage:\s+Empty\s+Owned\s+<RegionToken<u8>>/,
        'ByteBuilder storage state must distinguish empty storage from owned RegionToken payload structurally',
    );

    assert.match(
        code,
        /struct\s+ByteBuilder:\s+storage\s+<ByteBuilderStorage>\s+len\s+<i32>\s+cap\s+<i32>/,
        'ByteBuilder must keep byte storage ownership in ByteBuilderStorage instead of a loose RegionToken field',
    );

    assert.match(
        code,
        /\bfn\s+byte_builder_from_owned_region\s+<\(RegionToken<u8>,i32,i32\)->ByteBuilder>/,
        'ByteBuilder owned region construction must be centralized',
    );

    assert.doesNotMatch(
        code,
        /\b(?:pub\s+)?fn\s+byte_builder_empty_region\b/,
        'ByteBuilder must not encode empty storage with a zero-size RegionToken sentinel helper',
    );

    assert.doesNotMatch(
        code,
        /\bregion_new\b[\s\S]{0,80}\b0\b/,
        'ByteBuilder empty storage must not forge a zero-size RegionToken sentinel',
    );

    assert.match(
        code,
        /\bpub\s+fn\s+byte_builder_empty\s+<\(\)->ByteBuilder>/,
        'ByteBuilder typed empty constructor must remain public',
    );

    assert.match(
        code,
        /\bpub\s+fn\s+byte_builder_empty\s+<\(\)->ByteBuilder>\s+\(\):\s+ByteBuilder\s+ByteBuilderStorage::Empty\s+0\s+0\b/,
        'ByteBuilder typed empty constructor must use the structural empty storage state',
    );

    assert.match(
        code,
        /\bfn\s+byte_builder_from_owned_region\s+<\(RegionToken<u8>,i32,i32\)->ByteBuilder>[\s\S]*?\bByteBuilder\s+\(ByteBuilderStorage::Owned\s+region\)\s+len0\s+cap0\b/,
        'ByteBuilder owned region constructor must wrap the RegionToken in ByteBuilderStorage::Owned',
    );

    assert.doesNotMatch(
        code,
        /\bByteBuilder\s+region_new\s+mem_ptr_wrap\b/,
        'ByteBuilder must not inline null raw pointer ownership in constructor call sites',
    );

    assert.doesNotMatch(
        code,
        /\bByteBuilder\s+byte_builder_empty_region\b/,
        'ByteBuilder constructor call sites must not use an empty RegionToken sentinel',
    );

    assert.doesNotMatch(
        code,
        /\bResult<ByteBuilder,[^>]+>::Ok\s+ByteBuilder\b/,
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
        /\bfn\s+byte_builder_realloc_region_or_keep\s+<\(RegionToken<u8>,i32\)->Result<RegionToken<u8>,\s*RegionReallocError<u8>>>[\s\S]*\brealloc_region_bytes_keep<u8>\s+region\s+new_cap\b/,
        'ByteBuilder grow must delegate RegionToken realloc to core/mem and keep the returned owner on failure',
    );

    assert.doesNotMatch(
        code,
        /\bfn\s+byte_builder_realloc_region_or_keep\b[\s\S]*\bdealloc_ptr<u8>\s+old_ptr\s+old_size/,
        'ByteBuilder grow failure cleanup must not split RegionToken into raw ptr/size owner cleanup',
    );

    assert.doesNotMatch(
        code,
        /\bfn\s+byte_builder_realloc_region_or_keep\b[\s\S]*\bget\s+region\s+"(?:size|ptr)"/,
        'ByteBuilder grow must not reimplement RegionToken realloc by reading RegionToken internals',
    );

    assert.doesNotMatch(
        code,
        /\bfn\s+byte_builder_realloc_region_or_keep\b[\s\S]*\brealloc_ptr<u8>\b/,
        'ByteBuilder grow must not call raw MemPtr realloc directly; core/mem owns RegionToken realloc',
    );

    assert.match(
        code,
        /\bfn\s+byte_builder_free\s+<\(ByteBuilder\)->\(\)>[\s\S]*?\bmatch\s+get\s+builder\s+"storage":[\s\S]*?\bByteBuilderStorage::Empty:[\s\S]*?\bByteBuilderStorage::Owned\s+region:[\s\S]*?\bbyte_builder_dealloc_owned_region\s+region\b/,
        'ByteBuilder free must match storage state and consume only the Owned RegionToken payload',
    );

    assert.match(
        code,
        /\bfn\s+byte_builder_reserve\s+<\(ByteBuilder,i32\)->Result<ByteBuilder,\s*ByteBuilderError>>[\s\S]*?\bmatch\s+get\s+builder\s+"storage":[\s\S]*?\bByteBuilderStorage::Empty:[\s\S]*?\balloc_region_bytes<u8>\s+next_cap[\s\S]*?\bByteBuilderStorage::Owned\s+region:[\s\S]*?\bbyte_builder_realloc_region_or_keep\s+region\s+next_cap\b/,
        'ByteBuilder reserve must allocate only from Empty and reallocate only from the Owned RegionToken payload',
    );

    assert.match(
        code,
        /\bget_ref\s+&reserved\s+"storage"[\s\S]*?\bmatch\s+storage_ref:[\s\S]*?\bByteBuilderStorage::Empty:[\s\S]*?\bByteBuilderStorage::Owned\s+region:[\s\S]*?\bregion_ptr\s+region\b/,
        'ByteBuilder append paths must match storage state and borrow a non-owning pointer view only from Owned payload',
    );

    assert.match(
        code,
        /\bfn\s+byte_builder_with_len\s+<\(ByteBuilder,i32\)->ByteBuilder>/,
        'ByteBuilder must centralize len updates that preserve the storage owner field',
    );

    assert.match(
        code,
        /\bfn\s+byte_builder_with_len\s+<\(ByteBuilder,i32\)->ByteBuilder>[\s\S]*?\bByteBuilder\s+get\s+builder\s+"storage"\s+new_len\s+cap0\b/,
        'ByteBuilder len update helper must preserve the ByteBuilderStorage owner field',
    );

    for (const [name, pattern] of [
        ['byte_builder_free', /\bfn\s+byte_builder_free\s+<\(ByteBuilder\)->\(\)>/],
        ['byte_builder_reserve', /\bfn\s+byte_builder_reserve\s+<\(ByteBuilder,i32\)->Result<ByteBuilder,\s*ByteBuilderError>>/],
        ['byte_builder_push_u8', /\bfn\s+byte_builder_push_u8\s+<\(ByteBuilder,i32\)->Result<ByteBuilder,\s*ByteBuilderError>>/],
        ['byte_builder_push_str', /\bfn\s+byte_builder_push_str\s+<\(ByteBuilder,str\)->Result<ByteBuilder,\s*ByteBuilderError>>/],
        ['byte_builder_push_str_slice', /\bfn\s+byte_builder_push_str_slice\s+<\(ByteBuilder,str,i32,i32\)->Result<ByteBuilder,\s*ByteBuilderError>>/],
        ['byte_builder_finish', /\bfn\s+byte_builder_finish\s+<\(ByteBuilder\)->Result<ByteBuf,\s*ByteBuilderError>>/],
    ]) {
        assert.match(
            code,
            pattern,
            `${name} must expose a pure safe surface; raw memory effects remain checked inside the source boundary`,
        );
    }

    assert.doesNotMatch(
        code,
        /\bpub\s+fn\s+byte_builder_push_bytes_ref\b/,
        'ByteBuilder must not expose raw MemPtr plus length append as public API',
    );

    assert.match(
        code,
        /\bfn\s+byte_builder_push_bytes_ref\s+<\(ByteBuilder,&MemPtr<u8>,i32\)->Result<ByteBuilder,\s*ByteBuilderError>>/,
        'ByteBuilder may keep the raw copy helper only as a private implementation detail',
    );

    assert.match(
        code,
        /\bfn\s+byte_builder_push_str\s+<\(ByteBuilder,str\)->Result<ByteBuilder,\s*ByteBuilderError>>[\s\S]*\blet\s+s_len\s+<i32>\s+len\s+s[\s\S]*\blet\s+src\s+<MemPtr<u8>>\s+string_data_ptr\s+s[\s\S]*\bbyte_builder_push_bytes_ref\s+builder\s+&src\s+s_len/,
        'ByteBuilder full string append must derive source pointer and length from the same str value',
    );

    assert.match(
        code,
        /\bfn\s+byte_builder_push_str_slice\s+<\(ByteBuilder,str,i32,i32\)->Result<ByteBuilder,\s*ByteBuilderError>>[\s\S]*\blet\s+n\s+<i32>\s+len\s+s[\s\S]*\bor\s+lt\s+start\s+0\s+or\s+lt\s+end\s+start\s+gt\s+end\s+n[\s\S]*\blet\s+data_len\s+<i32>\s+sub\s+end\s+start[\s\S]*\blet\s+src\s+<MemPtr<u8>>\s+mem_ptr_add\s+string_data_ptr\s+s\s+start[\s\S]*\bbyte_builder_push_bytes_ref\s+builder\s+&src\s+data_len/,
        'ByteBuilder string slice append must prove the readable range from str length before raw copy',
    );

    assert.match(
        code,
        /struct\s+ByteBuilderError:\s+builder\s+<ByteBuilder>\s+error\s+<StdErrorKind>/,
        'ByteBuilder fallible owner-consuming APIs must return the consumed builder owner in the error payload',
    );

    assert.match(
        code,
        /struct\s+ByteBuilderByteBufError:\s+builder\s+<ByteBuilder>\s+bytes\s+<ByteBuf>\s+error\s+<StdErrorKind>/,
        'ByteBuilder ByteBuf append failures must expose both consumed owners in the error payload',
    );

    assert.doesNotMatch(
        code,
        /\bfn\s+byte_builder_(?:reserve|push_u8|push_ascii|push_char_utf8|push_str|push_str_slice|push_leb_u32|finish)\s+<[^>\n]*->Result<(?:ByteBuilder|ByteBuf),\s*StdErrorKind>/,
        'ByteBuilder owner-consuming fallible APIs must not return bare StdErrorKind errors',
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
        /\bbyte_builder_push_str\b[\s\S]*\bbyte_builder_push_char_utf8\b[\s\S]*\bbyte_builder_push_ascii\b[\s\S]*\bbyte_builder_push_u8\b/,
        'StringBuilder append APIs must delegate all byte writes to typed ByteBuilder helpers',
    );

    assert.match(
        code,
        /\bbyte_builder_push_str_slice\b/,
        'StringBuilder slice append must delegate source range copy to the typed ByteBuilder slice helper',
    );

    assert.match(
        code,
        /\bbyte_builder_finish\b[\s\S]*\bio_bytebuf_to_str_result\b/,
        'StringBuilder build must finalize through ByteBuilder and ByteBuf typed owner boundaries',
    );
}

module.exports = {
    stripNeplComments,
    implementationLineCount,
    assertByteBuilderOwnerBoundary,
    assertStringBuilderOwnerBoundary,
};
