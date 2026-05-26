#!/usr/bin/env node

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const { legacyTypeSyntaxView } = require('./source_policy/nepl_source_view');

const repoRoot = path.resolve(__dirname, '..');
const relPath = 'stdlib/alloc/collections/segment_tree.nepl';
const typesPath = 'stdlib/alloc/collections/segment_tree/types.nepl';
const storagePath = 'stdlib/alloc/collections/segment_tree/storage.nepl';
const layoutPath = 'stdlib/alloc/collections/segment_tree/layout.nepl';
const mutationPath = 'stdlib/alloc/collections/segment_tree/mutation.nepl';
const rangePath = 'stdlib/alloc/collections/segment_tree/range.nepl';
const apiPath = 'stdlib/alloc/collections/segment_tree/api.nepl';
const apiDiagnosticPath = 'stdlib/alloc/collections/segment_tree/api/diagnostic.nepl';
const apiCreatePath = 'stdlib/alloc/collections/segment_tree/api/create.nepl';
const apiObserverPath = 'stdlib/alloc/collections/segment_tree/api/observer.nepl';
const apiUpdatePath = 'stdlib/alloc/collections/segment_tree/api/update.nepl';
const apiQueryPath = 'stdlib/alloc/collections/segment_tree/api/query.nepl';
const apiCleanupPath = 'stdlib/alloc/collections/segment_tree/api/cleanup.nepl';

const rootCode = sourceWithoutComments(relPath);
const typesCode = sourceWithoutComments(typesPath);
const storageCode = sourceWithoutComments(storagePath);
const layoutCode = sourceWithoutComments(layoutPath);
const mutationCode = sourceWithoutComments(mutationPath);
const rangeCode = sourceWithoutComments(rangePath);
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
    layoutCode,
    mutationCode,
    rangeCode,
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
    assert.doesNotMatch(code, pattern, 'SegmentTree split modules must not use unsafe unwrap helpers in implementation code');
}

assert.doesNotMatch(rootCode, /\bfn\s+/, 'SegmentTree root facade must not keep implementation bodies');
for (const submodule of ['types', 'api']) {
    assert.match(
        rootCode,
        new RegExp(`pub\\s+#import\\s+"\\.\\/segment_tree\\/${submodule}"\\s+as\\s+@merge`),
        `SegmentTree root facade must re-export segment_tree/${submodule}`,
    );
}
assert.doesNotMatch(apiCode, /\bfn\s+/, 'SegmentTree api facade must not keep implementation bodies');
for (const submodule of ['diagnostic', 'create', 'observer', 'update', 'query', 'cleanup']) {
    assert.match(
        apiCode,
        new RegExp(`pub\\s+#import\\s+"\\.\\/api\\/${submodule}"\\s+as\\s+@merge`),
        `SegmentTree api facade must re-export api/${submodule}`,
    );
}

