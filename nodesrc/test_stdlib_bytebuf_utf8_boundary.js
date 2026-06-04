#!/usr/bin/env node

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const { legacyTypeSyntaxView } = require('./source_policy/nepl_source_view');

const repoRoot = path.resolve(__dirname, '..');

function implementation(relPath) {
    return legacyTypeSyntaxView(fs.readFileSync(path.join(repoRoot, relPath), 'utf8'));
}

const ioRootCode = implementation('stdlib/alloc/io.nepl');
const ioCode = implementation('stdlib/alloc/io/bytebuf.nepl');
const fsCode = implementation('stdlib/std/fs.nepl');
const fsErrorCode = implementation('stdlib/std/fs/error.nepl');
const fsBytesCode = implementation('stdlib/std/fs/bytes.nepl');
const fsReadCode = implementation('stdlib/std/fs/read/path.nepl');
const fsPathCode = implementation('stdlib/std/fs/path.nepl');
const fsPathEntryCode = implementation('stdlib/std/fs/path/entry.nepl');
const fsDirReadFdCode = implementation('stdlib/std/fs/dir/read_fd.nepl');

assert.match(ioRootCode, /pub\s+#import\s+"\.\/io\/bytebuf"\s+as\s+\*/, 'alloc/io root must re-export checked ByteBuf APIs');
assert.doesNotMatch(ioRootCode, /fn\s+io_bytebuf_to_str_result\b/, 'alloc/io root must not own ByteBuf UTF-8 conversion');
assert.match(ioCode, /fn\s+io_bytebuf_to_str_result\s+<\(ByteBuf\)->Result<str,\s*StdErrorKind>>\s+\(buf\):[\s\S]*string_utf8_validate_mem\s+data\s+byte_len/, 'io_bytebuf_to_str_result must validate UTF-8 before constructing str');
assert.match(ioCode, /Result::Err\s+_e:[\s\S]*io_bytebuf_free\s+buf[\s\S]*Result::Err\s+StdErrorKind::InvalidUtf8/, 'io_bytebuf_to_str_result must reject invalid UTF-8 as InvalidUtf8 and consume the buffer');
assert.match(ioCode, /Result::Ok\s+_:[\s\S]*string_from_mem_unchecked_result\s+data\s+byte_len/, 'io_bytebuf_to_str_result may only call unchecked construction after validation succeeds');

assert.match(fsCode, /pub\s+#import\s+"\.\/fs\/error"\s+as\s+\*/, 'std/fs root must re-export typed fs errors');
assert.match(fsErrorCode, /pub\s+enum\s+FsErrorKind:[\s\S]*InvalidUtf8[\s\S]*pub\s+struct\s+FsError:/, 'std/fs/error must expose typed error kind and payload');
assert.match(fsBytesCode, /fn\s+fs_bytes_to_string_result\s+<\(ByteBuf\)\*>Result<str,\s*FsError>>\s+\(buf\):[\s\S]*io_bytebuf_to_str_result\s+buf[\s\S]*fs_error_from_std_error\s+FsOperation::BytesToString\s+e/, 'fs_bytes_to_string_result must return FsError from the checked ByteBuf-to-str boundary');
assert.match(fsBytesCode, /fn\s+fs_bytes_to_string_errno_result\s+<\(ByteBuf\)\*>Result<str,i32>>\s+\(buf\):[\s\S]*fs_bytes_to_string_result\s+buf[\s\S]*fs_error_to_errno\s+e/, 'errno compatibility must be an explicit wrapper over typed fs text conversion');
assert.match(fsReadCode, /fn\s+fs_read_to_string\s+<\(str\)\*>Result<str,\s*FsError>>\s+\(path\):[\s\S]*fs_bytes_to_string_result\s+bytes/, 'fs_read_to_string must use checked ByteBuf-to-str conversion and preserve FsError');
assert.doesNotMatch(fsPathCode, /fn\s+fs_string_from_bytes\b/, 'std/fs/path root must not own directory entry UTF-8 conversion');
assert.doesNotMatch(fsPathEntryCode, /\bfs_string_from_bytes\b/, 'std/fs/path/entry must not expose raw directory byte conversion through the safe path facade');
assert.doesNotMatch(fsPathEntryCode, /\b(?:mem_ptr_wrap|mem_ptr_addr|string_utf8_validate_mem|string_from_mem_unchecked_result)\b/, 'std/fs/path/entry must stay out of the raw directory byte conversion boundary');
assert.match(fsDirReadFdCode, /fn\s+fs_dirent_name_to_string\s+<\(MemPtr<u8>,i32\)->Result<str,i32>>\s+\(src_ptr,\s*byte_len\):[\s\S]*string_utf8_validate_mem\s+src_ptr\s+byte_len[\s\S]*string_from_mem_unchecked_result\s+src_ptr\s+byte_len/, 'fd_readdir directory entry byte ranges must be UTF-8 validated before str construction inside the raw fd boundary');
assert.match(fsDirReadFdCode, /let\s+name_ptr\s+<MemPtr<u8>>\s+mem_ptr_add\s+buf_ptr\s+add\s+off\s+fs_dirent_header_size[\s\S]*match\s+fs_dirent_name_to_string\s+name_ptr\s+name_len/, 'fd_readdir entry conversion must derive the name pointer from the RegionToken-backed buffer view');
assert.doesNotMatch(fsDirReadFdCode, /\bmem_ptr_wrap\b/, 'std/fs/dir/read_fd must not rewrap raw directory entry addresses as MemPtr');

console.log('stdlib bytebuf utf8 boundary regression passed');
