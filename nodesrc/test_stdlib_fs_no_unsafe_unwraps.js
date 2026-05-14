#!/usr/bin/env node

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');

const repoRoot = path.resolve(__dirname, '..');
const relPaths = [
    'stdlib/std/fs.nepl',
    'stdlib/std/fs/constants.nepl',
    'stdlib/std/fs/path.nepl',
    'stdlib/std/fs/path/entry.nepl',
    'stdlib/std/fs/path/normalize.nepl',
    'stdlib/std/fs/path/normalize/build.nepl',
    'stdlib/std/fs/path/normalize/range_stack.nepl',
    'stdlib/std/fs/path/normalize/validate.nepl',
    'stdlib/std/fs/raw.nepl',
    'stdlib/std/fs/raw/wasi.nepl',
    'stdlib/std/fs/raw/fd_io.nepl',
    'stdlib/std/fs/raw/llvm.nepl',
    'stdlib/std/fs/fd.nepl',
    'stdlib/std/fs/stat.nepl',
    'stdlib/std/fs/dir.nepl',
    'stdlib/std/fs/dir/open.nepl',
    'stdlib/std/fs/dir/read_fd.nepl',
    'stdlib/std/fs/dir/path.nepl',
    'stdlib/std/fs/bytes.nepl',
    'stdlib/std/fs/read.nepl',
    'stdlib/std/fs/read/fd.nepl',
    'stdlib/std/fs/read/path.nepl',
    'stdlib/std/fs/write.nepl',
    'stdlib/std/fs/write/fd.nepl',
    'stdlib/std/fs/write/path.nepl',
];

