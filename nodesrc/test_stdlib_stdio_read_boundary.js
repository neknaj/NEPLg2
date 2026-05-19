#!/usr/bin/env node

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');

const repoRoot = path.resolve(__dirname, '..');
const relPath = 'stdlib/std/stdio.nepl';
const src = fs.readFileSync(path.join(repoRoot, relPath), 'utf8');
const rawRelPath = 'stdlib/std/stdio/raw.nepl';
const rawSrc = fs.readFileSync(path.join(repoRoot, rawRelPath), 'utf8');
const writeRelPath = 'stdlib/std/stdio/write.nepl';
const writeSrc = fs.readFileSync(path.join(repoRoot, writeRelPath), 'utf8');
const writeFdRelPath = 'stdlib/std/stdio/write/fd.nepl';
const writeFdSrc = fs.readFileSync(path.join(repoRoot, writeFdRelPath), 'utf8');
const writeTextRelPath = 'stdlib/std/stdio/write/text.nepl';
const writeTextSrc = fs.readFileSync(path.join(repoRoot, writeTextRelPath), 'utf8');
const writeBytesRelPath = 'stdlib/std/stdio/write/bytes.nepl';
const writeBytesSrc = fs.readFileSync(path.join(repoRoot, writeBytesRelPath), 'utf8');
const writeByteRelPath = 'stdlib/std/stdio/write/byte.nepl';
const writeByteSrc = fs.readFileSync(path.join(repoRoot, writeByteRelPath), 'utf8');
const readRelPath = 'stdlib/std/stdio/read.nepl';
const readSrc = fs.readFileSync(path.join(repoRoot, readRelPath), 'utf8');
const readBytesRelPath = 'stdlib/std/stdio/read/bytes.nepl';
const readBytesSrc = fs.readFileSync(path.join(repoRoot, readBytesRelPath), 'utf8');
const readTextRelPath = 'stdlib/std/stdio/read/text.nepl';
const readTextSrc = fs.readFileSync(path.join(repoRoot, readTextRelPath), 'utf8');
const readBufferRelPath = 'stdlib/std/stdio/read/buffer.nepl';
const readBufferSrc = fs.readFileSync(path.join(repoRoot, readBufferRelPath), 'utf8');

