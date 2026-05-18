#!/usr/bin/env node

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');

const repoRoot = path.resolve(__dirname, '..');
const rootRelPath = 'stdlib/alloc/io.nepl';
const rootSrc = fs.readFileSync(path.join(repoRoot, rootRelPath), 'utf8');
const relPath = 'stdlib/alloc/io/bytebuf.nepl';
const src = fs.readFileSync(path.join(repoRoot, relPath), 'utf8');
const byteBuilderSources = [
    'stdlib/alloc/io/bytebuilder.nepl',
    'stdlib/alloc/io/bytebuilder/types.nepl',
    'stdlib/alloc/io/bytebuilder/storage.nepl',
    'stdlib/alloc/io/bytebuilder/append.nepl',
    'stdlib/alloc/io/bytebuilder/build.nepl',
].map((relPath) => fs.readFileSync(path.join(repoRoot, relPath), 'utf8'));
const traitsRelPath = 'stdlib/alloc/io/traits.nepl';
const traitsSrc = fs.readFileSync(path.join(repoRoot, traitsRelPath), 'utf8');
const fsRelPath = 'stdlib/std/fs/read/fd.nepl';
const fsSrc = fs.readFileSync(path.join(repoRoot, fsRelPath), 'utf8');
const fsRawRelPath = 'stdlib/std/fs/raw/fd_io.nepl';
const fsRawSrc = fs.readFileSync(path.join(repoRoot, fsRawRelPath), 'utf8');
const bytebufResultRelPath = 'tests/stdlib/bytebuf_result.n.md';
const bytebufResultSrc = fs.readFileSync(path.join(repoRoot, bytebufResultRelPath), 'utf8');

