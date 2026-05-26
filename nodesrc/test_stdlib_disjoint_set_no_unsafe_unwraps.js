#!/usr/bin/env node

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const { legacyTypeSyntaxView } = require('./source_policy/nepl_source_view');

const repoRoot = path.resolve(__dirname, '..');
const relPath = 'stdlib/alloc/collections/disjoint_set.nepl';
const typesPath = 'stdlib/alloc/collections/disjoint_set/types.nepl';
const storagePath = 'stdlib/alloc/collections/disjoint_set/storage.nepl';
const queryPath = 'stdlib/alloc/collections/disjoint_set/query.nepl';
const apiPath = 'stdlib/alloc/collections/disjoint_set/api.nepl';
const apiDiagnosticPath = 'stdlib/alloc/collections/disjoint_set/api/diagnostic.nepl';
const apiCreatePath = 'stdlib/alloc/collections/disjoint_set/api/create.nepl';
const apiObserverPath = 'stdlib/alloc/collections/disjoint_set/api/observer.nepl';
const apiMutationPath = 'stdlib/alloc/collections/disjoint_set/api/mutation.nepl';
const apiCleanupPath = 'stdlib/alloc/collections/disjoint_set/api/cleanup.nepl';

const rootCode = sourceWithoutComments(relPath);
const typesCode = sourceWithoutComments(typesPath);
const storageCode = sourceWithoutComments(storagePath);
const queryCode = sourceWithoutComments(queryPath);
const apiCode = sourceWithoutComments(apiPath);
const apiDiagnosticCode = sourceWithoutComments(apiDiagnosticPath);
const apiCreateCode = sourceWithoutComments(apiCreatePath);
const apiObserverCode = sourceWithoutComments(apiObserverPath);
const apiMutationCode = sourceWithoutComments(apiMutationPath);
const apiCleanupCode = sourceWithoutComments(apiCleanupPath);
const code = [rootCode, typesCode, storageCode, queryCode, apiCode, apiDiagnosticCode, apiCreateCode, apiObserverCode, apiMutationCode, apiCleanupCode].join('\n');

const forbidden = [
    /\bunwrap\b/,
    /\bunwrap_ok\b/,
    /\bunwrap_err\b/,
    /\buwok\b/,
    /\buwerr\b/,
    /#intrinsic\s+"unreachable"/,
];

for (const pattern of forbidden) {
    assert.doesNotMatch(code, pattern, 'DisjointSet split modules must not use unsafe unwrap helpers in implementation code');
}

assert.doesNotMatch(rootCode, /\bfn\s+/, 'DisjointSet root facade must not keep implementation bodies');
for (const submodule of ['types', 'api']) {
    assert.match(
        rootCode,
        new RegExp(`pub\\s+#import\\s+"\\.\\/disjoint_set\\/${submodule}"\\s+as\\s+@merge`),
        `DisjointSet root facade must re-export disjoint_set/${submodule}`,
    );
}
assert.doesNotMatch(apiCode, /\bfn\s+/, 'DisjointSet api facade must not keep implementation bodies');
for (const submodule of ['diagnostic', 'create', 'observer', 'mutation', 'cleanup']) {
    assert.match(
        apiCode,
        new RegExp(`pub\\s+#import\\s+"\\.\\/api\\/${submodule}"\\s+as\\s+@merge`),
        `DisjointSet api facade must re-export api/${submodule}`,
    );
}