function implementation(relPath) {
    return fs.readFileSync(path.join(repoRoot, relPath), 'utf8')
        .split(/\r?\n/)
        .filter((line) => !/^\s*\/\//.test(line))
        .join('\n');
}

const codeByPath = new Map(relPaths.map((relPath) => [relPath, implementation(relPath)]));
const combinedCode = [...codeByPath.values()].join('\n');
const facadeCode = codeByPath.get('stdlib/std/fs.nepl');
const pathCode = codeByPath.get('stdlib/std/fs/path.nepl');
const pathEntryCode = codeByPath.get('stdlib/std/fs/path/entry.nepl');
const pathNormalizeCode = codeByPath.get('stdlib/std/fs/path/normalize.nepl');
const pathNormalizeBuildCode = codeByPath.get('stdlib/std/fs/path/normalize/build.nepl');
const pathNormalizeRangeStackCode = codeByPath.get('stdlib/std/fs/path/normalize/range_stack.nepl');
const pathNormalizeValidateCode = codeByPath.get('stdlib/std/fs/path/normalize/validate.nepl');
const rawCode = codeByPath.get('stdlib/std/fs/raw.nepl');
const rawFdIoCode = codeByPath.get('stdlib/std/fs/raw/fd_io.nepl');
const rawLlvmCode = codeByPath.get('stdlib/std/fs/raw/llvm.nepl');
const fdCode = codeByPath.get('stdlib/std/fs/fd.nepl');
const statCode = codeByPath.get('stdlib/std/fs/stat.nepl');
const dirCode = codeByPath.get('stdlib/std/fs/dir.nepl');
const dirOpenCode = codeByPath.get('stdlib/std/fs/dir/open.nepl');
const dirReadFdCode = codeByPath.get('stdlib/std/fs/dir/read_fd.nepl');
const dirPathCode = codeByPath.get('stdlib/std/fs/dir/path.nepl');
const readCode = codeByPath.get('stdlib/std/fs/read.nepl');
const readFdCode = codeByPath.get('stdlib/std/fs/read/fd.nepl');
const readPathCode = codeByPath.get('stdlib/std/fs/read/path.nepl');
const writeCode = codeByPath.get('stdlib/std/fs/write.nepl');
const writeFdCode = codeByPath.get('stdlib/std/fs/write/fd.nepl');
const writePathCode = codeByPath.get('stdlib/std/fs/write/path.nepl');

const forbidden = [
    /\bunwrap\b/,
    /\bunwrap_ok\b/,
    /\bunwrap_err\b/,
    /\buwok\b/,
    /\buwerr\b/,
];

for (const pattern of forbidden) {
    for (const [relPath, code] of codeByPath) {
        assert.doesNotMatch(code, pattern, `${relPath} must not use unsafe unwrap helpers in implementation code`);
    }
}

for (const [relPath, code] of codeByPath) {
    const lines = code.split(/\r?\n/);
    for (let i = 0; i < lines.length; i += 1) {
        if (!/#intrinsic\s+"unreachable"/.test(lines[i])) {
            continue;
        }
        const window = lines.slice(Math.max(0, i - 5), i + 1).join('\n');
        assert.match(
            window,
            /\bmatch\s+dealloc_ptr<[^>]+>\s+[^\n]+:[\s\S]*\bResult::Err\s+_:/,
            `${relPath}:${i + 1} may only use unreachable as a typed dealloc_ptr owner-cleanup invariant`,
        );
    }
}

assert.match(facadeCode, /pub\s+#import\s+"\.\/fs\/constants"\s+as\s+\*/, 'std/fs facade must re-export constants submodule');
assert.match(facadeCode, /pub\s+#import\s+"\.\/fs\/path"\s+as\s+\*/, 'std/fs facade must re-export path submodule');
assert.doesNotMatch(facadeCode, /pub\s+#import\s+"\.\/fs\/raw"\s+as\s+\*/, 'std/fs safe facade must not re-export raw syscall submodule');
assert.match(facadeCode, /pub\s+#import\s+"\.\/fs\/fd"\s+as\s+\*/, 'std/fs facade must re-export fd submodule');
assert.match(facadeCode, /pub\s+#import\s+"\.\/fs\/stat"\s+as\s+\*/, 'std/fs facade must re-export stat submodule');
assert.match(facadeCode, /pub\s+#import\s+"\.\/fs\/dir"\s+as\s+\*/, 'std/fs facade must re-export dir submodule');
assert.match(facadeCode, /pub\s+#import\s+"\.\/fs\/bytes"\s+as\s+\*/, 'std/fs facade must re-export bytes submodule');
assert.match(facadeCode, /pub\s+#import\s+"\.\/fs\/read"\s+as\s+\*/, 'std/fs facade must re-export read submodule');
assert.match(facadeCode, /pub\s+#import\s+"\.\/fs\/write"\s+as\s+\*/, 'std/fs facade must re-export write submodule');
assert.doesNotMatch(facadeCode, /\bfn\s+fs_normalize_relative\b/, 'std/fs facade must not inline path normalization helpers');
assert.doesNotMatch(facadeCode, /\bfn\s+fs_path_has_forbidden_host_byte\b/, 'std/fs facade must not inline path validation helpers');
assert.doesNotMatch(facadeCode, /\bfn\s+fs_fd_read_into_result\b/, 'std/fs facade must not inline raw fd read helpers');
assert.doesNotMatch(facadeCode, /\bfn\s+fs_finish_read_buffer\b/, 'std/fs facade must not inline raw read buffer finish helpers');
assert.doesNotMatch(facadeCode, /\bfn\s+fs_open_with_flags\b/, 'std/fs facade must not inline fd open helpers');
assert.doesNotMatch(facadeCode, /\bfn\s+fs_close\b/, 'std/fs facade must not inline fd close helpers');
assert.doesNotMatch(facadeCode, /\bfn\s+fs_path_filetype\b/, 'std/fs facade must not inline stat helpers');
assert.doesNotMatch(facadeCode, /\bfn\s+fs_exists\b/, 'std/fs facade must not inline existence helpers');
assert.doesNotMatch(facadeCode, /\bfn\s+fs_read_dir_fd\b/, 'std/fs facade must not inline directory listing helpers');
assert.doesNotMatch(facadeCode, /\bfn\s+fs_open_dir\b/, 'std/fs facade must not inline directory open helpers');
assert.doesNotMatch(facadeCode, /\bfn\s+fs_std_error_to_errno\b/, 'std/fs facade must not inline fs errno conversion helpers');
assert.doesNotMatch(facadeCode, /\bfn\s+fs_bytes_to_string_result\b/, 'std/fs facade must not inline ByteBuf conversion helpers');
assert.doesNotMatch(facadeCode, /\bfn\s+fs_read_fd_bytes\b/, 'std/fs facade must not inline fd read helpers');
assert.doesNotMatch(facadeCode, /\bfn\s+fs_read_to_bytes\b/, 'std/fs facade must not inline path read helpers');
assert.doesNotMatch(facadeCode, /\bfn\s+fs_write_fd_bytes\b/, 'std/fs facade must not inline fd write helpers');
assert.doesNotMatch(facadeCode, /\bfn\s+fs_write_to_bytes\b/, 'std/fs facade must not inline path write helpers');
for (const helper of ['wasi_path_open', 'wasi_path_filestat_get', 'wasi_fd_read', 'wasi_fd_write', 'wasi_fd_readdir', '__linux_syscall_openat_path', '__linux_syscall_rw', 'fs_fd_read_into_result', 'fs_fd_write_from_result']) {
    assert.doesNotMatch(facadeCode, new RegExp(`\\b${helper}\\b`), `std/fs safe facade must not expose raw helper ${helper}`);
}
for (const [relPath, code] of [
    ['stdlib/std/fs/fd.nepl', fdCode],
    ['stdlib/std/fs/stat.nepl', statCode],
    ['stdlib/std/fs/dir/read_fd.nepl', dirReadFdCode],
    ['stdlib/std/fs/read/fd.nepl', readFdCode],
    ['stdlib/std/fs/write/fd.nepl', writeFdCode],
]) {
    assert.match(code, /#import\s+"std\/fs\/raw"\s+as\s+\*/, `${relPath} must import std/fs/raw explicitly when crossing the raw ABI boundary`);
}

assert.match(readCode, /pub\s+#import\s+"std\/fs\/read\/fd"\s+as\s+\*/, 'std/fs/read facade must re-export fd read helper submodule');
assert.match(readCode, /pub\s+#import\s+"std\/fs\/read\/path"\s+as\s+\*/, 'std/fs/read facade must re-export path read helper submodule');
assert.doesNotMatch(readCode, /^\s*(fn|struct|impl)\s/m, 'std/fs/read root must stay a facade without implementation bodies');
assert.match(readFdCode, /\bfn\s+fs_read_fd_bytes\b[\s\S]*\bfs_fd_read_into_result\b[\s\S]*\bfs_finish_read_buffer\s+buf\s+cap\s+read_len\b/, 'fd read loop must stay in std/fs/read/fd');
assert.match(readPathCode, /\bfn\s+fs_read_to_bytes\b[\s\S]*\bfs_open_read\s+path[\s\S]*\bfs_read_fd_bytes\s+fd[\s\S]*\bfs_close\s+fd/, 'path read API must stay in std/fs/read/path');
assert.match(readPathCode, /\bfn\s+fs_read_to_string\b[\s\S]*\bfs_bytes_to_string_result\s+bytes/, 'path text read API must use checked ByteBuf conversion in std/fs/read/path');
assert.doesNotMatch(readPathCode, /\b(?:alloc_ptr|realloc_ptr|dealloc_raw|fs_fd_read_into_result)\b/, 'std/fs/read/path must not own fd scratch raw read loop');
assert.match(writeCode, /pub\s+#import\s+"std\/fs\/write\/fd"\s+as\s+\*/, 'std/fs/write facade must re-export fd write helper submodule');
assert.match(writeCode, /pub\s+#import\s+"std\/fs\/write\/path"\s+as\s+\*/, 'std/fs/write facade must re-export path write helper submodule');
assert.doesNotMatch(writeCode, /^\s*(fn|struct|impl)\s/m, 'std/fs/write root must stay a facade without implementation bodies');
assert.match(writeFdCode, /\bfn\s+fs_write_fd_mem_result\b[\s\S]*\bfs_fd_write_from_result\b[\s\S]*\bdealloc_ptr<u8>\s+nwritten_buf\s+4[\s\S]*\bdealloc_ptr<u8>\s+iov_buf\s+8/, 'fd write loop must stay in std/fs/write/fd');
assert.match(writeFdCode, /\bfn\s+fs_write_fd_bytes\b[\s\S]*\bfs_write_fd_mem_result\s+fd\s+data\s+data_len[\s\S]*\bio_bytebuf_free\s+bytes/, 'ByteBuf-consuming fd write API must stay in std/fs/write/fd and close storage through the ByteBuf owner boundary');
assert.match(writePathCode, /\bfn\s+fs_write_to_bytes\b[\s\S]*\bfs_open_write\s+path[\s\S]*\bfs_write_fd_bytes\s+fd\s+bytes[\s\S]*\bfs_close\s+fd/, 'path write API must stay in std/fs/write/path');
assert.match(writePathCode, /\bfn\s+fs_write_to_string\b[\s\S]*\bio_bytebuf_from_str_result\s+text[\s\S]*\bfs_write_to_bytes\s+path\s+bytes/, 'string write API must build ByteBuf then delegate in std/fs/write/path');
assert.doesNotMatch(writePathCode, /\b(?:alloc_ptr|realloc_ptr|fs_fd_write_from_result)\b/, 'std/fs/write/path must not own fd scratch raw write loop');

assert.match(pathCode, /pub\s+#import\s+"std\/fs\/path\/entry"\s+as\s+\*/, 'std/fs/path facade must re-export entry helper submodule');
assert.match(pathCode, /pub\s+#import\s+"std\/fs\/path\/normalize"\s+as\s+\*/, 'std/fs/path facade must re-export normalize submodule');
assert.doesNotMatch(pathCode, /^\s*(fn|struct|impl)\s/m, 'std/fs/path root must stay a facade without implementation bodies');

assert.doesNotMatch(combinedCode, /\bstr_split_result\b/, 'fs_normalize_relative must not use owned Vec<str> split');
assert.doesNotMatch(combinedCode, /\bstr_split_ranges_result\b/, 'fs_normalize_relative must not allocate split range vectors');
assert.match(pathNormalizeCode, /fn\s+fs_normalize_relative_builder\s+<\(str\)->Result<StringBuilder,i32>>\s+\(path\):[\s\S]*str_split_next\s+path\s+"\/"\s+cursor[\s\S]*match\s+get\s+step\s+"kind":[\s\S]*StrSplitStepKind::Part:/, 'fs_normalize_relative_builder must scan path components with allocation-free split steps');
assert.match(pathNormalizeCode, /fn\s+fs_normalize_relative_builder\s+<\(str\)->Result<StringBuilder,i32>>\s+\(path\):[\s\S]*match\s+normalize_range_stack::fs_normalize_range_push\s+stack\s+part_start\s+part_end:[\s\S]*Result::Err\s+e:[\s\S]*set\s+stack\s+v::vec_empty<i32>[\s\S]*set\s+err\s+e/, 'fs_normalize_relative_builder must store component ranges as Copy i32 pairs and map push failure to errno');
assert.match(pathNormalizeCode, /normalize_validate::fs_path_has_forbidden_host_byte\s+path/, 'fs_normalize_relative_builder must delegate host-path byte validation');
assert.match(pathNormalizeCode, /normalize_build::fs_normalize_build_ranges_builder\s+path\s+&stack/, 'fs_normalize_relative_builder must delegate range-stack output construction');
assert.doesNotMatch(pathNormalizeCode, /\bfn\s+fs_path_has_forbidden_host_byte\b/, 'std/fs/path/normalize root must not inline host-path byte validation');
assert.doesNotMatch(pathNormalizeCode, /\bfn\s+fs_normalize_range_push\b/, 'std/fs/path/normalize root must not inline range-stack mutation');
assert.doesNotMatch(pathNormalizeCode, /\bfn\s+fs_normalize_build_ranges_builder\b/, 'std/fs/path/normalize root must not inline range-stack output construction');
assert.match(pathNormalizeValidateCode, /\bfn\s+fs_path_has_forbidden_host_byte\b[\s\S]*\bstring_byte_at_unchecked\b/, 'host-path byte validation must stay in normalize/validate');
assert.match(pathNormalizeRangeStackCode, /\bfn\s+fs_normalize_range_push\b[\s\S]*\bv::push<i32>\s+stack\s+start[\s\S]*\bv::push<i32>\s+with_start\s+end/, 'range-stack push must stay in normalize/range_stack');
assert.match(pathNormalizeRangeStackCode, /\bfn\s+fs_normalize_range_pop\b[\s\S]*\bv::pop<i32>\s+stack[\s\S]*\bfield::get\s+popped_end\s+"vec"[\s\S]*\bv::pop<i32>\s+without_end[\s\S]*\bfield::get\s+popped_start\s+"vec"/, 'range-stack pop must stay in normalize/range_stack');
assert.match(pathNormalizeBuildCode, /\bfn\s+fs_normalize_build_ranges_builder\b[\s\S]*\bsb_append_slice_result\s+with_sep\s+path\s+part_start\s+part_end/, 'range-stack output construction must stay in normalize/build');
assert.match(pathNormalizeCode, /fn\s+fs_normalize_relative\s+<\(str\)->Result<str,i32>>\s+\(path\):[\s\S]*fs_normalize_relative_builder\s+path[\s\S]*sb_build_result\s+sb/, 'fs_normalize_relative must delegate through the builder boundary');
assert.match(pathEntryCode, /\bfn\s+fs_str_lt\b[\s\S]*\bstring_byte_at_unchecked\b/, 'directory entry comparison must stay in std/fs/path/entry');
assert.match(pathEntryCode, /\bfn\s+fs_sort_strings\s+<\(&Vec<str>\)\*>Result<\(\),i32>>\s+\(entries\):[\s\S]*\bv::get<str>\s+entries\s+i[\s\S]*\bv::replace<str>\s+entries\s+j\s+key/, 'directory entry sort must use Vec public get/replace boundary with an explicit mutation effect');
assert.doesNotMatch(pathEntryCode, /\bfn\s+fs_sort_strings\s+<\(i32,i32\)/, 'directory entry sort must not accept raw Vec storage pointers');
assert.doesNotMatch(pathEntryCode, /\b(?:load<str>|store<str>|mem_ptr_addr)\b/, 'directory entry helpers must not sort Vec<str> through raw str storage');
assert.match(pathEntryCode, /\bfn\s+fs_string_from_bytes\b[\s\S]*\bstring_utf8_validate_mem\b[\s\S]*\bstring_from_mem_unchecked_result\b/, 'directory entry byte conversion must validate UTF-8 before constructing str');
assert.match(dirCode, /pub\s+#import\s+"std\/fs\/dir\/open"\s+as\s+\*/, 'std/fs/dir facade must re-export directory open helper submodule');
assert.match(dirCode, /pub\s+#import\s+"std\/fs\/dir\/read_fd"\s+as\s+\*/, 'std/fs/dir facade must re-export fd directory reader submodule');
assert.match(dirCode, /pub\s+#import\s+"std\/fs\/dir\/path"\s+as\s+\*/, 'std/fs/dir facade must re-export path directory listing submodule');
assert.doesNotMatch(dirCode, /^\s*(fn|struct|impl)\s/m, 'std/fs/dir root must stay a facade without implementation bodies');
assert.match(dirOpenCode, /\bfn\s+fs_open_dir\b[\s\S]*\bfs_normalize_relative\s+path[\s\S]*\bfs_open_with_flags\s+normalized\s+fs_oflags_directory\s+fs_right_fd_readdir/, 'directory open helper must stay in std/fs/dir/open');
assert.match(dirReadFdCode, /fn\s+fs_read_dir_fd\s+<\(i32\)\*>Result<Vec<str>,i32>>\s+\(fd\):[\s\S]*\bwasi_fd_readdir\b[\s\S]*match\s+v::push<str>\s+entries\s+name:[\s\S]*Result::Err\s+e:[\s\S]*set\s+entries\s+v::vec_push_error_vec<str>\s+e[\s\S]*set\s+err\s+12/, 'fs_read_dir_fd must preserve the entry Vec owner while mapping accumulation push failure to errno 12');
assert.match(dirReadFdCode, /\bmatch\s+fs_sort_strings\s+&entries:[\s\S]*Result::Err\s+e:[\s\S]*v::free<str>\s+entries[\s\S]*Result<Vec<str>,i32>::Err\s+e/, 'fs_read_dir_fd must sort through the Vec boundary and free entries if sorting reports an invariant error');
assert.doesNotMatch(dirReadFdCode, /\bget\s+entries\s+"data"/, 'fs_read_dir_fd must not depend on the removed Vec.data field');
assert.match(dirPathCode, /\bfn\s+fs_read_dir\b[\s\S]*\bfs_open_dir\s+path[\s\S]*\bfs_read_dir_fd\s+fd[\s\S]*\bfs_close\s+fd/, 'path directory listing API must stay in std/fs/dir/path');
assert.doesNotMatch(dirPathCode, /\b(?:alloc_ptr|wasi_fd_readdir|load_i32|store_i32)\b/, 'std/fs/dir/path must not own fd_readdir raw entry conversion');
assert.match(rawCode, /pub\s+#import\s+"std\/fs\/raw\/wasi"\s+as\s+\*/, 'std/fs/raw facade must re-export WASI syscall submodule');
assert.match(rawCode, /pub\s+#import\s+"std\/fs\/raw\/fd_io"\s+as\s+\*/, 'std/fs/raw facade must re-export fd I/O scratch submodule');
assert.match(rawCode, /pub\s+#import\s+"std\/fs\/raw\/llvm"\s+as\s+\*/, 'std/fs/raw facade must re-export LLVM fallback submodule');
assert.doesNotMatch(rawCode, /^\s*(#extern|fn|struct|impl)\s/m, 'std/fs/raw root must stay a facade without syscall or helper bodies');
assert.match(rawFdIoCode, /\bfn\s+fs_fd_read_into_result\b[\s\S]*\bstore_i32\s+iov_raw\s+data_raw[\s\S]*\bwasi_fd_read\s+fd\s+iov_raw\s+1\s+nread_raw[\s\S]*\bload_i32\s+nread_raw/, 'fd read scratch initialization must stay in std/fs/raw/fd_io');
assert.match(rawFdIoCode, /\bfn\s+fs_finish_read_buffer\b[\s\S]*\bio_bytebuf_from_owned_ptr\b/, 'ByteBuf finish ownership normalization must stay in std/fs/raw/fd_io');
assert.match(rawLlvmCode, /\bfn\s+__fs_copy_to_cstr\b[\s\S]*\bfn\s+wasi_path_open\b/, 'LLVM filesystem fallback must stay in std/fs/raw/llvm');

console.log('stdlib fs unsafe unwrap regression passed');
