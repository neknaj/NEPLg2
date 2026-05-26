#!/usr/bin/env node

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const { legacyTypeSyntaxView } = require('./source_policy/nepl_source_view');

const repoRoot = path.resolve(__dirname, '..');
const rootPath = 'stdlib/alloc/collections/fenwick.nepl';
const typesPath = 'stdlib/alloc/collections/fenwick/types.nepl';
const storagePath = 'stdlib/alloc/collections/fenwick/storage.nepl';
const mutationPath = 'stdlib/alloc/collections/fenwick/mutation.nepl';
const queryPath = 'stdlib/alloc/collections/fenwick/query.nepl';
const apiPath = 'stdlib/alloc/collections/fenwick/api.nepl';
const apiDiagnosticPath = 'stdlib/alloc/collections/fenwick/api/diagnostic.nepl';
const apiCreatePath = 'stdlib/alloc/collections/fenwick/api/create.nepl';
const apiObserverPath = 'stdlib/alloc/collections/fenwick/api/observer.nepl';
const apiUpdatePath = 'stdlib/alloc/collections/fenwick/api/update.nepl';
const apiQueryPath = 'stdlib/alloc/collections/fenwick/api/query.nepl';
const apiCleanupPath = 'stdlib/alloc/collections/fenwick/api/cleanup.nepl';

const rootCode = sourceWithoutComments(rootPath);
const typesCode = sourceWithoutComments(typesPath);
const storageCode = sourceWithoutComments(storagePath);
const mutationCode = sourceWithoutComments(mutationPath);
const queryCode = sourceWithoutComments(queryPath);
const apiCode = sourceWithoutComments(apiPath);
const apiDiagnosticCode = sourceWithoutComments(apiDiagnosticPath);
const apiCreateCode = sourceWithoutComments(apiCreatePath);
const apiObserverCode = sourceWithoutComments(apiObserverPath);
const apiUpdateCode = sourceWithoutComments(apiUpdatePath);
const apiQueryCode = sourceWithoutComments(apiQueryPath);
const apiCleanupCode = sourceWithoutComments(apiCleanupPath);
const code = [
    rootCode,
    typesCode,
    storageCode,
    mutationCode,
    queryCode,
    apiCode,
    apiDiagnosticCode,
    apiCreateCode,
    apiObserverCode,
    apiUpdateCode,
    apiQueryCode,
    apiCleanupCode,
].join('\n');

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
assert.doesNotMatch(apiCode, /\bfn\s+/, 'Fenwick api facade must not keep implementation bodies');
for (const submodule of ['diagnostic', 'create', 'observer', 'update', 'query', 'cleanup']) {
    assert.match(
        apiCode,
        new RegExp(`pub\\s+#import\\s+"\\.\\/api\\/${submodule}"\\s+as\\s+@merge`),
        `Fenwick api facade must re-export api/${submodule}`,
    );
}

