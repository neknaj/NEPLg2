#!/usr/bin/env node

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');

const repoRoot = path.resolve(__dirname, '..');
const relPath = 'stdlib/alloc/collections/bitset.nepl';
const typesPath = 'stdlib/alloc/collections/bitset/types.nepl';
const layoutPath = 'stdlib/alloc/collections/bitset/layout.nepl';
const storagePath = 'stdlib/alloc/collections/bitset/storage.nepl';
const mutationPath = 'stdlib/alloc/collections/bitset/mutation.nepl';
const apiPath = 'stdlib/alloc/collections/bitset/api.nepl';
const apiDiagnosticPath = 'stdlib/alloc/collections/bitset/api/diagnostic.nepl';
const apiCreatePath = 'stdlib/alloc/collections/bitset/api/create.nepl';
const apiObserverPath = 'stdlib/alloc/collections/bitset/api/observer.nepl';
const apiUpdatePath = 'stdlib/alloc/collections/bitset/api/update.nepl';
const apiBulkPath = 'stdlib/alloc/collections/bitset/api/bulk.nepl';
const apiCleanupPath = 'stdlib/alloc/collections/bitset/api/cleanup.nepl';

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
    assert.doesNotMatch(code, pattern, 'BitSet split modules must not use unsafe unwrap helpers in implementation code');
}

assert.doesNotMatch(rootCode, /\bfn\s+/, 'BitSet root facade must not keep implementation bodies');
for (const submodule of ['types', 'api']) {
    assert.match(
        rootCode,
        new RegExp(`pub\\s+#import\\s+"\\.\\/bitset\\/${submodule}"\\s+as\\s+@merge`),
        `BitSet root facade must re-export bitset/${submodule}`,
    );
}
assert.doesNotMatch(apiCode, /\bfn\s+/, 'BitSet api facade must not keep implementation bodies');
for (const submodule of ['diagnostic', 'create', 'observer', 'update', 'bulk', 'cleanup']) {
    assert.match(
        apiCode,
        new RegExp(`pub\\s+#import\\s+"\\.\\/api\\/${submodule}"\\s+as\\s+@merge`),
        `BitSet api facade must re-export api/${submodule}`,
    );
}

