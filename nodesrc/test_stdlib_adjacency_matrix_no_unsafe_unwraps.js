#!/usr/bin/env node

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const { legacyTypeSyntaxView } = require('./source_policy/nepl_source_view');

const repoRoot = path.resolve(__dirname, '..');
const relPath = 'stdlib/alloc/collections/adjacency_matrix.nepl';
const typesPath = 'stdlib/alloc/collections/adjacency_matrix/types.nepl';
const layoutPath = 'stdlib/alloc/collections/adjacency_matrix/layout.nepl';
const storagePath = 'stdlib/alloc/collections/adjacency_matrix/storage.nepl';
const mutationPath = 'stdlib/alloc/collections/adjacency_matrix/mutation.nepl';
const apiPath = 'stdlib/alloc/collections/adjacency_matrix/api.nepl';
const apiDiagnosticPath = 'stdlib/alloc/collections/adjacency_matrix/api/diagnostic.nepl';
const apiCreatePath = 'stdlib/alloc/collections/adjacency_matrix/api/create.nepl';
const apiObserverPath = 'stdlib/alloc/collections/adjacency_matrix/api/observer.nepl';
const apiUpdatePath = 'stdlib/alloc/collections/adjacency_matrix/api/update.nepl';
const apiBulkPath = 'stdlib/alloc/collections/adjacency_matrix/api/bulk.nepl';
const apiCleanupPath = 'stdlib/alloc/collections/adjacency_matrix/api/cleanup.nepl';

const rootCode = sourceWithoutComments(relPath);
const typesCode = sourceWithoutComments(typesPath);
const layoutCode = sourceWithoutComments(layoutPath);
const storageCode = sourceWithoutComments(storagePath);
const mutationCode = sourceWithoutComments(mutationPath);
const apiCode = sourceWithoutComments(apiPath);
const apiDiagnosticCode = sourceWithoutComments(apiDiagnosticPath);
const apiCreateCode = sourceWithoutComments(apiCreatePath);
const apiObserverCode = sourceWithoutComments(apiObserverPath);
const apiUpdateCode = sourceWithoutComments(apiUpdatePath);
const apiBulkCode = sourceWithoutComments(apiBulkPath);
const apiCleanupCode = sourceWithoutComments(apiCleanupPath);
const code = [
    rootCode,
    typesCode,
    layoutCode,
    storageCode,
    mutationCode,
    apiCode,
    apiDiagnosticCode,
    apiCreateCode,
    apiObserverCode,
    apiUpdateCode,
    apiBulkCode,
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
    assert.doesNotMatch(code, pattern, 'AdjacencyMatrix split modules must not use unsafe unwrap helpers in implementation code');
}

assert.doesNotMatch(rootCode, /\bfn\s+/, 'AdjacencyMatrix root facade must not keep implementation bodies');
for (const submodule of ['types', 'api']) {
    assert.match(
        rootCode,
        new RegExp(`pub\\s+#import\\s+"\\.\\/adjacency_matrix\\/${submodule}"\\s+as\\s+@merge`),
        `AdjacencyMatrix root facade must re-export adjacency_matrix/${submodule}`,
    );
}
assert.doesNotMatch(apiCode, /\bfn\s+/, 'AdjacencyMatrix api facade must not keep implementation bodies');
for (const submodule of ['diagnostic', 'create', 'observer', 'update', 'bulk', 'cleanup']) {
    assert.match(
        apiCode,
        new RegExp(`pub\\s+#import\\s+"\\.\\/api\\/${submodule}"\\s+as\\s+@merge`),
        `AdjacencyMatrix api facade must re-export api/${submodule}`,
    );
}