assert.match(typesCode, /#import\s+"alloc\/collections\/vec"\s+as\s+vec/, 'Fenwick types must use typed Vec storage');
assert.match(typesCode, /struct\s+Fenwick:\s+[\s\S]*\bn\s+<i32>[\s\S]*\bbit\s+<Vec<i32>>/, 'Fenwick must store bit payload as a typed Vec<i32> owner');
assert.match(typesCode, /struct\s+FenwickAddError:\s+[\s\S]*tree\s+<Fenwick>[\s\S]*diag\s+<Diag>/, 'Fenwick.add errors must carry the original owner and diagnostic');
assert.match(typesCode, /fn\s+add_error_diag\s+<\(&FenwickAddError\)->Diag>/, 'Fenwick.add error diagnostics must be readable without moving the owner');
assert.match(typesCode, /fn\s+add_error_tree\s+<\(FenwickAddError\)->Fenwick>/, 'Fenwick.add error owner recovery helper is required');
assert.match(storageCode, /fn\s+fenwick_load_owned\s+<\(&Vec<i32>,i32\)->Option<i32>>[\s\S]*vec::get<i32>\s+bit\s+idx/, 'Fenwick must read tree cells through Vec.get and expose missing cells as Option');
assert.match(storageCode, /fn\s+fenwick_store_owned\s+<\(&Vec<i32>,i32,i32\)\*>unit>[\s\S]*vec::replace<i32>/, 'Fenwick must update tree cells through Vec.replace');
assert.match(storageCode, /fn\s+fenwick_alloc_bit\s+<\(i32\)\*>Result<Vec<i32>,\s*Diag>>[\s\S]*vec::filled<i32>\s+bit_len\s+0/, 'Fenwick.new must allocate initialized typed storage through storage helper');
assert.match(storageCode, /fn\s+fenwick_free_bit\s+<\(Vec<i32>\)->unit>[\s\S]*vec::free<i32>\s+bit/, 'Fenwick storage module must centralize Vec cleanup');
assert.match(mutationCode, /fn\s+fenwick_add_storage\s+<\(&Vec<i32>,i32,i32,i32\)\*>bool>[\s\S]*fenwick_load_owned[\s\S]*fenwick_store_owned/, 'Fenwick mutation module must own add traversal');
assert.match(queryCode, /fn\s+fenwick_sum_prefix_storage\s+<\(&Vec<i32>,i32\)->Option<i32>>[\s\S]*fenwick_load_owned[\s\S]*some\s+acc\s+none/, 'Fenwick query module must own prefix traversal');
assert.match(apiDiagnosticCode, /fn\s+fenwick_diag_len\s+<\(\)\*>Diag>/, 'Fenwick diagnostic module must own invalid length diagnostics');
assert.match(apiDiagnosticCode, /fn\s+fenwick_diag_index\s+<\(\)\*>Diag>/, 'Fenwick diagnostic module must own index diagnostics');
assert.match(apiDiagnosticCode, /fn\s+fenwick_diag_range\s+<\(\)\*>Diag>/, 'Fenwick diagnostic module must own range diagnostics');
assert.match(apiCreateCode, /fn\s+new\s+<\(i32\)\*>Result<Fenwick,\s*Diag>>[\s\S]*lt\s+n\s+0[\s\S]*fenwick_diag_len[\s\S]*fenwick_alloc_bit\s+bit_len/, 'Fenwick.new must reject negative lengths and use storage allocation helper');
assert.match(apiObserverCode, /fn\s+len\s+<\(&Fenwick\)->i32>\s+\(fw\):/, 'Fenwick.len must borrow the owner');
assert.match(apiUpdateCode, /fn\s+fenwick_add_err\s+<\(Fenwick,Diag\)->Result<Fenwick,\s*FenwickAddError>>[\s\S]*FenwickAddError\s+fw\s+d/, 'Fenwick update module must centralize owner-preserving add errors');
assert.match(apiUpdateCode, /fn\s+add\s+<\(Fenwick,i32,i32\)\*>Result<Fenwick,\s*FenwickAddError>>[\s\S]*fenwick_add_storage/, 'Fenwick.add must use mutation helper and return owner-carrying error type');
assert.doesNotMatch(apiUpdateCode, /fn\s+add\s+<\(Fenwick,i32,i32\)\*>Result<Fenwick,\s*Diag>>/, 'Fenwick.add must not lose the owner through Err(Diag)');
assert.match(apiQueryCode, /fn\s+sum_prefix\s+<\(&Fenwick,i32\)\*>Result<i32,\s*Diag>>\s+\(fw,\s*r\):/, 'Fenwick.sum_prefix must borrow the owner');
assert.match(apiQueryCode, /fn\s+sum_range\s+<\(&Fenwick,i32,i32\)\*>Result<i32,\s*Diag>>\s+\(fw,\s*l,\s*r\):/, 'Fenwick.sum_range must borrow the owner');
assert.match(apiCleanupCode, /fn\s+free\s+<\(Fenwick\)\*>unit>\s+\(fw\):[\s\S]*fenwick_free_bit\s+bit/, 'Fenwick.free must close typed Vec<i32> storage through cleanup helper');

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
    return legacyTypeSyntaxView(fs.readFileSync(path.join(repoRoot, file), 'utf8'));
}
