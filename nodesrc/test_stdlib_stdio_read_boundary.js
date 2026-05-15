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
    assert.match(rawCode, new RegExp(`\\bfn\\s+${helper}\\b`), `${helper} must exist in stdio/raw`);
}
for (const helper of ['std_alloc', 'std_free']) {
    assert.doesNotMatch(
        rawCode,
        new RegExp(`\\bfn\\s+${helper}\\b`),
        `${helper} must not be reintroduced as a unit wrapper that hides dealloc ownership`,
    );
}

for (const helper of [
    'stdio_write_fd_mem_result',
    'stdio_write_mem_result',
    'stdio_write_stderr_mem_result',
    'stdio_write_mem',
    'stdio_write_stderr_mem',
]) {
    assert.doesNotMatch(code, new RegExp(`\\bfn\\s+${helper}\\b`), `${helper} must stay below stdio/write`);
    assert.doesNotMatch(writeCode, new RegExp(`\\bfn\\s+${helper}\\b`), `${helper} must stay in stdio/write/fd`);
    assert.match(writeFdCode, new RegExp(`\\bfn\\s+${helper}\\b`), `${helper} must exist in stdio/write/fd`);
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
    /fn\s+stdio_write_fd_mem_result\s+<\(i32,MemPtr<u8>,i32\)\*>\s*Result<\(\),\s*StdErrorKind>>\s+\(fd,\s*data,\s*data_len\):([\s\S]*?)\n(?:pub\s+)?fn\s+stdio_write_mem_result\s+/,
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
    /\bstdio_fd_read_into_result\b/,
    'read_all must use the stdio fd_read scratch boundary helper',
);
assert.match(
    readAllMatch[1],
    /\bstdio_finish_read_buffer\b/,
    'read_all must return an exact-size ByteBuf through stdio_finish_read_buffer',
);
assert.match(
    readAllMatch[1],
    /\balloc_region<u8>/,
    'read_all buffer and scratch allocation must use RegionToken owners',
);
assert.match(
    readAllMatch[1],
    /\brealloc_region_bytes_keep<u8>/,
    'read_all grow path must keep buffer ownership in RegionToken',
);
assert.match(
    readAllMatch[1],
    /\bdealloc_region<u8>\s+nread_region[\s\S]*\bdealloc_region<u8>\s+iov_region\b/,
    'read_all private fd_read scratch owners must be explicitly consumed through typed owner cleanup',
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
