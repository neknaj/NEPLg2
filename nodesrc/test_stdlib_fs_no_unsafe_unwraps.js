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
    'stdlib/std/fs/raw.nepl',
    'stdlib/std/fs/fd.nepl',
    'stdlib/std/fs/stat.nepl',
    'stdlib/std/fs/dir.nepl',
    'stdlib/std/fs/bytes.nepl',
    'stdlib/std/fs/read.nepl',
    'stdlib/std/fs/write.nepl',
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
const dirCode = codeByPath.get('stdlib/std/fs/dir.nepl');

const forbidden = [
    /\bunwrap\b/,
    /\bunwrap_ok\b/,
    /\bunwrap_err\b/,
    /\buwok\b/,
    /\buwerr\b/,
    /#intrinsic\s+"unreachable"/,
];

for (const pattern of forbidden) {
    for (const [relPath, code] of codeByPath) {
        assert.doesNotMatch(code, pattern, `${relPath} must not use unsafe unwrap helpers in implementation code`);
    }
}

assert.match(facadeCode, /pub\s+#import\s+"\.\/fs\/constants"\s+as\s+\*/, 'std/fs facade must re-export constants submodule');
assert.match(facadeCode, /pub\s+#import\s+"\.\/fs\/path"\s+as\s+\*/, 'std/fs facade must re-export path submodule');
assert.match(facadeCode, /pub\s+#import\s+"\.\/fs\/raw"\s+as\s+\*/, 'std/fs facade must re-export raw syscall submodule');
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

assert.match(pathCode, /pub\s+#import\s+"std\/fs\/path\/entry"\s+as\s+\*/, 'std/fs/path facade must re-export entry helper submodule');
assert.match(pathCode, /pub\s+#import\s+"std\/fs\/path\/normalize"\s+as\s+\*/, 'std/fs/path facade must re-export normalize submodule');
assert.doesNotMatch(pathCode, /^\s*(fn|struct|impl)\s/m, 'std/fs/path root must stay a facade without implementation bodies');

assert.doesNotMatch(combinedCode, /\bstr_split_result\b/, 'fs_normalize_relative must not use owned Vec<str> split');
assert.doesNotMatch(combinedCode, /\bstr_split_ranges_result\b/, 'fs_normalize_relative must not allocate split range vectors');
assert.match(pathNormalizeCode, /fn\s+fs_normalize_relative_builder\s+<\(str\)->Result<StringBuilder,i32>>\s+\(path\):[\s\S]*str_split_next\s+path\s+"\/"\s+cursor[\s\S]*match\s+get\s+step\s+"kind":[\s\S]*StrSplitStepKind::Part:/, 'fs_normalize_relative_builder must scan path components with allocation-free split steps');
assert.match(pathNormalizeCode, /fn\s+fs_normalize_relative_builder\s+<\(str\)->Result<StringBuilder,i32>>\s+\(path\):[\s\S]*match\s+fs_normalize_range_push\s+stack\s+part_start\s+part_end:[\s\S]*Result::Err\s+e:[\s\S]*set\s+stack\s+v::vec_empty<i32>[\s\S]*set\s+err\s+e/, 'fs_normalize_relative_builder must store component ranges as Copy i32 pairs and map push failure to errno');
assert.match(pathNormalizeCode, /fn\s+fs_normalize_relative\s+<\(str\)->Result<str,i32>>\s+\(path\):[\s\S]*fs_normalize_relative_builder\s+path[\s\S]*sb_build_result\s+sb/, 'fs_normalize_relative must delegate through the builder boundary');
assert.match(pathEntryCode, /\bfn\s+fs_str_lt\b[\s\S]*\bstring_byte_at_unchecked\b/, 'directory entry comparison must stay in std/fs/path/entry');
assert.match(pathEntryCode, /\bfn\s+fs_string_from_bytes\b[\s\S]*\bstring_utf8_validate_mem\b[\s\S]*\bstring_from_mem_unchecked_result\b/, 'directory entry byte conversion must validate UTF-8 before constructing str');
assert.match(dirCode, /fn\s+fs_read_dir_fd\s+<\(i32\)\*>Result<Vec<str>,i32>>\s+\(fd\):[\s\S]*match\s+v::push<str>\s+entries\s+name:[\s\S]*Result::Err\s+_e:[\s\S]*set\s+entries\s+v::vec_empty<str>[\s\S]*set\s+err\s+12/, 'fs_read_dir_fd must map entry accumulation push failure to errno 12');

console.log('stdlib fs unsafe unwrap regression passed');
