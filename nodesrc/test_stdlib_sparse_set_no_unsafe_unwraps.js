#!/usr/bin/env node

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');

const repoRoot = path.resolve(__dirname, '..');
const rootPath = 'stdlib/alloc/collections/sparse_set.nepl';
const typesPath = 'stdlib/alloc/collections/sparse_set/types.nepl';
const storagePath = 'stdlib/alloc/collections/sparse_set/storage.nepl';
const membershipPath = 'stdlib/alloc/collections/sparse_set/membership.nepl';
const mutationPath = 'stdlib/alloc/collections/sparse_set/mutation.nepl';
const apiPath = 'stdlib/alloc/collections/sparse_set/api.nepl';

const rootCode = sourceWithoutComments(rootPath);
const typesCode = sourceWithoutComments(typesPath);
const storageCode = sourceWithoutComments(storagePath);
const membershipCode = sourceWithoutComments(membershipPath);
const mutationCode = sourceWithoutComments(mutationPath);
const apiCode = sourceWithoutComments(apiPath);
const code = [rootCode, typesCode, storageCode, membershipCode, mutationCode, apiCode].join('\n');

const forbidden = [
    /\bunwrap\b/,
    /\bunwrap_ok\b/,
    /\bunwrap_err\b/,
    /\buwok\b/,
    /\buwerr\b/,
    /#intrinsic\s+"unreachable"/,
];

for (const pattern of forbidden) {
    assert.doesNotMatch(code, pattern, 'SparseSet split modules must not use unsafe unwrap helpers in implementation code');
}

assert.doesNotMatch(rootCode, /\bfn\s+/, 'SparseSet root facade must not keep implementation bodies');
for (const submodule of ['types', 'api']) {
    assert.match(
        rootCode,
        new RegExp(`pub\\s+#import\\s+"\\.\\/sparse_set\\/${submodule}"\\s+as\\s+@merge`),
        `SparseSet root facade must re-export sparse_set/${submodule}`,
    );
}