const code = src
    .split(/\r?\n/)
    .filter((line) => !/^\s*\/\//.test(line))
    .join('\n');
const rawCode = rawSrc
    .split(/\r?\n/)
    .filter((line) => !/^\s*\/\//.test(line))
    .join('\n');
const writeCode = writeSrc
    .split(/\r?\n/)
    .filter((line) => !/^\s*\/\//.test(line))
    .join('\n');
const writeFdCode = writeFdSrc
    .split(/\r?\n/)
    .filter((line) => !/^\s*\/\//.test(line))
    .join('\n');
const writeTextCode = writeTextSrc
    .split(/\r?\n/)
    .filter((line) => !/^\s*\/\//.test(line))
    .join('\n');
const writeBytesCode = writeBytesSrc
    .split(/\r?\n/)
    .filter((line) => !/^\s*\/\//.test(line))
    .join('\n');
const writeByteCode = writeByteSrc
    .split(/\r?\n/)
    .filter((line) => !/^\s*\/\//.test(line))
    .join('\n');
const readCode = readSrc
    .split(/\r?\n/)
    .filter((line) => !/^\s*\/\//.test(line))
    .join('\n');
const readBytesCode = readBytesSrc
    .split(/\r?\n/)
    .filter((line) => !/^\s*\/\//.test(line))
    .join('\n');
const readTextCode = readTextSrc
    .split(/\r?\n/)
    .filter((line) => !/^\s*\/\//.test(line))
    .join('\n');
const readBufferCode = readBufferSrc
    .split(/\r?\n/)
    .filter((line) => !/^\s*\/\//.test(line))
    .join('\n');

assert.doesNotMatch(
    code,
    /pub\s+#import\s+"\.\/stdio\/raw"\s+as\s+\*/,
    'std/stdio safe facade must not re-export raw stdio ABI submodule',
);
assert.match(
    code,
    /pub\s+#import\s+"\.\/stdio\/write"\s+as\s+\*/,
    'std/stdio facade must re-export stdio write submodule',
);
assert.match(
    code,
    /pub\s+#import\s+"\.\/stdio\/read"\s+as\s+\*/,
    'std/stdio facade must re-export stdio read submodule',
);

assert.doesNotMatch(
    code,
    /\bfn\s+std_(?:store|load)_i32_at\b/,
    'stdio must not reintroduce generic i32 raw-memory load/store helpers',
);

for (const helper of [
    'std_alloc',
    'std_free',
    'stdio_fd_read_mem',
    'stdio_fd_write_mem',
    'stdio_fd_write_from_result',
    '__linux_syscall_rw',
    'fd_read',
    'fd_write',
]) {
    assert.doesNotMatch(code, new RegExp(`\\bfn\\s+${helper}\\b`), `${helper} must stay in stdio/raw`);
    assert.doesNotMatch(code, new RegExp(`\\b${helper}\\b`), `std/stdio safe facade must not expose raw helper ${helper}`);
}

for (const helper of [
    'stdio_fd_read_mem',
    'stdio_fd_write_mem',
    'stdio_fd_write_from_result',
]) {
    assert.doesNotMatch(
        rawCode,
        new RegExp(`\\b(?:pub\\s+)?fn\\s+${helper}\\b`),
        `${helper} must not be public or private API in stdio/raw; MemPtr fd helpers must stay inside owner-boundary modules`,
    );
}
for (const helper of [
    'stdio_fd_read_raw',
    'stdio_fd_write_raw',
]) {
    assert.match(
        rawCode,
        new RegExp(`\\bpub\\s+fn\\s+${helper}\\s+<\\(i32,i32,i32,i32\\)\\*>i32>`),
        `${helper} must be the only public stdio/raw ABI wrapper shape`,
    );
}
for (const helper of ['std_alloc', 'std_free']) {
    assert.doesNotMatch(
        rawCode,
        new RegExp(`\\bfn\\s+${helper}\\b`),
        `${helper} must not be reintroduced as a unit wrapper that hides dealloc ownership`,
    );
}

assert.doesNotMatch(
    code,
    /\bfn\s+stdio_write_fd_mem_result\b/,
    'stdio_write_fd_mem_result must stay below stdio/write',
);
assert.doesNotMatch(
    writeCode,
    /\bfn\s+stdio_write_fd_mem_result\b/,
    'stdio_write_fd_mem_result must stay in stdio/write/fd',
);
assert.match(
    writeFdCode,
    /\bfn\s+stdio_write_fd_mem_result\b/,
    'stdio_write_fd_mem_result must exist as the private fd_write loop helper',
);
assert.match(
    writeFdCode,
    /\bfn\s+stdio_fd_write_from_result\s+<\(i32,MemPtr<u8>,MemPtr<u8>,MemPtr<u8>,i32\)\*>Result<i32,\s*StdErrorKind>>/,
    'stdio fd_write raw layout helper must exist in stdio/write/fd where scratch RegionToken owners are allocated',
);
assert.doesNotMatch(
    writeFdCode,
    /\bpub\s+fn\s+stdio_fd_write_from_result\b/,
    'stdio fd_write raw layout helper must not be public because caller-selected MemPtr spans must not be importable',
);
assert.doesNotMatch(
    writeFdCode,
    /\bpub\s+fn\s+stdio_write_fd_mem_result\b/,
    'stdio_write_fd_mem_result must not be public because raw MemPtr span writes require a typed source proof',
);
for (const helper of [
    'stdio_write_mem_result',
    'stdio_write_stderr_mem_result',
    'stdio_write_mem',
    'stdio_write_stderr_mem',
]) {
    assert.doesNotMatch(code, new RegExp(`\\bfn\\s+${helper}\\b`), `${helper} must not be reintroduced in std/stdio`);
    assert.doesNotMatch(writeCode, new RegExp(`\\bfn\\s+${helper}\\b`), `${helper} must not be reintroduced in stdio/write`);
    assert.doesNotMatch(
        writeFdCode,
        new RegExp(`\\bfn\\s+${helper}\\b`),
        `${helper} must not exist because it exposes caller-chosen MemPtr/length pairs`,
    );
}
for (const helper of [
    'stdio_write_fd_str_result',
    'stdio_write_fd_bytebuf_result',
    'stdio_write_fd_bytebuilder_prefix_result',
    'stdio_write_fd_byte_result',
]) {
    assert.match(
        writeFdCode,
        new RegExp(`\\bpub\\s+fn\\s+${helper}\\b`),
        `${helper} must be the public typed fd_write wrapper`,
    );
}
for (const helper of [
    'stdio_write_str_result',
    'stdio_write_stderr_str_result',
]) {
    assert.doesNotMatch(code, new RegExp(`\\bfn\\s+${helper}\\b`), `${helper} must stay below stdio/write`);
    assert.doesNotMatch(writeCode, new RegExp(`\\bfn\\s+${helper}\\b`), `${helper} must stay in stdio/write/text`);
    assert.match(writeTextCode, new RegExp(`\\bfn\\s+${helper}\\b`), `${helper} must exist in stdio/write/text`);
}
for (const helper of [
    'stdio_write_bytes_result',
    'stdio_write_stderr_bytes_result',
    'stdio_write_bytes',
    'stdio_write_stderr_bytes',
]) {
    assert.doesNotMatch(code, new RegExp(`\\bfn\\s+${helper}\\b`), `${helper} must stay below stdio/write`);
    assert.doesNotMatch(writeCode, new RegExp(`\\bfn\\s+${helper}\\b`), `${helper} must stay in stdio/write/bytes`);
    assert.match(writeBytesCode, new RegExp(`\\bfn\\s+${helper}\\b`), `${helper} must exist in stdio/write/bytes`);
}
for (const helper of [
    'print_byte',
]) {
    assert.doesNotMatch(code, new RegExp(`\\bfn\\s+${helper}\\b`), `${helper} must stay below stdio/write`);
    assert.doesNotMatch(writeCode, new RegExp(`\\bfn\\s+${helper}\\b`), `${helper} must stay in stdio/write/byte`);
    assert.match(writeByteCode, new RegExp(`\\bfn\\s+${helper}\\b`), `${helper} must exist in stdio/write/byte`);
}
for (const submodule of ['fd', 'text', 'bytes', 'byte']) {
    assert.match(
        writeCode,
        new RegExp(`pub\\s+#import\\s+"\\.\\/write\\/${submodule}"\\s+as\\s+@merge`),
        `std/stdio/write facade must re-export write/${submodule}`,
    );
}
assert.doesNotMatch(writeCode, /\bfn\s+/, 'std/stdio/write facade must not keep write implementation bodies');
const writeFdMatch = writeFdCode.match(
    /fn\s+stdio_write_fd_mem_result\s+<\(i32,MemPtr<u8>,i32\)\*>\s*Result<\(\),\s*StdErrorKind>>\s+\(fd,\s*data,\s*data_len\):([\s\S]*?)\n(?:pub\s+)?fn\s+stdio_write_fd_str_result\s+/,
);
assert.ok(writeFdMatch, 'stdio_write_fd_mem_result body must be found');
assert.match(
    writeFdCode,
    /#import\s+"std\/stdio\/raw"\s+as\s+\*/,
    'std/stdio/write/fd must import std/stdio/raw explicitly when crossing the raw ABI boundary',
);
assert.match(
    writeFdMatch[1],
    /\balloc_region<u8>/,
    'stdio fd_write scratch allocation must use RegionToken owners',
);
assert.match(
    writeFdMatch[1],
    /\bstdio_fd_write_from_result\b/,
    'stdio fd_write helper must delegate raw ABI layout to stdio/raw',
);
assert.match(
    writeFdMatch[1],
    /\bdealloc_region<u8>\s+nwritten_region[\s\S]*\bdealloc_region<u8>\s+iov_region\b/,
    'stdio fd_write private scratch owners must be explicitly consumed through typed owner cleanup',
);
assert.doesNotMatch(
    writeFdMatch[1],
    /\b(?:alloc_ptr|dealloc_ptr|mem_ptr_addr|store_i32|load_i32)\b/,
    'stdio fd_write loop must not directly own low-level MemPtr allocation or raw layout operations',
);
assert.doesNotMatch(
    writeFdMatch[1],
    /\bdealloc_raw\b/,
    'stdio fd_write must not recover free obligations from non-owning raw address views',
);
assert.doesNotMatch(
    writeFdMatch[1],
    /\bstd_free\b/,
    'stdio fd_write must not hide checked dealloc failures behind std_free',
);
assert.match(
    writeTextCode,
    /\bstdio_write_fd_str_result\s+1\s+s\b[\s\S]*\bstdio_write_fd_str_result\s+2\s+s\b/,
    'stdio text writes must derive their MemPtr/len pair through the typed str wrapper',
);
assert.doesNotMatch(
    writeTextCode,
    /\b(?:stdio_write_mem_result|stdio_write_stderr_mem_result|string_data_ptr|string_storage::string_data_ptr)\b/,
    'stdio text writes must not reconstruct raw string spans outside write/fd',
);
assert.match(
    writeBytesCode,
    /\bstdio_write_fd_bytebuf_result\s+1\s+bytes\b[\s\S]*\bstdio_write_fd_bytebuf_result\s+2\s+bytes\b/,
    'stdio ByteBuf writes must consume the ByteBuf through the typed fd wrapper',
);
assert.doesNotMatch(
    writeBytesCode,
    /\b(?:stdio_write_mem_result|stdio_write_stderr_mem_result|io_bytebuf_ptr_ref)\b/,
    'stdio ByteBuf writes must not expose raw ByteBuf pointers outside write/fd',
);
assert.match(
    writeByteCode,
    /\bstdio_write_fd_byte_result\s+1\s+b\b/,
    'print_byte must use the typed one-byte fd wrapper',
);
assert.doesNotMatch(
    writeByteCode,
    /\b(?:alloc_region|region_ptr|stdio_write_mem|stdio_write_fd_mem_result)\b/,
    'print_byte must not own raw scratch allocation or call the raw fd span helper directly',
);

for (const helper of [
    'stdio_fd_read_into_result',
    'stdio_discard_read_buffer',
    'stdio_finish_read_buffer',
]) {
    assert.doesNotMatch(code, new RegExp(`\\bfn\\s+${helper}\\b`), `${helper} must stay below stdio/read`);
    assert.doesNotMatch(readCode, new RegExp(`\\bfn\\s+${helper}\\b`), `${helper} must stay in stdio/read/buffer`);
    assert.doesNotMatch(readBytesCode, new RegExp(`\\bfn\\s+${helper}\\b`), `${helper} must stay in stdio/read/buffer`);
    assert.doesNotMatch(readTextCode, new RegExp(`\\bfn\\s+${helper}\\b`), `${helper} must stay in stdio/read/buffer`);
    assert.match(readBufferCode, new RegExp(`\\bfn\\s+${helper}\\b`), `${helper} must exist in stdio/read/buffer`);
}
assert.doesNotMatch(
    readBufferCode,
    /\bpub\s+fn\s+stdio_fd_read_into_result\b/,
    'stdio_fd_read_into_result must not be public because raw MemPtr fd_read spans must stay inside read/buffer',
);
assert.doesNotMatch(
    readBufferCode,
    /\b(?:pub\s+)?fn\s+stdio_fd_read_region_slice_result\b/,
    'stdio read buffer must not expose a lower-level fd_read wrapper that accepts caller-selected buffer slices',
);
assert.match(
    readBufferCode,
    /\bpub\s+fn\s+stdio_read_all_buffer_result\s+<\(\)\*>Result<ByteBuf,\s*StdErrorKind>>/,
    'stdio read buffer must expose the high-level read-all ByteBuf boundary',
);
assert.match(
    readBufferCode,
    /\bpub\s+fn\s+stdio_read_line_buffer_result\s+<\(\)\*>Result<ByteBuf,\s*StdErrorKind>>/,
    'stdio read buffer must expose the high-level read-line ByteBuf boundary',
);
assert.match(
    readBufferCode,
    /\bfn\s+stdio_read_all_buffer_result\b[\s\S]*\balloc_region<u8>\s+8[\s\S]*\balloc_region<u8>\s+4[\s\S]*\brealloc_region_bytes_keep<u8>[\s\S]*\bstdio_fd_read_into_result\b[\s\S]*\bdealloc_region<u8>\s+nread_region[\s\S]*\bdealloc_region<u8>\s+iov_region[\s\S]*\bstdio_finish_read_buffer\b/,
    'read-all buffer boundary must own fd_read scratch allocation, growth, cleanup, and exact-size ByteBuf finalization',
);
assert.match(
    readBufferCode,
    /\bfn\s+stdio_fd_read_into_result\b[\s\S]*\bstdio_fd_read_raw\s+fd\s+iov_raw\s+1\s+nread_raw\b/,
    'stdio fd_read raw layout helper must call the raw ABI using raw addresses derived inside read/buffer owner boundary',
);
assert.doesNotMatch(
    readBufferCode,
    /\bstdio_fd_read_mem\b/,
    'stdio read buffer must not depend on a public MemPtr fd_read wrapper in stdio/raw',
);
assert.match(
    readBufferCode,
    /\bfn\s+stdio_read_line_buffer_result\b[\s\S]*\balloc_region<u8>\s+8[\s\S]*\balloc_region<u8>\s+4[\s\S]*\bstdio_fd_read_into_result\b[\s\S]*\bload_u8\s+write_ptr[\s\S]*\bdealloc_region<u8>\s+nread_region[\s\S]*\bdealloc_region<u8>\s+iov_region[\s\S]*\bstdio_finish_read_buffer\b/,
    'read-line buffer boundary must own fd_read scratch allocation, byte inspection, cleanup, and ByteBuf finalization',
);

assert.match(
    readCode,
    /pub\s+#import\s+"\.\/read\/bytes"\s+as\s+@merge/,
    'std/stdio/read facade must re-export read/bytes',
);
assert.match(
    readCode,
    /pub\s+#import\s+"\.\/read\/text"\s+as\s+@merge/,
    'std/stdio/read facade must re-export read/text',
);
assert.doesNotMatch(
    readCode,
    /\bfn\s+/,
    'std/stdio/read facade must not keep read implementation bodies',
);
assert.match(
    readBytesCode,
    /#import\s+"\.\/buffer"\s+as\s+\*/,
    'std/stdio/read/bytes must depend on read/buffer boundary helpers',
);
assert.match(
    readBufferCode,
    /#import\s+"std\/stdio\/raw"\s+as\s+\*/,
    'std/stdio/read/buffer must import std/stdio/raw explicitly when crossing the raw ABI boundary',
);
assert.doesNotMatch(
    readBufferCode,
    /#import\s+"core\/mem\/pointer\/alloc"\s+as\s+\*/,
    'std/stdio/read/buffer must not import low-level MemPtr owner allocation wrappers',
);
assert.match(
    readBufferCode,
    /\bfn\s+stdio_discard_read_buffer\s+<\(RegionToken<u8>,StdErrorKind\)\*>/,
    'stdio read discard helper must consume a RegionToken owner, not a MemPtr owner',
);
assert.match(
    readBufferCode,
    /\bfn\s+stdio_finish_read_buffer\s+<\(RegionToken<u8>,i32\)\*>/,
    'stdio read finish helper must consume a RegionToken owner, not a MemPtr owner',
);
assert.match(
    readBufferCode,
    /\brealloc_region_bytes_keep<u8>/,
    'stdio read finish helper must shrink through owner-preserving RegionToken realloc',
);
assert.match(
    readTextCode,
    /#import\s+"\.\/buffer"\s+as\s+\*/,
    'std/stdio/read/text must depend on read/buffer boundary helpers',
);
assert.match(
    readTextCode,
    /#import\s+"\.\/bytes"\s+as\s+\*/,
    'std/stdio/read/text must build on read/bytes for read_all text conversion',
);

for (const helper of [
    'stdio_read_all_bytes_result',
    'stdio_read_all_bytes',
]) {
    assert.doesNotMatch(code, new RegExp(`\\bfn\\s+(?:noshadow\\s+)?${helper}\\b`), `${helper} must stay in stdio/read`);
    assert.doesNotMatch(readCode, new RegExp(`\\bfn\\s+(?:noshadow\\s+)?${helper}\\b`), `${helper} must stay in stdio/read/bytes`);
    assert.match(readBytesCode, new RegExp(`\\bfn\\s+(?:noshadow\\s+)?${helper}\\b`), `${helper} must exist in stdio/read/bytes`);
    assert.doesNotMatch(readTextCode, new RegExp(`\\bfn\\s+(?:noshadow\\s+)?${helper}\\b`), `${helper} must not be duplicated in stdio/read/text`);
}

for (const helper of [
    'stdio_read_all_text_result',
    'read_all',
    'stdio_read_line_result',
    'read_line',
]) {
    assert.doesNotMatch(code, new RegExp(`\\bfn\\s+(?:noshadow\\s+)?${helper}\\b`), `${helper} must stay in stdio/read/text`);
    assert.doesNotMatch(readCode, new RegExp(`\\bfn\\s+(?:noshadow\\s+)?${helper}\\b`), `${helper} must stay in stdio/read/text`);
    assert.match(readTextCode, new RegExp(`\\bfn\\s+(?:noshadow\\s+)?${helper}\\b`), `${helper} must exist in stdio/read/text`);
    assert.doesNotMatch(readBytesCode, new RegExp(`\\bfn\\s+(?:noshadow\\s+)?${helper}\\b`), `${helper} must not be duplicated in stdio/read/bytes`);
}

const readAllMatch = readBytesCode.match(
    /fn\s+stdio_read_all_bytes_result\s+<\(\)\*>\s*Result<ByteBuf\s*,\s*StdErrorKind>>\s+\(\):([\s\S]*?)\n(?:pub\s+)?fn\s+stdio_read_all_bytes\s+/,
);
assert.ok(readAllMatch, 'stdio_read_all_bytes_result body must be found');
assert.match(
    readAllMatch[1],
    /\bstdio_read_all_buffer_result\b/,
    'read_all bytes facade must delegate to the high-level stdio read buffer boundary',
);
assert.doesNotMatch(
    readAllMatch[1],
    /\b(?:stdio_fd_read_into_result|mem_ptr_add|region_ptr|alloc_region|realloc_region_bytes_keep|dealloc_region)\b/,
    'read_all bytes facade must not reconstruct fd_read scratch, raw views, or buffer ownership outside read/buffer',
);
assert.doesNotMatch(
    readAllMatch[1],
    /\b(?:alloc_ptr|realloc_ptr|dealloc_ptr|mem_ptr_addr|store_i32|load_i32)\b/,
    'read_all must not directly own low-level MemPtr allocation or raw layout operations',
);
assert.doesNotMatch(
    readTextCode,
    /#import\s+"core\/mem\/pointer\/alloc"\s+as\s+\*/,
    'std/stdio/read/text must not import low-level MemPtr owner allocation wrappers',
);
assert.doesNotMatch(
    readTextCode,
    /#import\s+"core\/mem\/raw"\s+as\s+\*/,
    'std/stdio/read/text must not import raw memory after read_line moves to RegionToken and checked scalar access',
);
assert.match(
    readTextCode,
    /fn\s+stdio_read_line_result\s+<\(\)\*>\s*Result<str\s*,\s*StdErrorKind>>\s+\(\):[\s\S]*\bmatch\s+stdio_read_line_buffer_result\b[\s\S]*\btext_bytebuf_to_utf8_str_result\s+bytes\b/,
    'stdio_read_line_result must delegate raw line reading to read/buffer and only perform UTF-8 conversion',
);
assert.doesNotMatch(
    readTextCode,
    /\b(?:stdio_fd_read_into_result|stdio_fd_read_region_slice_result|alloc_region|dealloc_region|region_ptr|mem_ptr_add|store_u8|load_u8)\b/,
    'read/text must not own fd_read scratch, raw views, or byte inspection after read_line moves to read/buffer',
);

assert.match(
    readTextCode,
    /fn\s+stdio_read_all_text_result\s+<\(\)\*>\s*Result<str\s*,\s*StdErrorKind>>\s+\(\):[\s\S]*stdio_read_all_bytes_result[\s\S]*text_bytebuf_to_utf8_str_result/,
    'stdio_read_all_text_result must convert the read/bytes result through std/text',
);

const readLineMatch = readTextCode.match(
    /fn\s+noshadow\s+read_line\s+<\(\)\*>\s*str>\s+\(\):([\s\S]*?)$/,
);
assert.ok(readLineMatch, 'read_line body must be found');
const readLineBody = readLineMatch[1];
assert.match(
    readLineBody,
    /\bmatch\s+stdio_read_line_result\b/,
    'read_line facade must delegate to stdio_read_line_result',
);

for (const pattern of [
    /\balloc_ptr\b/,
    /\bdealloc_ptr\b/,
    /\bstd_alloc\b/,
    /\bstd_free\b/,
    /\bstring_from_addr_unchecked\b/,
    /\bfd_read\b/,
    /\bstore_i32\b/,
    /\bload_i32\b/,
]) {
    assert.doesNotMatch(
        readLineBody,
        pattern,
        'read_line facade must not rebuild raw string or WASI scratch handling inline',
    );
}

console.log('stdlib stdio read boundary regression passed');
