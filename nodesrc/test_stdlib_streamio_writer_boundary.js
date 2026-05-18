#!/usr/bin/env node

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const { implementationLineCount } = require('./source_policy/stdlib_builder_owner');

const repoRoot = path.resolve(__dirname, '..');
const facadeRelPath = 'stdlib/std/streamio.nepl';
const writerRelPath = 'stdlib/std/streamio/writer.nepl';
const stateRelPath = 'stdlib/std/streamio/writer/state.nepl';
const appendRelPath = 'stdlib/std/streamio/writer/append.nepl';
const appendTextRelPath = 'stdlib/std/streamio/writer/append/text.nepl';
const appendByteBufRelPath = 'stdlib/std/streamio/writer/append/bytebuf.nepl';
const appendIntegerRelPath = 'stdlib/std/streamio/writer/append/integer.nepl';
const appendFloatRelPath = 'stdlib/std/streamio/writer/append/float.nepl';
const facade = fs.readFileSync(path.join(repoRoot, facadeRelPath), 'utf8');
const rootSrc = fs.readFileSync(path.join(repoRoot, writerRelPath), 'utf8');
const stateSrc = fs.readFileSync(path.join(repoRoot, stateRelPath), 'utf8');
const appendSrc = fs.readFileSync(path.join(repoRoot, appendRelPath), 'utf8');
const appendTextSrc = fs.readFileSync(path.join(repoRoot, appendTextRelPath), 'utf8');
const appendByteBufSrc = fs.readFileSync(path.join(repoRoot, appendByteBufRelPath), 'utf8');
const appendIntegerSrc = fs.readFileSync(path.join(repoRoot, appendIntegerRelPath), 'utf8');
const appendFloatSrc = fs.readFileSync(path.join(repoRoot, appendFloatRelPath), 'utf8');

