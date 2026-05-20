#!/usr/bin/env node

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');

const repoRoot = path.resolve(__dirname, '..');
const rootPath = 'stdlib/alloc/collections/bloom_filter.nepl';
const typesPath = 'stdlib/alloc/collections/bloom_filter/types.nepl';
const hashPath = 'stdlib/alloc/collections/bloom_filter/hash.nepl';
const layoutPath = 'stdlib/alloc/collections/bloom_filter/layout.nepl';
const storagePath = 'stdlib/alloc/collections/bloom_filter/storage.nepl';
const mutationPath = 'stdlib/alloc/collections/bloom_filter/mutation.nepl';
const apiPath = 'stdlib/alloc/collections/bloom_filter/api.nepl';

const rootCode = sourceWithoutComments(rootPath);
const typesCode = sourceWithoutComments(typesPath);
const hashCode = sourceWithoutComments(hashPath);
const layoutCode = sourceWithoutComments(layoutPath);
const storageCode = sourceWithoutComments(storagePath);
const mutationCode = sourceWithoutComments(mutationPath);
const apiCode = sourceWithoutComments(apiPath);
const code = [rootCode, typesCode, hashCode, layoutCode, storageCode, mutationCode, apiCode].join('\n');

const forbidden = [
    /\bunwrap\b/,
    /\bunwrap_ok\b/,
    /\bunwrap_err\b/,
    /\buwok\b/,
    /\buwerr\b/,
    /#intrinsic\s+"unreachable"/,
];

for (const pattern of forbidden) {
    assert.doesNotMatch(code, pattern, 'BloomFilter split modules must not use unsafe unwrap helpers in implementation code');
}

assert.doesNotMatch(rootCode, /\bfn\s+/, 'BloomFilter root facade must not keep implementation bodies');
for (const submodule of ['types', 'api']) {
    assert.match(
        rootCode,
        new RegExp(`pub\\s+#import\\s+"\\.\\/bloom_filter\\/${submodule}"\\s+as\\s+@merge`),
        `BloomFilter root facade must re-export bloom_filter/${submodule}`,
    );
}

