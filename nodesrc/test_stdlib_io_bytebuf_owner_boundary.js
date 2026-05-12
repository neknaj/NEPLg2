#!/usr/bin/env node

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');

const repoRoot = path.resolve(__dirname, '..');
const rootRelPath = 'stdlib/alloc/io.nepl';
const rootSrc = fs.readFileSync(path.join(repoRoot, rootRelPath), 'utf8');
const relPath = 'stdlib/alloc/io/bytebuf.nepl';
const src = fs.readFileSync(path.join(repoRoot, relPath), 'utf8');
const byteBuilderRelPath = 'stdlib/alloc/io/bytebuilder.nepl';
const byteBuilderSrc = fs.readFileSync(path.join(repoRoot, byteBuilderRelPath), 'utf8');
const traitsRelPath = 'stdlib/alloc/io/traits.nepl';
const traitsSrc = fs.readFileSync(path.join(repoRoot, traitsRelPath), 'utf8');
const loaderSrc = fs.readFileSync(path.join(repoRoot, 'nepl-core/src/loader.rs'), 'utf8');
const fsRelPath = 'stdlib/std/fs/read/fd.nepl';
const fsSrc = fs.readFileSync(path.join(repoRoot, fsRelPath), 'utf8');
const fsRawRelPath = 'stdlib/std/fs/raw/fd_io.nepl';
const fsRawSrc = fs.readFileSync(path.join(repoRoot, fsRawRelPath), 'utf8');

const code = src
    .split(/\r?\n/)
    .filter((line) => !/^\s*\/\//.test(line))
    .join('\n');
const rootCode = rootSrc
    .split(/\r?\n/)
    .filter((line) => !/^\s*\/\//.test(line))
    .join('\n');
const byteBuilderCode = byteBuilderSrc
    .split(/\r?\n/)
    .filter((line) => !/^\s*\/\//.test(line))
    .join('\n');
const traitsCode = traitsSrc
    .split(/\r?\n/)
    .filter((line) => !/^\s*\/\//.test(line))
    .join('\n');

assert.match(rootCode, /pub\s+#import\s+"\.\/io\/bytebuf"\s+as\s+\*/, 'alloc/io root must re-export ByteBuf APIs');
assert.match(rootCode, /pub\s+#import\s+"\.\/io\/bytebuilder"\s+as\s+\*/, 'alloc/io root must re-export ByteBuilder APIs');
assert.match(rootCode, /pub\s+#import\s+"\.\/io\/traits"\s+as\s+\*/, 'alloc/io root must re-export stream trait APIs');
assert.doesNotMatch(rootCode, /\b(?:struct|trait)\s+/, 'alloc/io root facade must not own type or trait definitions');
assert.doesNotMatch(rootCode, /\bfn\s+/, 'alloc/io root facade must not own implementation function bodies');

assert.match(
    code,
    /\bfn\s+io_bytebuf_alloc_region\s+<\(i32\)->Result<RegionToken<u8>, StdErrorKind>>/,
    'ByteBuf string conversion must allocate through an owning RegionToken boundary',
);

assert.doesNotMatch(
    byteBuilderCode,
    /\bfn\s+io_bytebuf_alloc_region\b/,
    'ByteBuilder module must not own ByteBuf allocation helpers',
);

assert.match(
    byteBuilderCode,
    /#import\s+"alloc\/io\/bytebuf"\s+as\s+\*/,
    'ByteBuilder module must depend on the ByteBuf module instead of the alloc/io facade',
);

assert.match(
    traitsCode,
    /\btrait\s+ByteReader:/,
    'io/traits must own stream trait definitions',
);

assert.doesNotMatch(
    traitsCode,
    /\b(?:alloc_ptr|realloc_ptr|store_u8|load_u8|mem_copy)\b/,
    'io/traits must remain a raw-memory-free stream abstraction module',
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

assert.doesNotMatch(
    loaderSrc,
    /&\["alloc",\s*"io\.nepl"\]/,
    'alloc/io root facade must not retain raw-memory boundary capability',
);

assert.match(
    loaderSrc,
    /&\["alloc",\s*"io",\s*"bytebuf\.nepl"\]/,
    'alloc/io/bytebuf must be an exact raw-memory boundary',
);

assert.match(
    loaderSrc,
    /&\["alloc",\s*"io",\s*"bytebuilder\.nepl"\]/,
    'alloc/io/bytebuilder must be an exact raw-memory boundary',
);

const fsReadMatch = fsSrc.match(/fn\s+fs_read_fd_bytes\b([\s\S]*)/);
assert.ok(fsReadMatch, 'fs_read_fd_bytes body must be found');
const fsRead = fsReadMatch[1];
const fsFinishMatch = fsRawSrc.match(/fn\s+fs_finish_read_buffer\b([\s\S]*)/);
assert.ok(fsFinishMatch, 'fs_finish_read_buffer body must be found');
const fsFinish = fsFinishMatch[1];

assert.match(
    fsRead,
    /\bfs_finish_read_buffer\s+buf\s+cap\s+read_len\b/,
    'fs_read_fd_bytes must finish through the ByteBuf ownership-normalizing helper',
);

assert.match(
    fsFinish,
    /\beq\s+data_len\s+0[\s\S]*?\bdealloc_raw\s+mem_ptr_addr\s+buf\s+cap[\s\S]*?\bResult<ByteBuf,i32>::Ok\s+io_bytebuf_empty\b/,
    'fs_finish_read_buffer must deallocate private scratch storage with exact raw cleanup before returning an empty ByteBuf',
);

assert.doesNotMatch(
    fsFinish,
    /\bdealloc_ptr<u8>\s+buf\s+cap\b/,
    'fs_finish_read_buffer private scratch cleanup must not regress to checked dealloc_ptr',
);

console.log('alloc/io ByteBuf owner boundary regression passed');