function stripComments(src) {
    return src
        .split(/\r?\n/)
        .filter((line) => !/^\s*\/\//.test(line))
        .join('\n');
}

const rootCode = stripComments(rootSrc);
const stateCode = stripComments(stateSrc);
const appendCode = stripComments(appendSrc);
const appendTextCode = stripComments(appendTextSrc);
const appendByteBufCode = stripComments(appendByteBufSrc);
const appendIntegerCode = stripComments(appendIntegerSrc);
const appendFloatCode = stripComments(appendFloatSrc);
const code = [rootCode, stateCode, appendCode, appendTextCode, appendByteBufCode, appendIntegerCode, appendFloatCode].join('\n');
const facadeCode = facade
    .split(/\r?\n/)
    .filter((line) => !/^\s*\/\//.test(line))
    .join('\n');

for (const [relPath, src, maxLines] of [
    [writerRelPath, rootSrc, 180],
    [stateRelPath, stateSrc, 240],
    [appendRelPath, appendSrc, 80],
    [appendTextRelPath, appendTextSrc, 80],
    [appendByteBufRelPath, appendByteBufSrc, 110],
    [appendIntegerRelPath, appendIntegerSrc, 180],
    [appendFloatRelPath, appendFloatSrc, 130],
]) {
    const lineCount = implementationLineCount(src);
    assert.ok(lineCount <= maxLines, `${relPath} must stay within its responsibility boundary (${lineCount}/${maxLines})`);
}

assert.match(
    facadeCode,
    /pub\s+#import\s+"\.\/streamio\/writer"\s+as\s+\*/,
    'std/streamio.nepl must re-export the writer submodule',
);

for (const pattern of [
    /\bstruct\s+StreamWriter\b/,
    /\benum\s+StreamWriterTargetKind\b/,
    /\bfn\s+stream_writer_new\b/,
    /\bfn\s+stream_writer_close_impl\b/,
    /\bfn\s+drain_impl\b/,
    /\bfn\s+append_str_impl\b/,
]) {
    assert.doesNotMatch(
        facadeCode,
        pattern,
        'std/streamio.nepl facade must not keep writer implementation bodies',
    );
}

assert.match(
    rootCode,
    /#import\s+"std\/streamio\/writer\/state"\s+as\s+\*/,
    'streamio/writer root must import the writer state submodule',
);

assert.match(
    rootCode,
    /#import\s+"std\/streamio\/writer\/append"\s+as\s+\*/,
    'streamio/writer root must import the writer append submodule',
);

assert.match(
    appendCode,
    /pub\s+#import\s+"std\/streamio\/writer\/append\/text"\s+as\s+\*/,
    'streamio/writer/append facade must re-export text append helpers',
);
assert.match(
    appendCode,
    /pub\s+#import\s+"std\/streamio\/writer\/append\/bytebuf"\s+as\s+\*/,
    'streamio/writer/append facade must re-export ByteBuf append helpers',
);
assert.match(
    appendCode,
    /pub\s+#import\s+"std\/streamio\/writer\/append\/integer"\s+as\s+\*/,
    'streamio/writer/append facade must re-export integer append helpers',
);
assert.match(
    appendCode,
    /pub\s+#import\s+"std\/streamio\/writer\/append\/float"\s+as\s+\*/,
    'streamio/writer/append facade must re-export float append helpers',
);

assert.doesNotMatch(
    appendCode,
    /^\s*(struct|trait|impl|fn)\s/m,
    'streamio/writer/append facade must not keep implementation bodies',
);

for (const pattern of [
    /\bstruct\s+StreamWriter\b/,
    /\benum\s+StreamWriterTargetKind\b/,
    /\bfn\s+stream_writer_new\b/,
    /\bfn\s+stream_writer_close_impl\b/,
    /\bfn\s+drain_impl\b/,
    /\bfn\s+append_str_impl\b/,
    /\bfn\s+append_i32_impl\b/,
]) {
    assert.doesNotMatch(
        rootCode,
        pattern,
        'streamio/writer root must keep only public open/trait/write/flush/close API bodies',
    );
}

assert.doesNotMatch(
    code,
    /\bfn\s+stream_writer_header_ptr\b/,
    'StreamWriter must not reintroduce a RegionToken header pointer helper',
);

assert.doesNotMatch(
    code,
    /\bfn\s+stream_writer_header_region\b/,
    'StreamWriter must not reintroduce a raw header region',
);

assert.doesNotMatch(
    code,
    /\bfn\s+stream_writer_(header_off|load_header|store_header|load_header_ptr|load_target_kind|target_kind_code|target_kind_from_code)\b/,
    'StreamWriter must not reintroduce raw header state helpers or numeric target code helpers',
);

assert.match(
    stateCode,
    /struct\s+StreamWriter:\s+builder\s+<ByteBuilder>\s+target\s+<StreamWriterTargetKind>/,
    'StreamWriter state module must keep ByteBuilder ownership and target enum as visible struct fields',
);

assert.doesNotMatch(
    stateCode,
    /^\s+(?:buf|cap|write_len)\s+</m,
    'StreamWriter must not keep direct MemPtr/cap/write_len owner fields after ByteBuilder migration',
);

const writerNewMatch = stateCode.match(/(?:pub\s+)?fn\s+stream_writer_new\b([\s\S]*?)\n(?:pub\s+)?fn\s+stream_writer_close_impl\b/);
assert.ok(writerNewMatch, 'stream_writer_new body must be found');
assert.match(
    writerNewMatch[1],
    /\bmatch\s+byte_builder_with_capacity\s+4096:[\s\S]*Result<StreamWriter,str>::Ok\s+StreamWriter\s+builder\s+target\s+@stream_writer_noncopy_marker\b/,
    'stream_writer_new must allocate through ByteBuilder and return the builder owner as a StreamWriter field',
);

assert.match(
    stateCode,
    /\bfn\s+stream_writer_close_impl\s+<\(StreamWriter\)\*>\(\)>/,
    'writer state module must own the StreamWriter cleanup implementation helper',
);

assert.doesNotMatch(
    stateCode,
    /\bfn\s+close\s+<\(StreamWriter\)\*>\(\)>/,
    'writer state module must not own the public common-name close overload',
);

assert.match(
    rootCode,
    /\bfn\s+close\s+<\(StreamWriter\)\*>\(\)>\s+\(w\):\s*stream_writer_close_impl\s+w\b/,
    'streamio/writer root must expose owner-consuming close through the public facade',
);

const drainMatch = stateCode.match(/(?:pub\s+)?fn\s+drain_impl\b([\s\S]*?)\n(?:pub\s+)?fn\s+reserve_impl\b/);
assert.ok(drainMatch, 'drain_impl body must be found');
assert.match(
    drainMatch[1],
    /\bmatch\s+target:\s*[\s\S]*StreamWriterTargetKind::Stdout:[\s\S]*StreamWriterTargetKind::Stderr:/,
    'drain_impl must branch on StreamWriterTargetKind enum arms',
);
assert.match(
    drainMatch[1],
    /\blet\s+ptr\s+<MemPtr<u8>>\s+byte_builder_data_ptr_ref\s+&builder[\s\S]*stdio_write_mem\s+ptr\s+write_len[\s\S]*byte_builder_with_len\s+builder\s+0\s+target\b/,
    'drain_impl must flush through a non-owning ByteBuilder pointer view and reset builder length without moving raw owner into StreamWriter',
);
assert.doesNotMatch(
    drainMatch[1],
    /\beq\s+target(_kind)?\s+[01]\b/,
    'drain_impl must not branch on numeric target kind codes',
);

const writerOpenMatch = rootCode.match(/(?:pub\s+)?fn\s+open\s+<\(WriteStream\)([\s\S]*?)\n(?:pub\s+)?trait\s+StreamWritable\b/);
assert.ok(writerOpenMatch, 'WriteStream open body must be found');
assert.doesNotMatch(
    writerOpenMatch[1],
    /\blet\s+raw_res\b/,
    'WriteStream open must not keep the owning writer result in an intermediate raw local',
);

const appendStrMatch = appendTextCode.match(/(?:pub\s+)?fn\s+append_str_impl\b([\s\S]*)$/);
assert.ok(appendStrMatch, 'append_str_impl body must be found');
assert.match(
    appendStrMatch[1],
    /\bstring_byte_at_checked_or_unreachable\s+s\s+i\b/,
    'append_str_impl must use alloc/string byte access instead of raw data loads',
);
assert.doesNotMatch(
    appendStrMatch[1],
    /\bload_u8\s+mem_ptr_add\s+src\s+i\b/,
    'append_str_impl must not directly load from string_data_ptr',
);

const appendByteBufMatch = appendByteBufCode.match(/(?:pub\s+)?fn\s+append_bytebuf_impl\b([\s\S]*)$/);
assert.ok(appendByteBufMatch, 'append_bytebuf_impl body must be found');
assert.match(
    appendByteBufMatch[1],
    /\bstream_writer_bytebuf_byte_at\s+&bytes\s+i\b/,
    'append_bytebuf_impl must use the borrowed ByteBuf byte helper',
);
assert.doesNotMatch(
    appendByteBufMatch[1],
    /\bload_u8\s+mem_ptr_add\s+src\s+i\b/,
    'append_bytebuf_impl must not directly load from the ByteBuf pointer',
);

assert.doesNotMatch(
    appendTextCode,
    /\bByteBuf\b/,
    'text append module must not own ByteBuf behavior',
);

assert.doesNotMatch(
    appendByteBufCode,
    /\bappend_i32_impl\b/,
    'ByteBuf append module must not own numeric formatting behavior',
);

assert.match(
    appendIntegerCode,
    /fn\s+append_i32_impl\b[\s\S]*fn\s+append_i64_impl\b/,
    'integer append module must own signed and unsigned integer formatting helpers',
);

assert.doesNotMatch(
    appendIntegerCode,
    /\bappend_f64_impl\b/,
    'integer append module must not own floating-point formatting behavior',
);

assert.match(
    appendFloatCode,
    /#import\s+"std\/streamio\/writer\/append\/integer"\s+as\s+\*[\s\S]*fn\s+append_f64_fixed_impl\b[\s\S]*append_u64_impl/,
    'float append module must reuse integer append for the integral part',
);

console.log('stdlib streamio writer boundary regression passed');