assert.match(typesCode, /#import\s+"alloc\/collections\/vec"\s+as\s+vec/, 'BitSet types must use typed Vec storage');
assert.match(typesCode, /struct\s+BitSet:\s+[\s\S]*\bnbits\s+<i32>[\s\S]*\bnbytes\s+<i32>[\s\S]*\bbits\s+<Vec<u8>>/, 'BitSet must store bit payload as a typed Vec<u8> owner');
assert.match(typesCode, /struct\s+BitSetUpdateError:\s+[\s\S]*owner\s+<BitSet>[\s\S]*diag\s+<Diag>/, 'BitSet update errors must carry the original owner and diagnostic');
assert.match(typesCode, /fn\s+bitset_update_error_diag\s+<\(&BitSetUpdateError\)->Diag>/, 'BitSet update diagnostics must be readable without moving the owner');
assert.match(typesCode, /fn\s+bitset_update_error_owner\s+<\(BitSetUpdateError\)->BitSet>/, 'BitSet update error owner recovery helper is required');
assert.match(layoutCode, /fn\s+bitset_byte_index\s+<\(i32\)->i32>[\s\S]*div_s\s+bit_idx\s+8/, 'BitSet layout module must own byte index calculation');
assert.match(layoutCode, /fn\s+bitset_mask\s+<\(i32\)->i32>[\s\S]*shl\s+1\s+rem_s\s+bit_idx\s+8/, 'BitSet layout module must own bit mask calculation');
assert.match(layoutCode, /fn\s+bitset_byte_len\s+<\(i32\)->i32>[\s\S]*div_s\s+add\s+nbits\s+7\s+8/, 'BitSet layout module must own byte length rounding');
assert.match(storageCode, /fn\s+bitset_alloc_bits\s+<\(i32,i32\)\*>Result<Vec<u8>,\s*Diag>>[\s\S]*vec::filled<u8>\s+nbytes\s+byte/, 'BitSet must allocate initialized typed byte storage through Vec.filled');
assert.match(storageCode, /fn\s+bitset_byte_at\s+<\(&Vec<u8>,i32\)->Option<i32>>[\s\S]*vec::get<u8>[\s\S]*Option::None:[\s\S]*none<i32>/, 'BitSet must read bit bytes through Vec.get and expose missing bytes as Option');
assert.match(storageCode, /fn\s+bitset_store_byte\s+<\(&Vec<u8>,i32,i32\)\*>\(\)>[\s\S]*vec::replace<u8>/, 'BitSet must update bit bytes through Vec.replace');
assert.match(mutationCode, /fn\s+bitset_write_masked\s+<\(&Vec<u8>,i32,i32,bool\)\*>bool>[\s\S]*bitset_byte_at[\s\S]*bitset_store_byte/, 'BitSet mutation module must centralize byte read-modify-write');
assert.match(apiDiagnosticCode, /fn\s+bitset_invalid_len_diag\s+<\(\)\*>Diag>/, 'BitSet diagnostic module must own invalid length diagnostics');
assert.match(apiDiagnosticCode, /fn\s+bitset_index_diag\s+<\(\)\*>Diag>/, 'BitSet diagnostic module must own index diagnostics');
assert.match(apiCreateCode, /fn\s+new\s+<\(i32\)\*>Result<BitSet,\s*Diag>>[\s\S]*bitset_byte_len\s+nbits[\s\S]*bitset_alloc_bits\s+nbytes\s+0/, 'BitSet.new must use layout and storage helpers');
assert.match(apiObserverCode, /fn\s+len\s+<\(&BitSet\)->i32>\s+\(bs\):/, 'BitSet.len must borrow the owner');
assert.match(apiObserverCode, /fn\s+contains\s+<\(&BitSet,i32\)\*>Result<bool,\s*Diag>>\s+\(bs,\s*idx\):/, 'BitSet.contains must borrow the owner');
assert.match(apiUpdateCode, /fn\s+bitset_update\s+<\(BitSet,i32,bool\)\*>Result<BitSet,\s*BitSetUpdateError>>[\s\S]*bitset_write_masked\s+bits\s+byte_idx\s+mask\s+set_bit/, 'BitSet update module must centralize owner-preserving bit updates');
assert.match(apiUpdateCode, /fn\s+insert\s+<\(BitSet,i32\)\*>Result<BitSet,\s*BitSetUpdateError>>\s+\(bs,\s*idx\):[\s\S]*bitset_update\s+bs\s+idx\s+true/, 'BitSet.insert must use the shared update helper');
assert.match(apiUpdateCode, /fn\s+remove\s+<\(BitSet,i32\)\*>Result<BitSet,\s*BitSetUpdateError>>\s+\(bs,\s*idx\):[\s\S]*bitset_update\s+bs\s+idx\s+false/, 'BitSet.remove must use the shared update helper');
assert.doesNotMatch(apiUpdateCode, /fn\s+(?:insert|remove)\s+<\(BitSet,i32\)\*>Result<BitSet,\s*Diag>>/, 'BitSet mutating APIs must not lose the owner through Err(Diag)');
assert.match(apiUpdateCode, /let\s+e\s+<BitSetUpdateError>\s+BitSetUpdateError\s+bs\s+d[\s\S]*err<BitSet,\s*BitSetUpdateError>\s+e/, 'BitSet mutating Err paths must return the input owner in BitSetUpdateError');
assert.match(apiBulkCode, /fn\s+bitset_fill_value\s+<\(BitSet,i32\)\*>BitSet>[\s\S]*bitset_fill_bytes\s+bits\s+nbytes\s+byte_value/, 'BitSet bulk module must centralize byte fill updates');
assert.match(apiBulkCode, /fn\s+clear\s+<\(BitSet\)\*>BitSet>\s+\(bs\):[\s\S]*bitset_fill_value\s+bs\s+0/, 'BitSet.clear must use the bulk fill helper');
assert.match(apiBulkCode, /fn\s+fill\s+<\(BitSet\)\*>BitSet>\s+\(bs\):[\s\S]*bitset_fill_value\s+bs\s+255/, 'BitSet.fill must use the bulk fill helper');
assert.match(apiCleanupCode, /fn\s+free\s+<\(BitSet\)->\(\)>[\s\S]*field::get\s+bs\s+"bits"[\s\S]*vec::free<u8>\s+bits/, 'BitSet.free must consume and close typed Vec<u8> storage');

assert.doesNotMatch(code, /\bMemPtr\b/, 'BitSet must not expose raw MemPtr storage');
assert.doesNotMatch(code, /\bmem_ptr_wrap\b/, 'BitSet must not use raw pointer arithmetic');
assert.doesNotMatch(code, /\bmem_ptr_addr\b/, 'BitSet must not recover raw addresses for bit storage');
assert.doesNotMatch(code, /\balloc_ptr\b/, 'BitSet must not allocate bit storage through raw pointer APIs');
assert.doesNotMatch(code, /\bload_u8\b/, 'BitSet must not read bit storage through raw byte loads');
assert.doesNotMatch(code, /\bstore_u8\b/, 'BitSet must not write bit storage through raw byte stores');
assert.doesNotMatch(code, /\bdealloc_raw\b/, 'BitSet must not deallocate bit storage through raw pointer APIs');
assert.doesNotMatch(code, /\bdealloc_ptr\b/, 'BitSet.free must not unwrap checked deallocation for owned storage');

console.log('bitset unsafe unwrap regression passed');

function sourceWithoutComments(file) {
    return fs.readFileSync(path.join(repoRoot, file), 'utf8')
        .split(/\r?\n/)
        .filter((line) => !/^\s*\/\//.test(line))
        .join('\n');
}