assert.match(typesCode, /#import\s+"alloc\/collections\/vec"\s+as\s+vec/, 'SparseSet types must use the typed Vec storage module');
assert.match(typesCode, /struct\s+SparseSet:\s+[\s\S]*\bn\s+<i32>[\s\S]*\blen0\s+<i32>[\s\S]*\bdense\s+<Vec<i32>>[\s\S]*\bsparse\s+<Vec<i32>>/, 'SparseSet must store dense/sparse payloads as typed Vec<i32> owners');
assert.match(typesCode, /struct\s+SparseSetUpdateError:\s+[\s\S]*owner\s+<SparseSet>[\s\S]*diag\s+<Diag>/, 'SparseSet update errors must carry the original owner and diagnostic');
assert.match(typesCode, /fn\s+sparse_set_update_error_diag\s+<\(&SparseSetUpdateError\)->Diag>/, 'SparseSet update diagnostics must be readable without moving the owner');
assert.match(typesCode, /fn\s+sparse_set_update_error_owner\s+<\(SparseSetUpdateError\)->SparseSet>/, 'SparseSet update error owner recovery helper is required');
assert.match(storageCode, /fn\s+sparse_set_alloc_array\s+<\(i32\)\*>Result<Vec<i32>,\s*Diag>>[\s\S]*vec::filled<i32>\s+n\s+0/, 'SparseSet must allocate dense/sparse arrays through typed Vec initialization');
assert.match(storageCode, /fn\s+sparse_set_load_owned\s+<\(&Vec<i32>,i32\)->Option<i32>>[\s\S]*vec::get<i32>\s+base\s+idx/, 'SparseSet storage reads must use Vec.get and expose missing cells as Option');
assert.match(storageCode, /fn\s+sparse_set_store_owned\s+<\(&Vec<i32>,i32,i32\)\*>\(\)>[\s\S]*vec::replace<i32>\s+base\s+idx\s+value/, 'SparseSet must update dense/sparse slots through Vec.replace');
assert.match(storageCode, /fn\s+sparse_set_free_arrays\s+<\(Vec<i32>,Vec<i32>\)->\(\)>\s+\(dense,\s*sparse\):[\s\S]*vec::free<i32>\s+dense[\s\S]*vec::free<i32>\s+sparse/, 'SparseSet must centralize Vec owner cleanup for dense/sparse storage');
assert.match(membershipCode, /fn\s+sparse_set_valid_index\s+<\(i32,i32\)->bool>/, 'SparseSet membership module must own index validation');
assert.match(membershipCode, /fn\s+sparse_set_contains_raw\s+<\(i32,&Vec<i32>,&Vec<i32>,i32,i32\)->bool>[\s\S]*match\s+sparse_set_load_owned\s+sparse\s+value[\s\S]*match\s+sparse_set_load_owned\s+dense\s+idx/, 'SparseSet membership must read dense/sparse cells through Option-aware storage helpers');
assert.match(mutationCode, /fn\s+sparse_set_insert_storage\s+<\(i32,&Vec<i32>,&Vec<i32>,i32,i32\)\*>Option<i32>>[\s\S]*sparse_set_store_owned\s+dense\s+len0\s+value[\s\S]*sparse_set_store_owned\s+sparse\s+value\s+len0/, 'SparseSet insert mutation must centralize dense/sparse writes and return the next length');
assert.match(mutationCode, /fn\s+sparse_set_remove_storage\s+<\(i32,&Vec<i32>,&Vec<i32>,i32,i32\)\*>Option<i32>>[\s\S]*sparse_set_store_owned\s+dense\s+idx\s+last_value[\s\S]*sparse_set_store_owned\s+sparse\s+last_value\s+idx/, 'SparseSet remove mutation must centralize swap-with-last writes and return the next length');
assert.match(apiCode, /fn\s+new\s+<\(i32\)\*>Result<SparseSet,\s*Diag>>[\s\S]*sparse_set_alloc_array\s+n[\s\S]*ok<SparseSet,\s*Diag>\s+SparseSet\s+n\s+0\s+dense\s+sparse/, 'SparseSet.new must use storage allocation helpers');
assert.match(apiCode, /fn\s+len\s+<\(&SparseSet\)->i32>\s+\(s\):/, 'SparseSet.len must borrow the owner');
assert.match(apiCode, /fn\s+universe_len\s+<\(&SparseSet\)->i32>\s+\(s\):/, 'SparseSet.universe_len must borrow the owner');
assert.match(apiCode, /fn\s+contains\s+<\(&SparseSet,i32\)\*>Result<bool,\s*Diag>>\s+\(s,\s*value\):/, 'SparseSet.contains must borrow the owner');
assert.match(apiCode, /fn\s+insert\s+<\(SparseSet,i32\)\*>Result<SparseSet,\s*SparseSetUpdateError>>\s+\(s,\s*value\):[\s\S]*sparse_set_insert_storage/, 'SparseSet.insert must return an owner-carrying error type and use mutation helper');
assert.match(apiCode, /fn\s+remove\s+<\(SparseSet,i32\)\*>Result<SparseSet,\s*SparseSetUpdateError>>\s+\(s,\s*value\):[\s\S]*sparse_set_remove_storage/, 'SparseSet.remove must return an owner-carrying error type and use mutation helper');
assert.doesNotMatch(apiCode, /fn\s+(?:insert|remove)\s+<\(SparseSet,i32\)\*>Result<SparseSet,\s*Diag>>/, 'SparseSet mutating APIs must not lose the owner through Err(Diag)');
assert.match(apiCode, /let\s+e\s+<SparseSetUpdateError>\s+SparseSetUpdateError\s+s\s+d[\s\S]*err<SparseSet,\s*SparseSetUpdateError>\s+e/, 'SparseSet mutating Err paths must return the input owner in SparseSetUpdateError');
assert.match(apiCode, /fn\s+free\s+<\(SparseSet\)->\(\)>[\s\S]*sparse_set_free_arrays\s+dense\s+sparse/, 'SparseSet.free must close dense and sparse storage through the cleanup helper');

assert.doesNotMatch(code, /\bhdr\s+<i32>/, 'SparseSet must not store dense/sparse owners behind an opaque raw header');
assert.doesNotMatch(code, /sparse_set_hdr_/, 'SparseSet must not reintroduce raw header helpers');
assert.doesNotMatch(code, /\bMemPtr\b/, 'SparseSet must not expose raw MemPtr storage');
assert.doesNotMatch(code, /\bmem_ptr_wrap\b/, 'SparseSet must not use a null pointer sentinel for empty storage');
assert.doesNotMatch(code, /\bmem_ptr_addr\b/, 'SparseSet must not recover raw addresses for dense/sparse storage');
assert.doesNotMatch(code, /\balloc_ptr\b/, 'SparseSet must not allocate dense/sparse storage through raw pointer APIs');
assert.doesNotMatch(code, /\bload_i32\b/, 'SparseSet must not read dense/sparse storage through raw i32 loads');
assert.doesNotMatch(code, /\bstore_i32\b/, 'SparseSet must not write dense/sparse storage through raw i32 stores');
assert.doesNotMatch(code, /alloc_ptr<u8>\s+16/, 'SparseSet must not allocate a raw header block');
assert.doesNotMatch(code, /dealloc_raw\s+hdr\s+16/, 'SparseSet must not deallocate a raw header block');
assert.doesNotMatch(code, /dealloc_ptr/, 'SparseSet must not use checked pointer deallocation for owned internals');
assert.doesNotMatch(code, /dealloc_raw/, 'SparseSet must not deallocate dense/sparse storage through raw pointer APIs');
assert.doesNotMatch(code, /\bdealloc\b/, 'SparseSet must not use checked deallocation for owned internals');

console.log('sparse set unsafe unwrap regression passed');

function sourceWithoutComments(file) {
    return fs.readFileSync(path.join(repoRoot, file), 'utf8')
        .split(/\r?\n/)
        .filter((line) => !/^\s*\/\//.test(line))
        .join('\n');
}