assert.match(typesCode, /#import\s+"alloc\/collections\/vec"\s+as\s+vec/, 'SegmentTree types must use typed Vec storage');
assert.match(typesCode, /struct\s+SegmentTree:\s+[\s\S]*\bn\s+<i32>[\s\S]*\bbase\s+<i32>[\s\S]*\bdata\s+<Vec<i32>>/, 'SegmentTree must store tree payload as a typed Vec<i32> owner');
assert.match(typesCode, /struct\s+SegmentTreeUpdateError:\s+[\s\S]*owner\s+<SegmentTree>[\s\S]*diag\s+<Diag>/, 'SegmentTree update errors must carry the original owner and diagnostic');
assert.match(typesCode, /fn\s+segment_tree_update_error_diag\s+<\(&SegmentTreeUpdateError\)->Diag>/, 'SegmentTree update diagnostics must be readable without moving the owner');
assert.match(typesCode, /fn\s+segment_tree_update_error_owner\s+<\(SegmentTreeUpdateError\)->SegmentTree>/, 'SegmentTree update error owner recovery helper is required');

assert.match(storageCode, /fn\s+seg_load_owned\s+<\(&Vec<i32>,i32\)->Option<i32>>[\s\S]*vec::get<i32>[\s\S]*Option::None:[\s\S]*none/, 'SegmentTree must read tree cells through Vec.get and expose missing cells as Option');
assert.match(storageCode, /fn\s+seg_store_owned\s+<\(&Vec<i32>,i32,i32\)\*>unit>[\s\S]*vec::replace<i32>/, 'SegmentTree must update tree cells through Vec.replace');
assert.match(storageCode, /fn\s+seg_pair_sum\s+<\(&Vec<i32>,i32,i32\)->Option<i32>>[\s\S]*seg_load_owned[\s\S]*some\s+add\s+lv\s+rv/, 'SegmentTree storage module must own sibling pair sum');
assert.match(layoutCode, /fn\s+seg_next_pow2\s+<\(i32\)->i32>[\s\S]*while\s+lt\s+x\s+n/, 'SegmentTree layout module must own base power-of-two rounding');
assert.match(layoutCode, /fn\s+seg_storage_has_expected_len\s+<\(&Vec<i32>,i32\)->bool>[\s\S]*vec::len<i32>\s+data[\s\S]*seg_expected_cells\s+base/, 'SegmentTree layout module must own storage length invariant checks');
assert.match(mutationCode, /fn\s+seg_rebuild_parents\s+<\(&Vec<i32>,i32\)\*>bool>[\s\S]*seg_pair_sum[\s\S]*seg_store_owned/, 'SegmentTree mutation module must centralize parent rebuild');
assert.match(mutationCode, /fn\s+seg_replace_storage\s+<\(&Vec<i32>,i32,i32,i32\)\*>bool>[\s\S]*seg_store_owned[\s\S]*seg_rebuild_parents/, 'SegmentTree mutation module must own replace storage update');
assert.match(mutationCode, /fn\s+seg_add_storage\s+<\(&Vec<i32>,i32,i32,i32\)\*>bool>[\s\S]*seg_load_owned[\s\S]*seg_store_owned[\s\S]*seg_rebuild_parents/, 'SegmentTree mutation module must own add storage update');
assert.match(rangeCode, /fn\s+seg_sum_range_storage\s+<\(&Vec<i32>,i32,i32,i32\)->Option<i32>>[\s\S]*while\s+and\s+lt\s+left\s+right\s+valid[\s\S]*seg_load_owned/, 'SegmentTree range module must own iterative range traversal');

assert.match(apiDiagnosticCode, /fn\s+seg_diag_len\s+<\(\)\*>Diag>/, 'SegmentTree diagnostic module must own invalid length diagnostics');
assert.match(apiDiagnosticCode, /fn\s+seg_diag_index\s+<\(\)\*>Diag>/, 'SegmentTree diagnostic module must own index diagnostics');
assert.match(apiDiagnosticCode, /fn\s+seg_diag_range\s+<\(\)\*>Diag>/, 'SegmentTree diagnostic module must own range diagnostics');
assert.match(apiCreateCode, /fn\s+new\s+<\(i32\)\*>Result<SegmentTree,\s*Diag>>[\s\S]*lt\s+n\s+0[\s\S]*seg_diag_len[\s\S]*vec::filled<i32>\s+cells\s+0/, 'SegmentTree.new must reject negative lengths and allocate initialized typed storage through Vec.filled');
assert.match(apiObserverCode, /fn\s+len\s+<\(&SegmentTree\)->i32>\s+\(st\):/, 'SegmentTree.len must borrow the owner');
assert.match(apiQueryCode, /fn\s+sum_range\s+<\(&SegmentTree,i32,i32\)\*>Result<i32,\s*Diag>>\s+\(st,\s*l,\s*r\):/, 'SegmentTree.sum_range must borrow the owner');
assert.match(apiUpdateCode, /fn\s+seg_update_err\s+<\(SegmentTree,Diag\)->Result<SegmentTree,\s*SegmentTreeUpdateError>>[\s\S]*SegmentTreeUpdateError\s+st\s+d/, 'SegmentTree update module must centralize owner-preserving errors');
assert.match(apiUpdateCode, /fn\s+replace\s+<\(SegmentTree,i32,i32\)\*>Result<SegmentTree,\s*SegmentTreeUpdateError>>\s+\(st,\s*idx,\s*value\):/, 'SegmentTree.replace must return an owner-carrying error type');
assert.match(apiUpdateCode, /fn\s+add\s+<\(SegmentTree,i32,i32\)\*>Result<SegmentTree,\s*SegmentTreeUpdateError>>\s+\(st,\s*idx,\s*delta\):/, 'SegmentTree.add must return an owner-carrying error type');
assert.doesNotMatch(apiUpdateCode, /fn\s+(?:replace|add)\s+<\(SegmentTree,i32,i32\)\*>Result<SegmentTree,\s*Diag>>/, 'SegmentTree mutating APIs must not lose the owner through Err(Diag)');
assert.match(apiUpdateCode, /seg_update_err\s+st\s+d/, 'SegmentTree mutating Err paths must return the input owner in SegmentTreeUpdateError');
assert.match(apiCleanupCode, /fn\s+free\s+<\(SegmentTree\)->unit>\s+\(st\):[\s\S]*field::get\s+st\s+"data"[\s\S]*vec::free<i32>\s+data/, 'SegmentTree.free must consume and close typed Vec<i32> storage');
assert.doesNotMatch(apiCleanupCode, /fn\s+free\s+<\(SegmentTree\)->unit>\s+\(st\):[\s\S]*field::get_ref\s+&st\s+"data"/, 'SegmentTree.free must not borrow-read the data owner field');

assert.doesNotMatch(code, /\bMemPtr\b/, 'SegmentTree must not expose raw MemPtr storage');
assert.doesNotMatch(code, /\bmem_ptr_wrap\b/, 'SegmentTree must not use raw pointer arithmetic');
assert.doesNotMatch(code, /\bmem_ptr_addr\b/, 'SegmentTree must not recover raw addresses for tree storage');
assert.doesNotMatch(code, /\balloc_ptr\b/, 'SegmentTree must not allocate tree storage through raw pointer APIs');
assert.doesNotMatch(code, /\bload_i32\b/, 'SegmentTree must not read tree storage through raw i32 loads');
assert.doesNotMatch(code, /\bstore_i32\b/, 'SegmentTree must not write tree storage through raw i32 stores');
assert.doesNotMatch(code, /\bdealloc_raw\b/, 'SegmentTree must not deallocate tree storage through raw pointer APIs');
assert.doesNotMatch(code, /dealloc_ptr/, 'SegmentTree must not use checked deallocation for owned internals');

console.log('segment tree unsafe unwrap regression passed');

function sourceWithoutComments(file) {
    return legacyTypeSyntaxView(fs.readFileSync(path.join(repoRoot, file), 'utf8'));
}
