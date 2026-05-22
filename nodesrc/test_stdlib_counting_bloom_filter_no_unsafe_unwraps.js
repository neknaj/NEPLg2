#!/usr/bin/env node

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');

const repoRoot = path.resolve(__dirname, '..');
const rootPath = 'stdlib/alloc/collections/counting_bloom_filter.nepl';
const typesPath = 'stdlib/alloc/collections/counting_bloom_filter/types.nepl';
const hashPath = 'stdlib/alloc/collections/counting_bloom_filter/hash.nepl';
const storagePath = 'stdlib/alloc/collections/counting_bloom_filter/storage.nepl';
const mutationPath = 'stdlib/alloc/collections/counting_bloom_filter/mutation.nepl';
const apiPath = 'stdlib/alloc/collections/counting_bloom_filter/api.nepl';

const rootCode = sourceWithoutComments(rootPath);
const typesCode = sourceWithoutComments(typesPath);
const hashCode = sourceWithoutComments(hashPath);
const storageCode = sourceWithoutComments(storagePath);
const mutationCode = sourceWithoutComments(mutationPath);
const apiCode = sourceWithoutComments(apiPath);
const code = [rootCode, typesCode, hashCode, storageCode, mutationCode, apiCode].join('\n');

const forbidden = [
    /\bunwrap\b/,
    /\bunwrap_ok\b/,
    /\bunwrap_err\b/,
    /\buwok\b/,
    /\buwerr\b/,
    /#intrinsic\s+"unreachable"/,
];

for (const pattern of forbidden) {
    assert.doesNotMatch(code, pattern, 'CountingBloomFilter split modules must not use unsafe unwrap helpers in implementation code');
}

assert.doesNotMatch(rootCode, /\bfn\s+/, 'CountingBloomFilter root facade must not keep implementation bodies');
for (const submodule of ['types', 'api']) {
    assert.match(
        rootCode,
        new RegExp(`pub\\s+#import\\s+"\\.\\/counting_bloom_filter\\/${submodule}"\\s+as\\s+@merge`),
        `CountingBloomFilter root facade must re-export counting_bloom_filter/${submodule}`,
    );
}

