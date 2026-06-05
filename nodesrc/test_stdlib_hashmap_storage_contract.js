#!/usr/bin/env node

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const { legacyTypeSyntaxView } = require('./source_policy/nepl_source_view');

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
    return legacyTypeSyntaxView(fs.readFileSync(path.join(repoRoot, relPath), 'utf8'));
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

for (const submodule of ['types', 'api']) {
    assert.match(
        codes.root,
        new RegExp(`pub\\s+#import\\s+"alloc/collections/hashmap/${submodule}"\\s+as\\s+@merge`),
        `HashMap root facade must publicly merge ${submodule}`,
    );
}

for (const submodule of ['storage', 'probe', 'rehash']) {
    assert.doesNotMatch(
        codes.root,
        new RegExp(`pub\\s+#import\\s+"alloc/collections/hashmap/${submodule}"\\s+as\\s+@merge`),
        `HashMap root facade must not publicly merge internal ${submodule}`,
    );
}

assert.doesNotMatch(
    codes.root,
    /\b(fn|struct|enum)\s+\w+/,
    'HashMap root facade must not keep implementation bodies after module split',
);

assert.doesNotMatch(
    codes.root,
    /^\s*#import\s+/m,
    'HashMap root facade must not keep private implementation imports',
);

const allocStorageStart = codes.storage.indexOf('fn hashmap_alloc_storage ');
assert.notEqual(allocStorageStart, -1, 'missing section start: fn hashmap_alloc_storage ');
const allocStorageSection = codes.storage.slice(allocStorageStart);
const findPresentSection = between(codes.probe, 'fn hashmap_find_present ', 'fn hashmap_find_insert_slot ');
const findInsertSlotSection = between(codes.probe, 'fn hashmap_find_insert_slot ', 'fn hashmap_insert_entry_into_storage ');
const rehashSection = between(codes.rehash, 'fn hashmap_rehash_to ', 'fn hashmap_prepare_insert ');
const rehashErrSection = between(rehashSection, 'Result::Err d:', 'Result::Ok new_storage:');
const insertSection = between(codes.api, 'fn insert ', 'fn get ');
const getSection = between(codes.api, 'fn get ', 'fn contains ');
const containsSection = between(codes.api, 'fn contains ', 'fn remove ');
const removeSection = between(codes.api, 'fn remove ', 'fn len ');
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
    codes.types,
    /struct\s+HashMapUpdateError<\.K,\.V,\.H>:\s+owner\s+<HashMap<\.K,\.V,\.H>>\s+diag\s+<Diag>/,
    'HashMap owner-consuming update errors must carry the consumed map owner and diagnostic',
);

assert.match(
    codes.types,
    /fn\s+hashmap_update_error_owner\s+<\.K:\s*HashKey&Copy,\.V:\s*Copy,\.H:\s*Hasher<\.K>&Copy>\s+<\(HashMapUpdateError<\.K,\.V,\.H>\)->HashMap<\.K,\.V,\.H>>/,
    'HashMap update error must expose an owner-returning accessor with the collection cleanup contract',
);

assert.match(
    allocStorageSection,
    /vec::filled\s+cap\s+HashMapBucketState::Empty[\s\S]*vec::filled\s+cap\s+none[\s\S]*vec::filled\s+cap\s+none[\s\S]*ok\s+HashMapStorage<\.K,\.V>\s+states\s+keys\s+values/,
    'HashMap storage allocation must initialize all state/key/value slots',
);

assert.match(
    allocStorageSection,
    /Result::Err\s+_e:[\s\S]*vec::free\s+states[\s\S]*Result::Err\s+_e:[\s\S]*vec::free\s+states[\s\S]*vec::free\s+keys/,
    'HashMap storage allocation failures must release already allocated Vec owners',
);

assert.match(
    findPresentSection,
    /match\s+hashmap_state_at\s+states\s+cur:[\s\S]*HashMapBucketState::Empty:[\s\S]*HashMapBucketState::Tombstone:[\s\S]*HashMapBucketState::Full:/,
    'HashMap lookup must branch on bucket state with exhaustive enum match',
);

assert.match(
    findPresentSection,
    /fn\s+hashmap_find_present[\s\S]*->Option<i32>/,
    'HashMap lookup absence must be represented by Option<i32> instead of a numeric sentinel',
);

assert.doesNotMatch(
    findPresentSection,
    /\b-1\b|\b(?:not\s+)?lt\s+\w+\s+0\b/,
    'HashMap lookup must not encode absence or presence with -1 / index < 0 checks',
);

assert.match(
    findInsertSlotSection,
    /match\s+hashmap_state_at\s+states\s+cur:[\s\S]*HashMapBucketState::Empty:[\s\S]*HashMapBucketState::Tombstone:[\s\S]*HashMapBucketState::Full:/,
    'HashMap insertion slot search must branch on bucket state with exhaustive enum match',
);

