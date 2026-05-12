#!/usr/bin/env node

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');

const repoRoot = path.resolve(__dirname, '..');
const rootPath = 'stdlib/alloc/collections/fenwick.nepl';
const typesPath = 'stdlib/alloc/collections/fenwick/types.nepl';
const storagePath = 'stdlib/alloc/collections/fenwick/storage.nepl';
const mutationPath = 'stdlib/alloc/collections/fenwick/mutation.nepl';
const queryPath = 'stdlib/alloc/collections/fenwick/query.nepl';
const apiPath = 'stdlib/alloc/collections/fenwick/api.nepl';

const rootCode = sourceWithoutComments(rootPath);
const typesCode = sourceWithoutComments(typesPath);
const storageCode = sourceWithoutComments(storagePath);
const mutationCode = sourceWithoutComments(mutationPath);
const queryCode = sourceWithoutComments(queryPath);
const apiCode = sourceWithoutComments(apiPath);
const code = [rootCode, typesCode, storageCode, mutationCode, queryCode, apiCode].join('\n');

const forbidden = [
    /\bunwrap\b/,
    /\bunwrap_ok\b/,
    /\bunwrap_err\b/,
    /\buwok\b/,
    /\buwerr\b/,
    /#intrinsic\s+"unreachable"/,
];

for (const pattern of forbidden) {
    assert.doesNotMatch(code, pattern, 'Fenwick split modules must not use unsafe unwrap helpers in implementation code');
}

assert.doesNotMatch(rootCode, /\bfn\s+/, 'Fenwick root facade must not keep implementation bodies');
for (const submodule of ['types', 'api']) {
    assert.match(
        rootCode,
        new RegExp(`pub\\s+#import\\s+"\\.\\/fenwick\\/${submodule}"\\s+as\\s+@merge`),
        `Fenwick root facade must re-export fenwick/${submodule}`,
    );
}

assert.match(typesCode, /#import\s+"alloc\/collections\/vec"\s+as\s+vec/, 'Fenwick types must use typed Vec storage');
assert.match(typesCode, /struct\s+Fenwick:\s+[\s\S]*\bn\s+<i32>[\s\S]*\bbit\s+<Vec<i32>>/, 'Fenwick must store bit payload as a typed Vec<i32> owner');
assert.match(typesCode, /struct\s+FenwickAddError:\s+[\s\S]*tree\s+<Fenwick>[\s\S]*diag\s+<Diag>/, 'Fenwick.add errors must carry the original owner and diagnostic');
assert.match(typesCode, /fn\s+add_error_diag\s+<\(&FenwickAddError\)->Diag>/, 'Fenwick.add error diagnostics must be readable without moving the owner');
assert.match(typesCode, /fn\s+add_error_tree\s+<\(FenwickAddError\)->Fenwick>/, 'Fenwick.add error owner recovery helper is required');
assert.match(storageCode, /fn\s+fenwick_load_owned\s+<\(&Vec<i32>,i32\)->Option<i32>>[\s\S]*vec::get<i32>\s+bit\s+idx/, 'Fenwick must read tree cells through Vec.get and expose missing cells as Option');
assert.match(storageCode, /fn\s+fenwick_store_owned\s+<\(&Vec<i32>,i32,i32\)\*>\(\)>[\s\S]*vec::replace<i32>/, 'Fenwick must update tree cells through Vec.replace');
assert.match(storageCode, /fn\s+fenwick_alloc_bit\s+<\(i32\)\*>Result<Vec<i32>,\s*Diag>>[\s\S]*vec::filled<i32>\s+bit_len\s+0/, 'Fenwick.new must allocate initialized typed storage through storage helper');
assert.match(storageCode, /fn\s+fenwick_free_bit\s+<\(Vec<i32>\)->\(\)>[\s\S]*vec::free<i32>\s+bit/, 'Fenwick storage module must centralize Vec cleanup');
assert.match(mutationCode, /fn\s+fenwick_add_storage\s+<\(&Vec<i32>,i32,i32,i32\)\*>bool>[\s\S]*fenwick_load_owned[\s\S]*fenwick_store_owned/, 'Fenwick mutation module must own add traversal');
assert.match(queryCode, /fn\s+fenwick_sum_prefix_storage\s+<\(&Vec<i32>,i32\)->Option<i32>>[\s\S]*fenwick_load_owned[\s\S]*some<i32>\s+acc\s+none<i32>/, 'Fenwick query module must own prefix traversal');
assert.match(apiCode, /fn\s+new\s+<\(i32\)\*>Result<Fenwick,\s*Diag>>[\s\S]*lt\s+n\s+0[\s\S]*fenwick_diag_len[\s\S]*fenwick_alloc_bit\s+bit_len/, 'Fenwick.new must reject negative lengths and use storage allocation helper');
assert.match(apiCode, /fn\s+add\s+<\(Fenwick,i32,i32\)\*>Result<Fenwick,\s*FenwickAddError>>[\s\S]*fenwick_add_storage/, 'Fenwick.add must use mutation helper and return owner-carrying error type');
assert.doesNotMatch(apiCode, /fn\s+add\s+<\(Fenwick,i32,i32\)\*>Result<Fenwick,\s*Diag>>/, 'Fenwick.add must not lose the owner through Err(Diag)');
assert.match(apiCode, /fn\s+free\s+<\(Fenwick\)\*>\(\)>\s+\(fw\):[\s\S]*fenwick_free_bit\s+bit/, 'Fenwick.free must close typed Vec<i32> storage through cleanup helper');

assert.doesNotMatch(code, /\bMemPtr\b/, 'Fenwick must not expose raw MemPtr storage');
assert.doesNotMatch(code, /\bmem_ptr_wrap\b/, 'Fenwick must not use raw pointer arithmetic');
assert.doesNotMatch(code, /\bmem_ptr_addr\b/, 'Fenwick must not recover raw addresses for bit storage');
assert.doesNotMatch(code, /\balloc_ptr\b/, 'Fenwick must not allocate bit storage through raw pointer APIs');
assert.doesNotMatch(code, /\bload_i32\b/, 'Fenwick must not read bit storage through raw i32 loads');
assert.doesNotMatch(code, /\bstore_i32\b/, 'Fenwick must not write bit storage through raw i32 stores');
assert.doesNotMatch(code, /\bdealloc_raw\b/, 'Fenwick must not deallocate bit storage through raw pointer APIs');
assert.doesNotMatch(code, /\bdealloc_ptr\b/, 'Fenwick.free must not unwrap checked deallocation for owned storage');

console.log('fenwick unsafe unwrap regression passed');

function sourceWithoutComments(file) {
    return fs.readFileSync(path.join(repoRoot, file), 'utf8')
        .split(/\r?\n/)
        .filter((line) => !/^\s*\/\//.test(line))
        .join('\n');
}