const code = src
    .split(/\r?\n/)
    .filter((line) => !/^\s*\/\//.test(line))
    .join('\n');
const rootCode = rootSrc
    .split(/\r?\n/)
    .filter((line) => !/^\s*\/\//.test(line))
    .join('\n');
const byteBuilderCode = byteBuilderSources.join('\n')
    .split(/\r?\n/)
    .filter((line) => !/^\s*\/\//.test(line))
    .join('\n');
const traitsCode = traitsSrc
    .split(/\r?\n/)
    .filter((line) => !/^\s*\/\//.test(line))
    .join('\n');

function markdownSection(src, heading) {
    const start = src.indexOf(`## ${heading}`);
    assert.notEqual(start, -1, `${heading} doctest section must exist`);
    const rest = src.slice(start);
    const next = rest.slice(1).search(/\n## /);
    return next === -1 ? rest : rest.slice(0, next + 1);
}

const removedOwnerHelperDoctest = markdownSection(bytebufResultSrc, 'io_bytebuf_rejects_raw_memptr_ownership_forging');

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
    /enum\s+ByteBufStorage:\s+Empty\s+Owned\s+<RegionToken<u8>>/,
    'ByteBuf storage state must distinguish empty storage from owned RegionToken payload structurally',
);

assert.match(
    code,
    /struct\s+ByteBuf:\s+storage\s+<ByteBufStorage>\s+len\s+<i32>/,
    'ByteBuf must store byte ownership in ByteBufStorage instead of a loose RegionToken/len pair',
);

assert.doesNotMatch(
    code,
    /\bfn\s+io_bytebuf_empty_region\b/,
    'ByteBuf must not encode empty storage with a zero-size RegionToken sentinel helper',
);

assert.doesNotMatch(
    code,
    /\bregion_new\s+ptr\s+0\b/,
    'ByteBuf empty storage must not forge a zero-size RegionToken sentinel',
);

assert.match(
    code,
    /\bpub\s+fn\s+io_bytebuf_empty\s+<\(\)->ByteBuf>\s+\(\):\s+ByteBuf\s+ByteBufStorage::Empty\s+0\b/,
    'ByteBuf typed empty constructor must use the structural empty storage state',
);

assert.doesNotMatch(
    code,
    /\bpub\s+fn\s+io_bytebuf_empty_region\b/,
    'ByteBuf empty RegionToken sentinel helper must not be public API',
);

assert.match(
    code,
    /\bpub\s+fn\s+io_bytebuf_empty\s+<\(\)->ByteBuf>/,
    'ByteBuf typed empty constructor must remain public',
);

assert.doesNotMatch(
    code,
    /\b(?:pub\s+)?fn\s+io_bytebuf_from_owned_ptr\b/,
    'ByteBuf must not provide a raw MemPtr ownership-forging helper',
);

assert.doesNotMatch(
    removedOwnerHelperDoctest,
    /#import\s+"core\/mem(?:\/internal)?"\s+as\s+\*/,
    'removed io_bytebuf_from_owned_ptr compile-fail doctest must not import raw memory modules',
);

assert.doesNotMatch(
    removedOwnerHelperDoctest,
    /\bmem_ptr_wrap\b/,
    'removed io_bytebuf_from_owned_ptr compile-fail doctest must not forge a MemPtr fixture',
);

assert.match(
    removedOwnerHelperDoctest,
    /\bio_bytebuf_from_owned_ptr\s+0\s+1\b/,
    'removed io_bytebuf_from_owned_ptr compile-fail doctest must prove helper unavailability without raw MemPtr construction',
);

assert.doesNotMatch(
    byteBuilderCode,
    /\b(?:pub\s+)?fn\s+byte_builder_from_owned_ptr\b/,
    'ByteBuilder must not provide a raw MemPtr ownership-forging helper',
);

assert.doesNotMatch(
    code,
    /\bregion_new\s+ptr\s+byte_len\b/,
    'ByteBuf must not wrap caller-provided MemPtr values into RegionToken owners',
);

assert.doesNotMatch(
    byteBuilderCode,
    /\bregion_new\s+ptr\s+cap0\b/,
    'ByteBuilder must not wrap caller-provided MemPtr values into RegionToken owners',
);

assert.match(
    code,
    /\bfn\s+io_bytebuf_data_ptr_ref\s+<\(&ByteBuf\)->MemPtr<u8>>[\s\S]*?\bmatch\s+storage_ref:[\s\S]*?\bByteBufStorage::Empty:[\s\S]*?\bmem_ptr_wrap\s+0[\s\S]*?\bByteBufStorage::Owned\s+region:[\s\S]*?\bregion_ptr\s+region\b/,
    'ByteBuf public raw-byte access must match storage state and borrow owned RegionToken payload only in the Owned branch',
);

assert.match(
    code,
    /\bfn\s+io_bytebuf_finish_region\s+<\(RegionToken<u8>,i32\)->ByteBuf>/,
    'ByteBuf string conversion must finish by moving the RegionToken owner into ByteBuf',
);

const fromStrMatch = code.match(/(?:pub\s+)?fn\s+io_bytebuf_from_str_result\b([\s\S]*?)\n(?:pub\s+)?fn\s+io_bytebuf_from_str\b/);
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
    'ByteBuf construction must not inline null raw pointer ownership',
);

assert.doesNotMatch(
    code,
    /\bResult<ByteBuf,[^>]+>::Ok\s+ByteBuf\s+(?:buf|ptr|exact|data)\b/,
    'ByteBuf Result return paths must not bypass the typed owner finalization boundary',
);

assert.doesNotMatch(
    rootCode,
    /\b(?:mem_ptr_addr|load_u8|store_u8|mem_copy|alloc_raw|dealloc_raw)\b/,
    'alloc/io root facade must not carry direct raw memory evidence',
);

assert.match(
    code,
    /\b(?:mem_ptr_addr|load_u8|store_u8|mem_copy|RegionToken)\b/,
    'alloc/io/bytebuf must carry source-level raw memory boundary evidence',
);

assert.doesNotMatch(
    byteBuilderSources[0],
    /\b(?:mem_ptr_addr|load_u8|store_u8|mem_copy|alloc_raw|dealloc_raw)\b/,
    'alloc/io/bytebuilder facade must not carry direct raw memory evidence',
);

assert.match(byteBuilderCode, /\b(?:mem_ptr_addr|load_u8|store_u8|mem_copy|RegionToken)\b/, 'ByteBuilder implementation modules must carry source-level raw memory evidence');

const fsReadMatch = fsSrc.match(/(?:pub\s+)?fn\s+fs_read_fd_bytes\b([\s\S]*)/);
assert.ok(fsReadMatch, 'fs_read_fd_bytes body must be found');
const fsRead = fsReadMatch[1];
const fsFinishMatch = fsRawSrc.match(/(?:pub\s+)?fn\s+fs_finish_read_buffer\b([\s\S]*)/);
assert.ok(fsFinishMatch, 'fs_finish_read_buffer body must be found');
const fsFinish = fsFinishMatch[1];

assert.match(
    fsRead,
    /\balloc_region<u8>\s+cap[\s\S]*\brealloc_region_bytes_keep<u8>\s+buf_region\s+new_cap[\s\S]*\bfs_finish_read_buffer\s+buf_region\s+read_len\b/,
    'fs_read_fd_bytes must finish through the ByteBuf ownership-normalizing helper',
);

assert.match(
    fsFinish,
    /<\(RegionToken<u8>,i32\)\*>[\s\S]*\beq\s+data_len\s+0[\s\S]*?\bmatch\s+dealloc_region<u8>\s+region:[\s\S]*?\bResult<ByteBuf,i32>::Ok\s+io_bytebuf_empty\b[\s\S]*\brealloc_region_bytes_keep<u8>\s+region\s+data_len[\s\S]*\bio_bytebuf_finish_region\b/,
    'fs_finish_read_buffer must consume private scratch storage through RegionToken cleanup before returning an empty ByteBuf',
);

assert.doesNotMatch(
    fsFinish,
    /\b(?:alloc_ptr|realloc_ptr|dealloc_ptr|dealloc_raw\s+mem_ptr_addr)\b/,
    'fs_finish_read_buffer private scratch cleanup must not recover ownership from low-level MemPtr or raw address views',
);

console.log('alloc/io ByteBuf owner boundary regression passed');
