#!/usr/bin/env node

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');

const repoRoot = path.resolve(__dirname, '..');
const modulePaths = {
    root: 'stdlib/alloc/collections/hashmap.nepl',
    types: 'stdlib/alloc/collections/hashmap/types.nepl',
    storage: 'stdlib/alloc/collections/hashmap/storage.nepl',
    probe: 'stdlib/alloc/collections/hashmap/probe.nepl',
    rehash: 'stdlib/alloc/collections/hashmap/rehash.nepl',
    api: 'stdlib/alloc/collections/hashmap/api.nepl',
};

function readCode(relPath) {
    return fs
        .readFileSync(path.join(repoRoot, relPath), 'utf8')
        .split(/\r?\n/)
        .filter((line) => !/^\s*\/\//.test(line))
        .join('\n');
}

function between(source, start, end) {
    const startIdx = source.indexOf(start);
    assert.notEqual(startIdx, -1, `missing section start: ${start}`);
    const endIdx = source.indexOf(end, startIdx + start.length);
    assert.notEqual(endIdx, -1, `missing section end: ${end}`);
    return source.slice(startIdx, endIdx);
}

const codes = Object.fromEntries(Object.entries(modulePaths).map(([name, relPath]) => [name, readCode(relPath)]));
const allCode = Object.values(codes).join('\n');

for (const submodule of ['types', 'storage', 'probe', 'rehash', 'api']) {
    assert.match(
        codes.root,
        new RegExp(`pub\\s+#import\\s+"alloc/collections/hashmap/${submodule}"\\s+as\\s+@merge`),
        `HashMap root facade must publicly merge ${submodule}`,
    );
}

assert.doesNotMatch(
    codes.root,
    /\b(fn|struct|enum)\s+\w+/,
    'HashMap root facade must not keep implementation bodies after module split',
);

const allocStorageStart = codes.storage.indexOf('fn hashmap_alloc_storage ');
assert.notEqual(allocStorageStart, -1, 'missing section start: fn hashmap_alloc_storage ');
const allocStorageSection = codes.storage.slice(allocStorageStart);
const findPresentSection = between(codes.probe, 'fn hashmap_find_present ', 'fn hashmap_find_insert_slot ');
const findInsertSlotSection = between(codes.probe, 'fn hashmap_find_insert_slot ', 'fn hashmap_insert_entry_into_storage ');
const rehashSection = between(codes.rehash, 'fn hashmap_rehash_to ', 'fn hashmap_prepare_insert ');
const insertSection = between(codes.api, 'fn insert ', 'fn get ');
const getSection = between(codes.api, 'fn get ', 'fn contains ');
const containsSection = between(codes.api, 'fn contains ', 'fn remove ');
const lenSection = between(codes.api, 'fn len ', 'fn free ');
const freeSection = codes.api.slice(codes.api.indexOf('fn free '));

const forbiddenImplementationPatterns = [
    /\balloc_raw\b/,
    /\bdealloc_raw\b/,
    /\bload_i32\b/,
    /\bstore_i32\b/,
    /\bMemPtr\b/,
    /\bunwrap\b/,
    /\bunwrap_ok\b/,
    /\bunwrap_err\b/,
    /\buwok\b/,
    /\buwerr\b/,
    /#intrinsic\s+"unreachable"/,
];

for (const pattern of forbiddenImplementationPatterns) {
    assert.doesNotMatch(
        allCode,
        pattern,
        'HashMap modules must keep storage typed and must not use raw storage or unsafe unwrap helpers',
    );
}

assert.match(
    codes.types,
    /enum\s+HashMapBucketState:\s+Empty\s+Full\s+Tombstone/,
    'HashMap bucket state must be represented as an enum',
);

assert.match(
    codes.types,
    /enum\s+HashMapInsertSlotState:\s+EmptySlot\s+TombstoneSlot/,
    'HashMap insertion slot state must be represented as an enum',
);

assert.match(
    codes.types,
    /struct\s+HashMapStorage<\.K,\.V>:\s+states\s+<Vec<HashMapBucketState>>\s+keys\s+<Vec<Option<\.K>>>\s+values\s+<Vec<Option<\.V>>>/,
    'HashMap storage must keep initialized state, keys, and values as typed Vec owners',
);

assert.match(
    codes.types,
    /struct\s+HashMap<\.K,\.V,\.H>:\s+count\s+<i32>\s+cap\s+<i32>\s+tombstones\s+<i32>\s+storage\s+<HashMapStorage<\.K,\.V>>\s+hasher\s+<\.H>/,
    'HashMap must own typed storage directly instead of raw header or entries pointers',
);

assert.match(
    allocStorageSection,
    /vec::filled<HashMapBucketState>\s+cap\s+HashMapBucketState::Empty[\s\S]*vec::filled<Option<\.K>>\s+cap\s+none<\.K>[\s\S]*vec::filled<Option<\.V>>\s+cap\s+none<\.V>[\s\S]*ok<HashMapStorage<\.K,\.V>,\s*Diag>\s+HashMapStorage<\.K,\.V>\s+states\s+keys\s+values/,
    'HashMap storage allocation must initialize all state/key/value slots',
);

assert.match(
    allocStorageSection,
    /Result::Err\s+_e:[\s\S]*vec::free<HashMapBucketState>\s+states[\s\S]*Result::Err\s+_e:[\s\S]*vec::free<HashMapBucketState>\s+states[\s\S]*vec::free<Option<\.K>>\s+keys/,
    'HashMap storage allocation failures must release already allocated Vec owners',
);

assert.match(
    findPresentSection,
    /match\s+hashmap_state_at\s+states\s+cur:[\s\S]*HashMapBucketState::Empty:[\s\S]*HashMapBucketState::Tombstone:[\s\S]*HashMapBucketState::Full:/,
    'HashMap lookup must branch on bucket state with exhaustive enum match',
);

assert.match(
    findInsertSlotSection,
    /match\s+hashmap_state_at\s+states\s+cur:[\s\S]*HashMapBucketState::Empty:[\s\S]*HashMapBucketState::Tombstone:[\s\S]*HashMapBucketState::Full:/,
    'HashMap insertion slot search must branch on bucket state with exhaustive enum match',
);

assert.match(
    rehashSection,
    /let\s+old_storage\s+<HashMapStorage<\.K,\.V>>\s+field::get\s+hm\s+"storage"[\s\S]*Result::Err\s+d:[\s\S]*hashmap_free_storage<\.K,\.V>\s+old_storage[\s\S]*Result::Ok\s+new_storage:[\s\S]*hashmap_free_storage<\.K,\.V>\s+old_storage/,
    'HashMap rehash must own old storage and release it on both failure and success paths',
);

assert.match(
    insertSection,
    /let\s+storage\s+<HashMapStorage<\.K,\.V>>\s+field::get\s+ready\s+"storage"[\s\S]*ok<HashMap<\.K,\.V,\.H>,\s*Diag>\s+HashMap<\.K,\.V,\.H>[\s\S]*storage\s+hasher/,
    'HashMap insert must transfer storage owner from the consumed map into the returned map',
);

assert.match(
    getSection,
    /fn\s+get\s+<\.K:\s*HashKey&Copy,\.V:\s*Copy,\.H:\s*Hasher<\.K>&Copy>\s+<\(&HashMap<\.K,\.V,\.H>,\.K\)->Option<\.V>>/,
    'HashMap.get must be a borrow-based read API returning Copy values',
);

assert.match(
    containsSection,
    /fn\s+contains\s+<\.K:\s*HashKey&Copy,\.V:\s*Copy,\.H:\s*Hasher<\.K>&Copy>\s+<\(&HashMap<\.K,\.V,\.H>,\.K\)->bool>/,
    'HashMap.contains must be a borrow-based read API',
);

assert.match(
    lenSection,
    /fn\s+len\s+<\.K,\.V,\.H>\s+<\(&HashMap<\.K,\.V,\.H>\)->i32>/,
    'HashMap.len must be a borrow-based read API',
);

assert.match(
    freeSection,
    /fn\s+free\s+<\.K:\s*HashKey&Copy,\.V:\s*Copy,\.H:\s*Hasher<\.K>&Copy>\s+<\(HashMap<\.K,\.V,\.H>\)->\(\)>/,
    'HashMap.free must expose the Copy-only key/value/hasher cleanup contract',
);

assert.match(
    freeSection,
    /let\s+storage\s+<HashMapStorage<\.K,\.V>>\s+field::get\s+hm\s+"storage"[\s\S]*hashmap_free_storage<\.K,\.V>\s+storage/,
    'HashMap.free must release the typed storage owner',
);

console.log('hashmap storage contract regression passed');