assert.match(typesCode, /#import\s+"alloc\/collections\/vec"\s+as\s+vec/, 'CountingBloomFilter types must use typed Vec storage');
assert.match(typesCode, /struct\s+CountingBloomFilter<\.T,\.H>:\s+[\s\S]*\bnslots\s+<i32>[\s\S]*\bcounters\s+<Vec<u8>>[\s\S]*\bhasher\s+<\.H>/, 'CountingBloomFilter must store counter bytes as a typed Vec<u8> owner');
assert.doesNotMatch(typesCode, /\bnbytes\s+<i32>/, 'CountingBloomFilter must not keep a duplicate nbytes field for u8 counters');
assert.match(hashCode, /#import\s+"alloc\/hash\/hash32"\s+as\s+hash32/, 'CountingBloomFilter hash module must import hash32 explicitly');
assert.match(hashCode, /fn\s+counting_bloom_hash0\s+<\.T:\s*HashKey&Copy,\.H:\s*Hasher<\.T>&Copy>/, 'CountingBloomFilter hash module must own primary hash calculation');
assert.match(hashCode, /hash32::mix\s+xor\s+h0\s+hk/, 'CountingBloomFilter secondary hash must call hash32::mix through a qualified dependency');
assert.match(hashCode, /fn\s+counting_bloom_hash1\s+<\.T:\s*HashKey&Copy,\.H:\s*Hasher<\.T>&Copy>[\s\S]*hashkey_hash32\s+key[\s\S]*if\s+eq\s+mixed\s+0\s+1\s+mixed/, 'CountingBloomFilter hash module must own non-zero secondary hash calculation');
assert.match(hashCode, /fn\s+counting_bloom_probe_index\s+<\(i32,i32,i32\)->i32>[\s\S]*rem_u\s+add\s+h0\s+h1\s+nslots/, 'CountingBloomFilter hash module must own probe index calculation');
assert.match(storageCode, /fn\s+counting_bloom_alloc_counters\s+<\(i32,i32\)\*>Result<Vec<u8>,\s*Diag>>[\s\S]*vec::filled<u8>\s+nslots\s+counter/, 'CountingBloomFilter must allocate initialized counter storage through Vec.filled');
assert.match(storageCode, /fn\s+counting_bloom_counter_at\s+<\(&Vec<u8>,i32\)->Option<i32>>[\s\S]*vec::get<u8>[\s\S]*Option::None:[\s\S]*none<i32>/, 'CountingBloomFilter must read counters through Vec.get and expose missing counters as Option');
assert.match(storageCode, /fn\s+counting_bloom_store_counter\s+<\(&Vec<u8>,i32,i32\)\*>\(\)>[\s\S]*vec::replace<u8>/, 'CountingBloomFilter must update counters through Vec.replace');
assert.match(storageCode, /fn\s+counting_bloom_free_counters\s+<\(Vec<u8>\)->\(\)>[\s\S]*vec::free<u8>\s+counters/, 'CountingBloomFilter storage module must centralize Vec cleanup');
assert.match(mutationCode, /fn\s+counting_bloom_counter_inc\s+<\(&Vec<u8>,i32\)\*>\(\)>[\s\S]*lt\s+cur\s+255[\s\S]*counting_bloom_store_counter\s+counters\s+idx\s+add\s+cur\s+1/, 'CountingBloomFilter mutation module must own saturating increment');
assert.match(mutationCode, /fn\s+counting_bloom_counter_dec\s+<\(&Vec<u8>,i32\)\*>\(\)>[\s\S]*lt\s+0\s+cur[\s\S]*counting_bloom_store_counter\s+counters\s+idx\s+sub\s+cur\s+1/, 'CountingBloomFilter mutation module must own lower-bounded decrement');
assert.match(mutationCode, /fn\s+counting_bloom_counter_nonzero\s+<\(&Vec<u8>,i32\)->bool>[\s\S]*counting_bloom_counter_at[\s\S]*lt\s+0\s+cur/, 'CountingBloomFilter mutation module must own nonzero test');
assert.match(apiCode, /fn\s+new\s+<\.T:\s*HashKey&Copy,\.H:\s*Hasher<\.T>&Copy>[\s\S]*counting_bloom_alloc_counters\s+nslots\s+0/, 'CountingBloomFilter.new must use storage allocation helper');
assert.match(apiCode, /fn\s+len\s+<\.T,\.H>\s+<\(&CountingBloomFilter<\.T,\.H>\)->i32>\s+\(bf\):/, 'CountingBloomFilter.len must borrow the owner and not require Copy or HashKey for metadata-only observation');
assert.match(apiCode, /fn\s+insert\s+<\.T:\s*HashKey&Copy,\.H:\s*Hasher<\.T>&Copy>[\s\S]*counting_bloom_hash0[\s\S]*counting_bloom_counter_inc\s+counters\s+i0[\s\S]*counting_bloom_counter_inc\s+counters\s+i2/, 'CountingBloomFilter.insert must use hash and mutation helpers');
assert.match(apiCode, /fn\s+remove\s+<\.T:\s*HashKey&Copy,\.H:\s*Hasher<\.T>&Copy>[\s\S]*counting_bloom_hash0[\s\S]*counting_bloom_counter_dec\s+counters\s+i0[\s\S]*counting_bloom_counter_dec\s+counters\s+i2/, 'CountingBloomFilter.remove must use hash and mutation helpers');
assert.match(apiCode, /fn\s+contains\s+<\.T:\s*HashKey&Copy,\.H:\s*Hasher<\.T>&Copy>[\s\S]*counting_bloom_counter_nonzero\s+counters\s+i0[\s\S]*counting_bloom_counter_nonzero\s+counters\s+i2/, 'CountingBloomFilter.contains must use hash and mutation helpers');
assert.match(apiCode, /fn\s+clear\s+<\.T:\s*HashKey&Copy,\.H:\s*Hasher<\.T>&Copy>\s+<\(CountingBloomFilter<\.T,\.H>\)\*>CountingBloomFilter<\.T,\.H>>/, 'CountingBloomFilter.clear must expose the same Copy-only key/hasher contract as mutating APIs');
assert.match(apiCode, /fn\s+free\s+<\.T:\s*HashKey&Copy,\.H:\s*Hasher<\.T>&Copy>\s+<\(CountingBloomFilter<\.T,\.H>\)->\(\)>[\s\S]*counting_bloom_free_counters\s+counters/, 'CountingBloomFilter.free must close typed Vec<u8> storage through cleanup helper with Copy-only key/hasher bounds');

assert.doesNotMatch(code, /\bMemPtr\b/, 'CountingBloomFilter must not expose raw MemPtr storage');
assert.doesNotMatch(code, /\bmem_ptr_wrap\b/, 'CountingBloomFilter must not use raw pointer arithmetic');
assert.doesNotMatch(code, /\bmem_ptr_addr\b/, 'CountingBloomFilter must not recover raw addresses for counter storage');
assert.doesNotMatch(code, /\balloc_ptr\b/, 'CountingBloomFilter must not allocate counter storage through raw pointer APIs');
assert.doesNotMatch(code, /\bload_u8\b/, 'CountingBloomFilter must not read counter storage through raw byte loads');
assert.doesNotMatch(code, /\bstore_u8\b/, 'CountingBloomFilter must not write counter storage through raw byte stores');
assert.doesNotMatch(code, /\bdealloc_raw\b/, 'CountingBloomFilter must not deallocate counter storage through raw pointer APIs');
assert.doesNotMatch(code, /fn\s+free\s+<\.T,\.H>\s+<\(CountingBloomFilter<\.T,\.H>\)->\(\)>[\s\S]*dealloc_ptr/, 'CountingBloomFilter.free must not unwrap checked deallocation for owned storage');
assert.doesNotMatch(apiCode, /fn\s+clear\s+<\.T,\.H>\s+<\(CountingBloomFilter<\.T,\.H>\)\*>CountingBloomFilter<\.T,\.H>>/, 'CountingBloomFilter.clear must not accept unconstrained key/hasher payloads');
assert.doesNotMatch(apiCode, /fn\s+free\s+<\.T,\.H>\s+<\(CountingBloomFilter<\.T,\.H>\)->\(\)>/, 'CountingBloomFilter.free must not accept unconstrained key/hasher payloads');

console.log('counting bloom filter unsafe unwrap regression passed');

function sourceWithoutComments(file) {
    return fs.readFileSync(path.join(repoRoot, file), 'utf8')
        .split(/\r?\n/)
        .filter((line) => !/^\s*\/\//.test(line))
        .join('\n');
}