assert.match(typesCode, /#import\s+"alloc\/collections\/vec"\s+as\s+vec/, 'DisjointSet types must use typed Vec storage');
assert.match(typesCode, /struct\s+DisjointSet:\s+[\s\S]*\bn\s+<i32>[\s\S]*\bparent\s+<Vec<i32>>[\s\S]*\bsizes\s+<Vec<i32>>/, 'DisjointSet must store parent and size payloads as typed Vec<i32> owners');
assert.match(typesCode, /struct\s+DisjointSetUpdateError:\s+[\s\S]*owner\s+<DisjointSet>[\s\S]*diag\s+<Diag>/, 'DisjointSet update errors must carry the original owner and diagnostic');
assert.match(typesCode, /fn\s+disjoint_set_update_error_diag\s+<\(&DisjointSetUpdateError\)->Diag>/, 'DisjointSet update diagnostics must be readable without moving the owner');
assert.match(typesCode, /fn\s+disjoint_set_update_error_owner\s+<\(DisjointSetUpdateError\)->DisjointSet>/, 'DisjointSet update error owner recovery helper is required');

assert.match(storageCode, /fn\s+dsu_load_owned\s+<\(&Vec<i32>,i32\)->Option<i32>>[\s\S]*vec::get<i32>[\s\S]*Option::None:[\s\S]*none/, 'DisjointSet must read parent and size cells through Vec.get and expose missing cells as Option');
assert.match(storageCode, /fn\s+dsu_store_owned\s+<\(&Vec<i32>,i32,i32\)\*>unit>[\s\S]*vec::replace<i32>/, 'DisjointSet must update parent and size cells through Vec.replace');
assert.match(queryCode, /fn\s+dsu_root_storage\s+<\(&Vec<i32>,i32\)->Option<i32>>[\s\S]*while[\s\S]*dsu_load_owned[\s\S]*Option::None:[\s\S]*set\s+valid\s+false/, 'DisjointSet query module must own bounded root traversal');
assert.match(queryCode, /fn\s+dsu_size_storage\s+<\(&Vec<i32>,i32\)->Option<i32>>[\s\S]*dsu_load_owned\s+sizes\s+root/, 'DisjointSet query module must own size lookup');

assert.match(apiDiagnosticCode, /fn\s+dsu_diag_len\s+<\(\)\*>Diag>/, 'DisjointSet diagnostic module must own length diagnostic constructor');
assert.match(apiDiagnosticCode, /fn\s+dsu_diag_index\s+<\(\)\*>Diag>/, 'DisjointSet diagnostic module must own index diagnostic constructor');
assert.match(apiCreateCode, /fn\s+new\s+<\(i32\)\*>Result<DisjointSet,\s*Diag>>[\s\S]*lt\s+n\s+0[\s\S]*dsu_diag_len[\s\S]*vec::filled<i32>\s+n\s+0[\s\S]*vec::filled<i32>\s+n\s+1/, 'DisjointSet.new must reject negative lengths and allocate initialized typed storage through Vec.filled');
assert.match(apiObserverCode, /fn\s+len\s+<\(&DisjointSet\)->i32>\s+\(dsu\):/, 'DisjointSet.len must borrow the owner');
assert.match(apiObserverCode, /fn\s+find\s+<\(&DisjointSet,i32\)\*>Result<i32,\s*Diag>>\s+\(dsu,\s*idx\):/, 'DisjointSet.find must borrow the owner');
assert.match(apiObserverCode, /fn\s+same\s+<\(&DisjointSet,i32,i32\)\*>Result<bool,\s*Diag>>\s+\(dsu,\s*a,\s*b\):/, 'DisjointSet.same must borrow the owner');
assert.match(apiObserverCode, /fn\s+size\s+<\(&DisjointSet,i32\)\*>Result<i32,\s*Diag>>\s+\(dsu,\s*idx\):/, 'DisjointSet.size must borrow the owner');
assert.match(apiMutationCode, /fn\s+union\s+<\(DisjointSet,i32,i32\)\*>Result<DisjointSet,\s*DisjointSetUpdateError>>\s+\(dsu,\s*a,\s*b\):/, 'DisjointSet.union must return an owner-carrying error type');
assert.doesNotMatch(apiMutationCode, /fn\s+union\s+<\(DisjointSet,i32,i32\)\*>Result<DisjointSet,\s*Diag>>/, 'DisjointSet.union must not lose the owner through Err(Diag)');
assert.match(apiMutationCode, /let\s+e\s+<DisjointSetUpdateError>\s+DisjointSetUpdateError\s+dsu\s+d[\s\S]*err\s+e/, 'DisjointSet.union Err path must return the input owner in DisjointSetUpdateError');
assert.match(apiCleanupCode, /fn\s+free\s+<\(DisjointSet\)->unit>\s+\(dsu\):[\s\S]*field::get\s+dsu\s+"parent"[\s\S]*field::get\s+dsu\s+"sizes"[\s\S]*vec::free<i32>\s+parent[\s\S]*vec::free<i32>\s+sizes/, 'DisjointSet.free must consume and close typed Vec<i32> storage');
assert.doesNotMatch(apiCleanupCode, /fn\s+free\s+<\(DisjointSet\)->unit>\s+\(dsu\):[\s\S]*field::get_ref\s+&dsu\s+"(?:parent|sizes)"/, 'DisjointSet.free must not borrow-read owned array fields');

assert.doesNotMatch(apiCode, /DisjointSet\s+0\s+mem_ptr_wrap\s+0\s+mem_ptr_wrap\s+0/, 'DisjointSet.new must not encode empty storage with null raw pointers');
assert.doesNotMatch(code, /\bMemPtr\b/, 'DisjointSet must not expose raw MemPtr storage');
assert.doesNotMatch(code, /\bmem_ptr_wrap\b/, 'DisjointSet must not use raw pointer sentinels');
assert.doesNotMatch(code, /\bmem_ptr_addr\b/, 'DisjointSet must not recover raw addresses for parent or size storage');
assert.doesNotMatch(code, /\balloc_ptr\b/, 'DisjointSet must not allocate storage through raw pointer APIs');
assert.doesNotMatch(code, /\bload_i32\b/, 'DisjointSet must not read storage through raw i32 loads');
assert.doesNotMatch(code, /\bstore_i32\b/, 'DisjointSet must not write storage through raw i32 stores');
assert.doesNotMatch(code, /\bdealloc_raw\b/, 'DisjointSet must not deallocate storage through raw pointer APIs');
assert.doesNotMatch(code, /dealloc_ptr/, 'DisjointSet must not use checked deallocation for owned internals');

console.log('disjoint set unsafe unwrap regression passed');

function sourceWithoutComments(file) {
    return legacyTypeSyntaxView(fs.readFileSync(path.join(repoRoot, file), 'utf8'));
}
