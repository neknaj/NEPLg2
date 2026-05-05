#!/usr/bin/env node

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');

const repoRoot = path.resolve(__dirname, '..');
const facadeRelPath = 'stdlib/std/streamio.nepl';
const writerRelPath = 'stdlib/std/streamio/writer.nepl';
const facade = fs.readFileSync(path.join(repoRoot, facadeRelPath), 'utf8');
const src = fs.readFileSync(path.join(repoRoot, writerRelPath), 'utf8');

const code = src
    .split(/\r?\n/)
    .filter((line) => !/^\s*\/\//.test(line))
    .join('\n');
const facadeCode = facade
    .split(/\r?\n/)
    .filter((line) => !/^\s*\/\//.test(line))
    .join('\n');

assert.match(
    facadeCode,
    /pub\s+#import\s+"\.\/streamio\/writer"\s+as\s+\*/,
    'std/streamio.nepl must re-export the writer submodule',
);

for (const pattern of [
    /\bstruct\s+StreamWriter\b/,
    /\benum\s+StreamWriterTargetKind\b/,
    /\btrait\s+StreamWritable\b/,
    /\bfn\s+stream_writer_new\b/,
    /\bfn\s+drain_impl\b/,
    /\bfn\s+append_str_impl\b/,
]) {
    assert.doesNotMatch(
        facadeCode,
        pattern,
        'std/streamio.nepl facade must not keep writer implementation bodies',
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
    code,
    /struct\s+StreamWriter:\s+buf\s+<MemPtr<u8>>\s+cap\s+<i32>\s+write_len\s+<i32>\s+target\s+<StreamWriterTargetKind>/,
    'StreamWriter must keep buffer ownership and target enum as visible struct fields',
);

const writerNewMatch = code.match(/fn\s+stream_writer_new\b([\s\S]*?)\nfn\s+open\s+<\(WriteStream\)/);
assert.ok(writerNewMatch, 'stream_writer_new body must be found');
assert.match(
    writerNewMatch[1],
    /\bResult<StreamWriter,str>::Ok\s+StreamWriter\s+buf\s+4096\s+0\s+target\s+@stream_writer_noncopy_marker\b/,
    'stream_writer_new must return the buffer owner as a StreamWriter field',
);

const drainMatch = code.match(/fn\s+drain_impl\b([\s\S]*?)\nfn\s+reserve_impl\b/);
assert.ok(drainMatch, 'drain_impl body must be found');
assert.match(
    drainMatch[1],
    /\bmatch\s+target:\s*[\s\S]*StreamWriterTargetKind::Stdout:[\s\S]*StreamWriterTargetKind::Stderr:/,
    'drain_impl must branch on StreamWriterTargetKind enum arms',
);
assert.doesNotMatch(
    drainMatch[1],
    /\beq\s+target(_kind)?\s+[01]\b/,
    'drain_impl must not branch on numeric target kind codes',
);

const writerOpenMatch = code.match(/fn\s+open\s+<\(WriteStream\)([\s\S]*?)\nfn\s+close\b/);
assert.ok(writerOpenMatch, 'WriteStream open body must be found');
assert.doesNotMatch(
    writerOpenMatch[1],
    /\blet\s+raw_res\b/,
    'WriteStream open must not keep the owning writer result in an intermediate raw local',
);

const appendStrMatch = code.match(/fn\s+append_str_impl\b([\s\S]*?)\nfn\s+append_bytebuf_impl\b/);
assert.ok(appendStrMatch, 'append_str_impl body must be found');
assert.match(
    appendStrMatch[1],
    /\bstring_byte_at_unchecked\s+s\s+i\b/,
    'append_str_impl must use alloc/string byte access instead of raw data loads',
);
assert.doesNotMatch(
    appendStrMatch[1],
    /\bload_u8\s+mem_ptr_add\s+src\s+i\b/,
    'append_str_impl must not directly load from string_data_ptr',
);

const appendByteBufMatch = code.match(/fn\s+append_bytebuf_impl\b([\s\S]*?)\nfn\s+append_i32_digits_impl\b/);
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

console.log('stdlib streamio writer boundary regression passed');
