#!/usr/bin/env node

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');

const repoRoot = path.resolve(__dirname, '..');
const relPath = 'stdlib/std/stdio.nepl';
const src = fs.readFileSync(path.join(repoRoot, relPath), 'utf8');
const rawRelPath = 'stdlib/std/stdio/raw.nepl';
const rawSrc = fs.readFileSync(path.join(repoRoot, rawRelPath), 'utf8');

const code = src
    .split(/\r?\n/)
    .filter((line) => !/^\s*\/\//.test(line))
    .join('\n');
const rawCode = rawSrc
    .split(/\r?\n/)
    .filter((line) => !/^\s*\/\//.test(line))
    .join('\n');

assert.match(
    code,
    /pub\s+#import\s+"\.\/stdio\/raw"\s+as\s+\*/,
    'std/stdio facade must re-export raw stdio ABI submodule',
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
    '__linux_syscall_rw',
    'fd_read',
    'fd_write',
]) {
    assert.doesNotMatch(code, new RegExp(`\\bfn\\s+${helper}\\b`), `${helper} must stay in stdio/raw`);
}

for (const helper of [
    'std_alloc',
    'std_free',
    'stdio_fd_read_mem',
]) {
    assert.match(rawCode, new RegExp(`\\bfn\\s+${helper}\\b`), `${helper} must exist in stdio/raw`);
}

for (const helper of [
    'stdio_fd_read_into_result',
    'stdio_finish_read_buffer',
    'stdio_read_line_result',
]) {
    assert.match(code, new RegExp(`\\bfn\\s+${helper}\\b`), `${helper} must exist`);
}

const readAllMatch = code.match(
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

const readLineMatch = code.match(
    /fn\s+noshadow\s+read_line\s+<\(\)\*>\s*str>\s+\(\):([\s\S]*?)\nfn\s+noshadow\s+println\s+/,
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
