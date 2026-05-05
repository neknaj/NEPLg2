#!/usr/bin/env node

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');

const repoRoot = path.resolve(__dirname, '..');
const relPaths = [
    'stdlib/std/fs.nepl',
    'stdlib/std/fs/constants.nepl',
    'stdlib/std/fs/path.nepl',
    'stdlib/std/fs/raw.nepl',
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
assert.doesNotMatch(facadeCode, /\bfn\s+fs_normalize_relative\b/, 'std/fs facade must not inline path normalization helpers');
assert.doesNotMatch(facadeCode, /\bfn\s+fs_path_has_forbidden_host_byte\b/, 'std/fs facade must not inline path validation helpers');
assert.doesNotMatch(facadeCode, /\bfn\s+fs_fd_read_into_result\b/, 'std/fs facade must not inline raw fd read helpers');
assert.doesNotMatch(facadeCode, /\bfn\s+fs_finish_read_buffer\b/, 'std/fs facade must not inline raw read buffer finish helpers');

assert.doesNotMatch(combinedCode, /\bstr_split_result\b/, 'fs_normalize_relative must not use owned Vec<str> split');
assert.doesNotMatch(combinedCode, /\bstr_split_ranges_result\b/, 'fs_normalize_relative must not allocate split range vectors');
assert.match(pathCode, /fn\s+fs_normalize_relative_builder\s+<\(str\)->Result<StringBuilder,i32>>\s+\(path\):[\s\S]*str_split_next\s+path\s+"\/"\s+cursor[\s\S]*match\s+get\s+step\s+"kind":[\s\S]*StrSplitStepKind::Part:/, 'fs_normalize_relative_builder must scan path components with allocation-free split steps');
assert.match(pathCode, /fn\s+fs_normalize_relative_builder\s+<\(str\)->Result<StringBuilder,i32>>\s+\(path\):[\s\S]*match\s+fs_normalize_range_push\s+stack\s+part_start\s+part_end:[\s\S]*Result::Err\s+e:[\s\S]*set\s+stack\s+v::Vec<i32>\s+0\s+0\s+mem_ptr_wrap\s+0[\s\S]*set\s+err\s+e/, 'fs_normalize_relative_builder must store component ranges as Copy i32 pairs and map push failure to errno');
assert.match(pathCode, /fn\s+fs_normalize_relative\s+<\(str\)->Result<str,i32>>\s+\(path\):[\s\S]*fs_normalize_relative_builder\s+path[\s\S]*sb_build_result\s+sb/, 'fs_normalize_relative must delegate through the builder boundary');
assert.match(facadeCode, /fn\s+fs_read_dir_fd\s+<\(i32\)\*>Result<Vec<str>,i32>>\s+\(fd\):[\s\S]*match\s+v::push<str>\s+entries\s+name:[\s\S]*Result::Err\s+_e:[\s\S]*set\s+entries\s+v::Vec<str>\s+0\s+0\s+mem_ptr_wrap\s+0[\s\S]*set\s+err\s+12/, 'fs_read_dir_fd must map entry accumulation push failure to errno 12');

console.log('stdlib fs unsafe unwrap regression passed');