assert.match(typesCode, /#import\s+"alloc\/collections\/vec"\s+as\s+vec/, 'BloomFilter types must use typed Vec storage');
assert.match(typesCode, /struct\s+BloomFilter<\.T,\.H>:\s+[\s\S]*\bnbits\s+<i32>[\s\S]*\bnbytes\s+<i32>[\s\S]*\bbits\s+<Vec<u8>>/, 'BloomFilter must store bit array bytes as a typed Vec<u8> owner');
assert.match(hashCode, /#import\s+"alloc\/hash\/hash32"\s+as\s+hash32/, 'BloomFilter hash module must import hash32 explicitly');
assert.match(hashCode, /fn\s+bloom_hash0\s+<\.T:\s*HashKey&Copy,\.H:\s*Hasher<\.T>&Copy>/, 'BloomFilter hash module must own primary hash calculation');
assert.match(hashCode, /hash32::mix\s+xor\s+h0\s+hk/, 'BloomFilter secondary hash must call hash32::mix through a qualified dependency');
assert.match(hashCode, /fn\s+bloom_hash1\s+<\.T:\s*HashKey&Copy,\.H:\s*Hasher<\.T>&Copy>[\s\S]*hashkey_hash32\s+key[\s\S]*if\s+eq\s+mixed\s+0\s+1\s+mixed/, 'BloomFilter hash module must own non-zero secondary hash calculation');
assert.match(hashCode, /fn\s+bloom_probe_index\s+<\(i32,i32,i32\)->i32>[\s\S]*rem_u\s+add\s+h0\s+h1\s+nbits/, 'BloomFilter hash module must own probe index calculation');
assert.match(layoutCode, /fn\s+bloom_byte_len\s+<\(i32\)->i32>[\s\S]*div_s\s+add\s+nbits\s+7\s+8/, 'BloomFilter layout module must own byte length rounding');
assert.match(layoutCode, /fn\s+bloom_byte_index\s+<\(i32\)->i32>[\s\S]*div_s\s+bit_idx\s+8/, 'BloomFilter layout module must own byte index calculation');
assert.match(layoutCode, /fn\s+bloom_bit_mask\s+<\(i32\)->i32>[\s\S]*shl\s+1\s+rem_s\s+bit_idx\s+8/, 'BloomFilter layout module must own bit mask calculation');
assert.match(storageCode, /fn\s+bloom_alloc_bits\s+<\(i32,i32\)\*>Result<Vec<u8>,\s*Diag>>[\s\S]*vec::filled<u8>\s+nbytes\s+byte/, 'BloomFilter must allocate initialized byte storage through Vec.filled');
assert.match(storageCode, /fn\s+bloom_byte_at\s+<\(&Vec<u8>,i32\)->Option<i32>>[\s\S]*vec::get<u8>[\s\S]*Option::None:[\s\S]*none<i32>/, 'BloomFilter must read bit bytes through Vec.get and expose missing bytes as Option');
assert.match(storageCode, /fn\s+bloom_store_byte\s+<\(&Vec<u8>,i32,i32\)\*>\(\)>[\s\S]*vec::replace<u8>/, 'BloomFilter must update bit bytes through Vec.replace');
assert.match(storageCode, /fn\s+bloom_free_bits\s+<\(Vec<u8>\)->\(\)>[\s\S]*vec::free<u8>\s+bits/, 'BloomFilter storage module must centralize Vec cleanup');
assert.match(mutationCode, /fn\s+bloom_set_bit\s+<\(&Vec<u8>,i32\)\*>\(\)>[\s\S]*bloom_byte_index[\s\S]*bloom_bit_mask[\s\S]*bloom_store_byte/, 'BloomFilter mutation module must own bit set read-modify-write');
assert.match(mutationCode, /fn\s+bloom_test_bit\s+<\(&Vec<u8>,i32\)->bool>[\s\S]*bloom_byte_at[\s\S]*ne\s+and\s+cur\s+mask\s+0/, 'BloomFilter mutation module must own bit test');
assert.match(apiCode, /fn\s+new\s+<\.T:\s*HashKey&Copy,\.H:\s*Hasher<\.T>&Copy>[\s\S]*bloom_byte_len\s+nbits[\s\S]*bloom_alloc_bits\s+nbytes\s+0/, 'BloomFilter.new must use layout and storage helpers');
assert.match(apiCode, /fn\s+len\s+<\.T:\s*HashKey&Copy,\.H:\s*Hasher<\.T>&Copy>\s+<\(&BloomFilter<\.T,\.H>\)->i32>\s+\(bf\):/, 'BloomFilter.len must borrow the owner and remain Copy-only while drop traversal is incomplete');
assert.match(apiCode, /fn\s+insert\s+<\.T:\s*HashKey&Copy,\.H:\s*Hasher<\.T>&Copy>[\s\S]*bloom_hash0[\s\S]*bloom_set_bit\s+bits\s+i0[\s\S]*bloom_set_bit\s+bits\s+i2/, 'BloomFilter.insert must use hash and mutation helpers');
assert.match(apiCode, /fn\s+contains\s+<\.T:\s*HashKey&Copy,\.H:\s*Hasher<\.T>&Copy>[\s\S]*bloom_test_bit\s+bits\s+i0[\s\S]*bloom_test_bit\s+bits\s+i2/, 'BloomFilter.contains must use hash and mutation helpers');
assert.match(apiCode, /fn\s+clear\s+<\.T:\s*HashKey&Copy,\.H:\s*Hasher<\.T>&Copy>\s+<\(BloomFilter<\.T,\.H>\)\*>BloomFilter<\.T,\.H>>/, 'BloomFilter.clear must expose the same Copy-only key/hasher contract as mutating APIs');
assert.match(apiCode, /fn\s+free\s+<\.T:\s*HashKey&Copy,\.H:\s*Hasher<\.T>&Copy>\s+<\(BloomFilter<\.T,\.H>\)->\(\)>[\s\S]*bloom_free_bits\s+bits/, 'BloomFilter.free must close typed Vec<u8> storage through cleanup helper with Copy-only key/hasher bounds');

assert.doesNotMatch(code, /\bMemPtr\b/, 'BloomFilter must not expose raw MemPtr storage');
assert.doesNotMatch(code, /\bmem_ptr_wrap\b/, 'BloomFilter must not use raw pointer arithmetic');
assert.doesNotMatch(code, /\bmem_ptr_addr\b/, 'BloomFilter must not recover raw addresses for bit storage');
assert.doesNotMatch(code, /\balloc_ptr\b/, 'BloomFilter must not allocate bit storage through raw pointer APIs');
assert.doesNotMatch(code, /\bload_u8\b/, 'BloomFilter must not read bit storage through raw byte loads');
assert.doesNotMatch(code, /\bstore_u8\b/, 'BloomFilter must not write bit storage through raw byte stores');
assert.doesNotMatch(code, /\bdealloc_raw\b/, 'BloomFilter must not deallocate bit storage through raw pointer APIs');
assert.doesNotMatch(code, /fn\s+free\s+<\.T,\.H>\s+<\(BloomFilter<\.T,\.H>\)->\(\)>[\s\S]*dealloc_ptr/, 'BloomFilter.free must not unwrap checked deallocation for owned storage');
assert.doesNotMatch(apiCode, /fn\s+clear\s+<\.T,\.H>\s+<\(BloomFilter<\.T,\.H>\)\*>BloomFilter<\.T,\.H>>/, 'BloomFilter.clear must not accept unconstrained key/hasher payloads');
assert.doesNotMatch(apiCode, /fn\s+free\s+<\.T,\.H>\s+<\(BloomFilter<\.T,\.H>\)->\(\)>/, 'BloomFilter.free must not accept unconstrained key/hasher payloads');

console.log('bloom filter unsafe unwrap regression passed');

function sourceWithoutComments(file) {
    return fs.readFileSync(path.join(repoRoot, file), 'utf8')
        .split(/\r?\n/)
        .filter((line) => !/^\s*\/\//.test(line))
        .join('\n');
}