assert.match(
    findInsertSlotSection,
    /fn\s+hashmap_find_insert_slot[\s\S]*->Option<HashMapInsertSlot>/,
    'HashMap insertion slot search must expose full-table or invariant failure as Option<HashMapInsertSlot>',
);

assert.doesNotMatch(
    findInsertSlotSection,
    /\b-1\b|\b(?:not\s+)?lt\s+\w+\s+0\b/,
    'HashMap insertion slot search must not use -1 or index < 0 as a hidden search state',
);

assert.doesNotMatch(
    `${rehashSection}\n${insertSection}\n${getSection}\n${containsSection}\n${removeSection}`,
    /\b(?:not\s+)?lt\s+[^:\n]*hashmap_find_present|\blet\s+\w+\s+<i32>\s+hashmap_find_present|\b(?:not\s+)?lt\s+\w+\s+0\b/,
    'HashMap API and rehash callers must match on lookup Option instead of comparing probe results as integers',
);

assert.match(
    rehashSection,
    /fn\s+hashmap_rehash_to\s+<\.K:\s*HashKey&Copy,\.V:\s*Copy,\.H:\s*Hasher<\.K>&Copy>\s+<\(HashMap<\.K,\.V,\.H>,i32\)\*>Result<HashMap<\.K,\.V,\.H>,\s*HashMapUpdateError<\.K,\.V,\.H>>>/,
    'HashMap rehash must return owner-preserving update errors',
);

assert.match(
    rehashErrSection,
    /HashMapUpdateError<\.K,\.V,\.H>\s+\(HashMap<\.K,\.V,\.H>\s+count0\s+old_cap\s+tombstones0\s+old_storage\s+hasher\)\s+d/,
    'HashMap rehash allocation failure must return the consumed owner in the error payload',
);

assert.doesNotMatch(
    rehashErrSection,
    /hashmap_free_storage<\.K,\.V>\s+old_storage/,
    'HashMap rehash allocation failure must not destroy the old owner before returning Err',
);

assert.match(
    rehashSection,
    /Result::Ok\s+new_storage:[\s\S]*hashmap_free_storage<\.K,\.V>\s+old_storage/,
    'HashMap rehash success must release the old storage after moving live entries',
);

assert.match(
    insertSection,
    /fn\s+insert\s+<\.K:\s*HashKey&Copy,\.V:\s*Copy,\.H:\s*Hasher<\.K>&Copy>\s+<\(HashMap<\.K,\.V,\.H>,\.K,\.V\)\*>Result<HashMap<\.K,\.V,\.H>,\s*HashMapUpdateError<\.K,\.V,\.H>>>[\s\S]*let\s+storage\s+<HashMapStorage<\.K,\.V>>\s+field::get\s+ready\s+"storage"[\s\S]*Result::Ok\s+HashMap<\.K,\.V,\.H>[\s\S]*storage\s+hasher/,
    'HashMap insert must transfer storage owner from the consumed map into the returned map',
);

assert.doesNotMatch(
    `${rehashSection}\n${insertSection}\n${removeSection}`,
    /\b(?:ok|err)<HashMap<\.K,\.V,\.H>,\s*HashMapUpdateError<\.K,\.V,\.H>>/,
    'HashMap owner-bearing update results must use direct Result constructors, not generic helpers',
);

assert.match(
    removeSection,
    /fn\s+remove\s+<\.K:\s*HashKey&Copy,\.V:\s*Copy,\.H:\s*Hasher<\.K>&Copy>\s+<\(HashMap<\.K,\.V,\.H>,\.K\)\*>Result<HashMap<\.K,\.V,\.H>,\s*HashMapUpdateError<\.K,\.V,\.H>>>[\s\S]*HashMapUpdateError<\.K,\.V,\.H>\s+\(HashMap<\.K,\.V,\.H>\s+count0\s+cap0\s+tombstones0\s+storage\s+hasher\)\s+diag_key_not_found/,
    'HashMap remove-missing must return the consumed owner instead of destroying storage internally',
);

assert.doesNotMatch(
    removeSection,
    /then:[\s\S]*hashmap_free_storage<\.K,\.V>\s+storage[\s\S]*diag_key_not_found/,
    'HashMap remove-missing must not free the consumed owner before reporting the error',
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
    'HashMap.len must be a borrow-based metadata read API without Copy or HashKey bounds',
);

assert.match(
    freeSection,
    /fn\s+free\s+<\.K:\s*HashKey&Copy,\.V:\s*Copy,\.H:\s*Hasher<\.K>&Copy>\s+<\(HashMap<\.K,\.V,\.H>\)\*>unit>/,
    'HashMap.free must expose the Copy-only key/value/hasher impure cleanup contract',
);

assert.match(
    freeSection,
    /let\s+storage\s+<HashMapStorage<\.K,\.V>>\s+field::get\s+hm\s+"storage"[\s\S]*hashmap_free_storage<\.K,\.V>\s+storage/,
    'HashMap.free must release the typed storage owner',
);

console.log('hashmap storage contract regression passed');
