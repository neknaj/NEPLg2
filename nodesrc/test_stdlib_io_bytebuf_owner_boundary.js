#!/usr/bin/env node

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');

const repoRoot = path.resolve(__dirname, '..');
const relPath = 'stdlib/alloc/io.nepl';
const src = fs.readFileSync(path.join(repoRoot, relPath), 'utf8');
const fsRelPath = 'stdlib/std/fs.nepl';
const fsSrc = fs.readFileSync(path.join(repoRoot, fsRelPath), 'utf8');
const fsRawRelPath = 'stdlib/std/fs/raw.nepl';
const fsRawSrc = fs.readFileSync(path.join(repoRoot, fsRawRelPath), 'utf8');

const code = src
    .split(/\r?\n/)
    .filter((line) => !/^\s*\/\//.test(line))
    .join('\n');

assert.match(
    code,
    /\bfn\s+io_bytebuf_alloc_region\s+<\(i32\)->Result<RegionToken<u8>, StdErrorKind>>/,
    'ByteBuf string conversion must allocate through an owning RegionToken boundary',
);

assert.match(
    code,
    /struct\s+ByteBuf:\s+ptr\s+<Option<MemPtr<u8>>>\s+len\s+<i32>/,
    'ByteBuf must represent empty storage as Option::None instead of a null owning pointer',
);

assert.match(
    code,
    /\bfn\s+io_bytebuf_from_owned_ptr\s+<\(MemPtr<u8>,i32\)->ByteBuf>/,
    'ByteBuf owned pointer construction must be centralized in io_bytebuf_from_owned_ptr',
);

assert.match(
    code,
    /\bfn\s+io_bytebuf_region_ptr\s+<\(&RegionToken<u8>\)->MemPtr<u8>>/,
    'ByteBuf string conversion must project copy pointers from RegionToken references',
);

assert.match(
    code,
    /\bfn\s+io_bytebuf_finish_region\s+<\(RegionToken<u8>,i32\)->ByteBuf>/,
    'ByteBuf string conversion must finish by moving the RegionToken owner into ByteBuf',
);

const fromStrMatch = code.match(/fn\s+io_bytebuf_from_str_result\b([\s\S]*?)\nfn\s+io_bytebuf_from_str\b/);
assert.ok(fromStrMatch, 'io_bytebuf_from_str_result body must be found');
const fromStr = fromStrMatch[1];

assert.match(
    fromStr,
    /\bmatch\s+io_bytebuf_alloc_region\s+byte_len\b/,
    'io_bytebuf_from_str_result must allocate through io_bytebuf_alloc_region',
);

assert.match(
    fromStr,
    /\blet\s+out_data\s+<MemPtr<u8>>\s+io_bytebuf_region_ptr\s+&region\b/,
    'io_bytebuf_from_str_result must derive copy destination from a RegionToken reference',
);

assert.match(
    fromStr,
    /\bResult<ByteBuf, StdErrorKind>::Ok\s+io_bytebuf_finish_region\s+region\s+byte_len\b/,
    'io_bytebuf_from_str_result must transfer the output owner into ByteBuf exactly at finish',
);

assert.doesNotMatch(
    fromStr,
    /\blet\s+(?:out_raw|data_raw)\b/,
    'io_bytebuf_from_str_result must not keep raw pointer intermediates for copied buffers',
);

assert.doesNotMatch(
    fromStr,
    /\bmatch\s+alloc_ptr<u8>\s+byte_len\b/,
    'io_bytebuf_from_str_result must not bypass the RegionToken owner boundary',
);

assert.doesNotMatch(
    fromStr,
    /\bmem_ptr_addr\s+out\b/,
    'io_bytebuf_from_str_result must not transfer the output owner into a raw-address local',
);

assert.doesNotMatch(
    code,
    /\bByteBuf\s+mem_ptr_wrap\b/,
    'ByteBuf construction must not encode empty storage as mem_ptr_wrap 0',
);

assert.doesNotMatch(
    code,
    /\bResult<ByteBuf,[^>]+>::Ok\s+ByteBuf\s+(?:buf|ptr|exact|data)\b/,
    'ByteBuf Result return paths must use the centralized owned pointer constructor',
);

const fsReadMatch = fsSrc.match(/fn\s+fs_read_fd_bytes\b([\s\S]*?)\n\/\/: fs_std_error_to_errno\b/);
assert.ok(fsReadMatch, 'fs_read_fd_bytes body must be found');
const fsRead = fsReadMatch[1];
const fsFinishMatch = fsRawSrc.match(/fn\s+fs_finish_read_buffer\b([\s\S]*?)\n\/\/: fs:/);
assert.ok(fsFinishMatch, 'fs_finish_read_buffer body must be found');
const fsFinish = fsFinishMatch[1];

assert.match(
    fsRead,
    /\bfs_finish_read_buffer\s+buf\s+cap\s+read_len\b/,
    'fs_read_fd_bytes must finish through the ByteBuf ownership-normalizing helper',
);

assert.match(
    fsFinish,
    /\beq\s+data_len\s+0[\s\S]*?\bdealloc_ptr<u8>\s+buf\s+cap[\s\S]*?\bResult<ByteBuf,i32>::Ok\s+io_bytebuf_empty\b/,
    'fs_finish_read_buffer must deallocate scratch storage before returning an empty ByteBuf',
);

console.log('alloc/io ByteBuf owner boundary regression passed');