assert.match(typesCode, /#import\s+"alloc\/collections\/vec"\s+as\s+vec/, 'AdjacencyMatrix types must use typed Vec storage');
assert.match(typesCode, /struct\s+AdjacencyMatrix:\s+[\s\S]*\bnverts\s+<i32>[\s\S]*\bnbytes\s+<i32>[\s\S]*\bbits\s+<Vec<u8>>/, 'AdjacencyMatrix must store matrix bytes as a typed Vec<u8> owner');
assert.match(typesCode, /struct\s+AdjacencyMatrixUpdateError:\s+[\s\S]*owner\s+<AdjacencyMatrix>[\s\S]*diag\s+<Diag>/, 'AdjacencyMatrix update errors must carry the original owner and diagnostic');
assert.match(typesCode, /fn\s+adjacency_matrix_update_error_diag\s+<\(&AdjacencyMatrixUpdateError\)->Diag>/, 'AdjacencyMatrix update diagnostics must be readable without moving the owner');
assert.match(typesCode, /fn\s+adjacency_matrix_update_error_owner\s+<\(AdjacencyMatrixUpdateError\)->AdjacencyMatrix>/, 'AdjacencyMatrix update error owner recovery helper is required');
assert.match(layoutCode, /fn\s+adjacency_matrix_bit_index\s+<\(i32,i32,i32\)->i32>[\s\S]*add\s+mul\s+from\s+nverts\s+to/, 'AdjacencyMatrix layout module must own edge bit index calculation');
assert.match(layoutCode, /fn\s+adjacency_matrix_byte_index\s+<\(i32\)->i32>[\s\S]*div_s\s+bit_idx\s+8/, 'AdjacencyMatrix layout module must own byte index calculation');
assert.match(layoutCode, /fn\s+adjacency_matrix_mask\s+<\(i32\)->i32>[\s\S]*shl\s+1\s+rem_s\s+bit_idx\s+8/, 'AdjacencyMatrix layout module must own bit mask calculation');
assert.match(layoutCode, /fn\s+adjacency_matrix_byte_len\s+<\(i32\)->i32>[\s\S]*div_s\s+add\s+mul\s+nverts\s+nverts\s+7\s+8/, 'AdjacencyMatrix layout module must own byte length rounding');
assert.match(storageCode, /fn\s+adjacency_matrix_alloc_bits\s+<\(i32,i32\)\*>Result<Vec<u8>,\s*Diag>>[\s\S]*vec::filled<u8>\s+nbytes\s+byte/, 'AdjacencyMatrix must allocate initialized byte storage through Vec.filled');
assert.match(storageCode, /fn\s+adjacency_matrix_byte_at\s+<\(&Vec<u8>,i32\)->Option<i32>>[\s\S]*vec::get<u8>[\s\S]*Option::None:[\s\S]*none/, 'AdjacencyMatrix must read matrix bytes through Vec.get and expose missing bytes as Option');
assert.match(storageCode, /fn\s+adjacency_matrix_store_byte\s+<\(&Vec<u8>,i32,i32\)\*>unit>[\s\S]*vec::replace<u8>/, 'AdjacencyMatrix must update matrix bytes through Vec.replace');
assert.match(mutationCode, /fn\s+adjacency_matrix_write_masked\s+<\(&Vec<u8>,i32,i32,bool\)\*>bool>[\s\S]*adjacency_matrix_byte_at[\s\S]*adjacency_matrix_store_byte/, 'AdjacencyMatrix mutation module must centralize byte read-modify-write');
assert.match(apiDiagnosticCode, /fn\s+adjacency_matrix_invalid_len_diag\s+<\(unit\)\*>Diag>/, 'AdjacencyMatrix diagnostic module must own invalid length diagnostics');
assert.match(apiDiagnosticCode, /fn\s+adjacency_matrix_vertex_diag\s+<\(unit\)\*>Diag>/, 'AdjacencyMatrix diagnostic module must own vertex diagnostics');
assert.match(apiCreateCode, /fn\s+new\s+<\(i32\)\*>Result<AdjacencyMatrix,\s*Diag>>[\s\S]*adjacency_matrix_byte_len\s+nverts[\s\S]*adjacency_matrix_alloc_bits\s+nbytes\s+0/, 'AdjacencyMatrix.new must use layout and storage helpers');
assert.match(apiObserverCode, /fn\s+len\s+<\(&AdjacencyMatrix\)->i32>\s+\(g\):/, 'AdjacencyMatrix.len must borrow the owner');
assert.match(apiObserverCode, /fn\s+contains\s+<\(&AdjacencyMatrix,i32,i32\)\*>Result<bool,\s*Diag>>\s+\(g,\s*from,\s*to\):/, 'AdjacencyMatrix.contains must borrow the owner');
assert.match(apiUpdateCode, /fn\s+adjacency_matrix_update\s+<\(AdjacencyMatrix,i32,i32,bool\)\*>Result<AdjacencyMatrix,\s*AdjacencyMatrixUpdateError>>[\s\S]*adjacency_matrix_write_masked\s+bits\s+byte_idx\s+mask\s+set_edge/, 'AdjacencyMatrix update module must centralize owner-preserving edge updates');
assert.match(apiUpdateCode, /fn\s+insert\s+<\(AdjacencyMatrix,i32,i32\)\*>Result<AdjacencyMatrix,\s*AdjacencyMatrixUpdateError>>\s+\(g,\s*from,\s*to\):[\s\S]*adjacency_matrix_update\s+g\s+from\s+to\s+true/, 'AdjacencyMatrix.insert must use the shared update helper');
assert.match(apiUpdateCode, /fn\s+remove\s+<\(AdjacencyMatrix,i32,i32\)\*>Result<AdjacencyMatrix,\s*AdjacencyMatrixUpdateError>>\s+\(g,\s*from,\s*to\):[\s\S]*adjacency_matrix_update\s+g\s+from\s+to\s+false/, 'AdjacencyMatrix.remove must use the shared update helper');
assert.doesNotMatch(apiUpdateCode, /fn\s+(?:insert|remove)\s+<\(AdjacencyMatrix,i32,i32\)\*>Result<AdjacencyMatrix,\s*Diag>>/, 'AdjacencyMatrix mutating APIs must not lose the owner through Err(Diag)');
assert.match(apiUpdateCode, /let\s+e\s+<AdjacencyMatrixUpdateError>\s+AdjacencyMatrixUpdateError\s+g\s+d[\s\S]*err\s+e/, 'AdjacencyMatrix mutating Err paths must return the input owner in AdjacencyMatrixUpdateError');
assert.match(apiBulkCode, /fn\s+adjacency_matrix_fill_value\s+<\(AdjacencyMatrix,i32\)\*>AdjacencyMatrix>[\s\S]*adjacency_matrix_fill_bytes\s+bits\s+nbytes\s+byte_value/, 'AdjacencyMatrix bulk module must centralize byte fill updates');
assert.match(apiBulkCode, /fn\s+clear\s+<\(AdjacencyMatrix\)\*>AdjacencyMatrix>\s+\(g\):[\s\S]*adjacency_matrix_fill_value\s+g\s+0/, 'AdjacencyMatrix.clear must use the bulk fill helper');
assert.match(apiCleanupCode, /fn\s+free\s+<\(AdjacencyMatrix\)->unit>[\s\S]*field::get\s+g\s+"bits"[\s\S]*vec::free<u8>\s+bits/, 'AdjacencyMatrix.free must consume and close typed Vec<u8> storage');

assert.doesNotMatch(code, /\bMemPtr\b/, 'AdjacencyMatrix must not expose raw MemPtr storage');
assert.doesNotMatch(code, /\bmem_ptr_wrap\b/, 'AdjacencyMatrix must not use raw pointer arithmetic');
assert.doesNotMatch(code, /\bmem_ptr_addr\b/, 'AdjacencyMatrix must not recover raw addresses for matrix storage');
assert.doesNotMatch(code, /\balloc_ptr\b/, 'AdjacencyMatrix must not allocate matrix storage through raw pointer APIs');
assert.doesNotMatch(code, /\bload_u8\b/, 'AdjacencyMatrix must not read matrix storage through raw byte loads');
assert.doesNotMatch(code, /\bstore_u8\b/, 'AdjacencyMatrix must not write matrix storage through raw byte stores');
assert.doesNotMatch(code, /\bdealloc_raw\b/, 'AdjacencyMatrix must not deallocate matrix storage through raw pointer APIs');
assert.doesNotMatch(code, /fn\s+free\s+<\(AdjacencyMatrix\)->unit>[\s\S]*dealloc_ptr/, 'AdjacencyMatrix.free must not unwrap checked deallocation for owned storage');

console.log('adjacency matrix unsafe unwrap regression passed');

function sourceWithoutComments(file) {
    return legacyTypeSyntaxView(fs.readFileSync(path.join(repoRoot, file), 'utf8'));
}
