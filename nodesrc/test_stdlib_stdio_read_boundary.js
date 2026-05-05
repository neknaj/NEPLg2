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
const readRelPath = 'stdlib/std/stdio/read.nepl';
const readSrc = fs.readFileSync(path.join(repoRoot, readRelPath), 'utf8');
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
const readCode = readSrc
    .split(/\r?\n/)
    .filter((line) => !/^\s*\/\//.test(line))
    .join('\n');
const readBufferCode = readBufferSrc
    .split(/\r?\n/)
    .filter((line) => !/^\s*\/\//.test(line))
    .join('\n');

assert.match(
    code,
    /pub\s+#import\s+"\.\/stdio\/raw"\s+as\s+\*/,
    'std/stdio facade must re-export raw stdio ABI submodule',
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
    '__linux_syscall_rw',
    'fd_read',
    'fd_write',
]) {
    assert.doesNotMatch(code, new RegExp(`\\bfn\\s+${helper}\\b`), `${helper} must stay in stdio/raw`);
}

for (const helper of [
    'stdio_fd_read_mem',
    'stdio_fd_write_mem',
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
    'stdio_write_bytes_result',
    'print_byte',
]) {
    assert.doesNotMatch(code, new RegExp(`\\bfn\\s+${helper}\\b`), `${helper} must stay in stdio/write`);
    assert.match(writeCode, new RegExp(`\\bfn\\s+${helper}\\b`), `${helper} must exist in stdio/write`);
}
const writeFdMatch = writeCode.match(
    /fn\s+stdio_write_fd_mem_result\s+<\(i32,MemPtr<u8>,i32\)\*>\s*Result<\(\),\s*StdErrorKind>>\s+\(fd,\s*data,\s*data_len\):([\s\S]*?)\nfn\s+stdio_write_mem_result\s+/,
);
assert.ok(writeFdMatch, 'stdio_write_fd_mem_result body must be found');
assert.match(
    writeFdMatch[1],
    /\balloc_ptr<u8>/,
    'stdio fd_write scratch allocation must use MemPtr owners',
);
assert.match(
    writeFdMatch[1],
    /\bstdio_fd_write_mem\b/,
    'stdio fd_write helper must use the MemPtr ABI wrapper',
);
assert.match(
    writeFdMatch[1],
    /\bdealloc_raw\b/,
    'stdio fd_write private scratch owners must be explicitly consumed',
);
assert.doesNotMatch(
    writeFdMatch[1],
    /\bstd_free\b/,
    'stdio fd_write must not hide checked dealloc failures behind std_free',
);

for (const helper of [
    'stdio_fd_read_into_result',
    'stdio_finish_read_buffer',
]) {
    assert.doesNotMatch(code, new RegExp(`\\bfn\\s+${helper}\\b`), `${helper} must stay below stdio/read`);
    assert.doesNotMatch(readCode, new RegExp(`\\bfn\\s+${helper}\\b`), `${helper} must stay in stdio/read/buffer`);
    assert.match(readBufferCode, new RegExp(`\\bfn\\s+${helper}\\b`), `${helper} must exist in stdio/read/buffer`);
}

assert.match(
    readCode,
    /#import\s+"std\/stdio\/read\/buffer"\s+as\s+\*/,
    'std/stdio/read must depend on read/buffer boundary helpers',
);

for (const helper of [
    'stdio_read_all_bytes_result',
    'stdio_read_all_text_result',
    'read_all',
    'stdio_read_line_result',
    'read_line',
]) {
    assert.doesNotMatch(code, new RegExp(`\\bfn\\s+(?:noshadow\\s+)?${helper}\\b`), `${helper} must stay in stdio/read`);
    assert.match(readCode, new RegExp(`\\bfn\\s+(?:noshadow\\s+)?${helper}\\b`), `${helper} must exist in stdio/read`);
}

const readAllMatch = readCode.match(
    /fn\s+stdio_read_all_bytes_result\s+<\(\)\*>\s*Result<ByteBuf\s*,\s*StdErrorKind>>\s+\(\):([\s\S]*?)\nfn\s+stdio_read_all_bytes\s+/,
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

const readLineMatch = readCode.match(
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
