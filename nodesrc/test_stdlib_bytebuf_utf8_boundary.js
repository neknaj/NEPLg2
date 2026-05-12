#!/usr/bin/env node

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');

const repoRoot = path.resolve(__dirname, '..');

function implementation(relPath) {
    return fs.readFileSync(path.join(repoRoot, relPath), 'utf8')
        .split(/\r?\n/)
        .filter((line) => !/^\s*\/\//.test(line))
        .join('\n');
}

const ioRootCode = implementation('stdlib/alloc/io.nepl');
const ioCode = implementation('stdlib/alloc/io/bytebuf.nepl');
const fsCode = implementation('stdlib/std/fs.nepl');
const fsBytesCode = implementation('stdlib/std/fs/bytes.nepl');
const fsReadCode = implementation('stdlib/std/fs/read/path.nepl');
const fsPathCode = implementation('stdlib/std/fs/path.nepl');
const fsPathEntryCode = implementation('stdlib/std/fs/path/entry.nepl');

assert.match(ioRootCode, /pub\s+#import\s+"\.\/io\/bytebuf"\s+as\s+\*/, 'alloc/io root must re-export checked ByteBuf APIs');
assert.doesNotMatch(ioRootCode, /fn\s+io_bytebuf_to_str_result\b/, 'alloc/io root must not own ByteBuf UTF-8 conversion');
assert.match(ioCode, /fn\s+io_bytebuf_to_str_result\s+<\(ByteBuf\)\*>Result<str,\s*StdErrorKind>>\s+\(buf\):[\s\S]*string_utf8_validate_mem\s+data\s+byte_len/, 'io_bytebuf_to_str_result must validate UTF-8 before constructing str');
assert.match(ioCode, /Result::Err\s+_e:[\s\S]*io_bytebuf_free\s+buf[\s\S]*Result<str,\s*StdErrorKind>::Err\s+StdErrorKind::InvalidUtf8/, 'io_bytebuf_to_str_result must reject invalid UTF-8 as InvalidUtf8 and consume the buffer');
assert.match(ioCode, /Result::Ok\s+_:[\s\S]*string_from_mem_unchecked_result\s+data\s+byte_len/, 'io_bytebuf_to_str_result may only call unchecked construction after validation succeeds');

assert.match(fsBytesCode, /fn\s+fs_bytes_to_string_result\s+<\(ByteBuf\)\*>Result<str,i32>>\s+\(buf\):[\s\S]*io_bytebuf_to_str_result\s+buf/, 'fs_bytes_to_string_result must use the checked ByteBuf-to-str boundary');
assert.match(fsReadCode, /fn\s+fs_read_to_string\s+<\(str\)\*>Result<str,i32>>\s+\(path\):[\s\S]*fs_bytes_to_string_result\s+bytes/, 'fs_read_to_string must use checked ByteBuf-to-str conversion');
assert.doesNotMatch(fsPathCode, /fn\s+fs_string_from_bytes\b/, 'std/fs/path root must not own directory entry UTF-8 conversion');
assert.match(fsPathEntryCode, /fn\s+fs_string_from_bytes\s+<\(i32,i32\)->Result<str,i32>>\s+\(src,\s*byte_len\):[\s\S]*string_utf8_validate_mem\s+src_ptr\s+byte_len[\s\S]*string_from_mem_unchecked_result\s+src_ptr\s+byte_len/, 'directory entry byte ranges must be UTF-8 validated before str construction');

console.log('stdlib bytebuf utf8 boundary regression passed');
