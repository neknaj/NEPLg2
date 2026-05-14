#!/usr/bin/env node

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');

const repoRoot = path.resolve(__dirname, '..');
const modulePaths = {
    root: 'stdlib/alloc/collections/hashset.nepl',
    types: 'stdlib/alloc/collections/hashset/types.nepl',
    storage: 'stdlib/alloc/collections/hashset/storage.nepl',
    probe: 'stdlib/alloc/collections/hashset/probe.nepl',
    rehash: 'stdlib/alloc/collections/hashset/rehash.nepl',
    api: 'stdlib/alloc/collections/hashset/api.nepl',
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
        new RegExp(`pub\\s+#import\\s+"alloc/collections/hashset/${submodule}"\\s+as\\s+@merge`),
        `HashSet root facade must publicly merge ${submodule}`,
    );
}

assert.doesNotMatch(
    codes.root,
    /\b(fn|struct|enum)\s+\w+/,
    'HashSet root facade must not keep implementation bodies after module split',
);

const allocStorageStart = codes.storage.indexOf('fn hashset_alloc_storage ');
assert.notEqual(allocStorageStart, -1, 'missing section start: fn hashset_alloc_storage ');
const allocStorageSection = codes.storage.slice(allocStorageStart);
const findPresentSection = between(codes.probe, 'fn hashset_find_present ', 'fn hashset_find_insert_slot ');
const findInsertSlotSection = between(codes.probe, 'fn hashset_find_insert_slot ', 'fn hashset_insert_entry_into_storage ');
const rehashSection = between(codes.rehash, 'fn hashset_rehash_to ', 'fn hashset_prepare_insert ');
const insertSection = between(codes.api, 'fn insert ', 'fn contains ');
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
        'HashSet modules must keep storage typed and must not use raw storage or unsafe unwrap helpers',
    );
}

assert.match(
    codes.types,
    /enum\s+HashSetBucketState:\s+Empty\s+Full\s+Tombstone/,
    'HashSet bucket state must be represented as an enum',
);

assert.match(
    codes.types,
    /enum\s+HashSetInsertSlotState:\s+EmptySlot\s+TombstoneSlot/,
    'HashSet insertion slot state must be represented as an enum',
);

assert.match(
    codes.types,
    /struct\s+HashSetStorage<\.T>:\s+states\s+<Vec<HashSetBucketState>>\s+keys\s+<Vec<Option<\.T>>>/,
    'HashSet storage must keep initialized state and keys as typed Vec owners',
);

assert.match(
    codes.types,
    /struct\s+HashSet<\.T,\.H>:\s+count\s+<i32>\s+cap\s+<i32>\s+tombstones\s+<i32>\s+storage\s+<HashSetStorage<\.T>>\s+hasher\s+<\.H>/,
    'HashSet must own typed storage directly instead of raw header or entries pointers',
);

assert.match(
    allocStorageSection,
    /vec::filled<HashSetBucketState>\s+cap\s+HashSetBucketState::Empty[\s\S]*vec::filled<Option<\.T>>\s+cap\s+none<\.T>[\s\S]*ok<HashSetStorage<\.T>,\s*Diag>\s+HashSetStorage<\.T>\s+states\s+keys/,
    'HashSet storage allocation must initialize all state/key slots',
);

assert.match(
    allocStorageSection,
    /Result::Err\s+_e:[\s\S]*vec::free<HashSetBucketState>\s+states/,
    'HashSet key storage allocation failure must release already allocated state Vec owner',
);

assert.match(
    findPresentSection,
    /match\s+hashset_state_at\s+states\s+cur:[\s\S]*HashSetBucketState::Empty:[\s\S]*HashSetBucketState::Tombstone:[\s\S]*HashSetBucketState::Full:/,
    'HashSet lookup must branch on bucket state with exhaustive enum match',
);

assert.match(
    findInsertSlotSection,
    /match\s+hashset_state_at\s+states\s+cur:[\s\S]*HashSetBucketState::Empty:[\s\S]*HashSetBucketState::Tombstone:[\s\S]*HashSetBucketState::Full:/,
    'HashSet insertion slot search must branch on bucket state with exhaustive enum match',
);

assert.match(
    rehashSection,
    /let\s+old_storage\s+<HashSetStorage<\.T>>\s+field::get\s+hs\s+"storage"[\s\S]*Result::Err\s+d:[\s\S]*hashset_free_storage<\.T>\s+old_storage[\s\S]*Result::Ok\s+new_storage:[\s\S]*hashset_free_storage<\.T>\s+old_storage/,
    'HashSet rehash must own old storage and release it on both failure and success paths',
);

assert.match(
    insertSection,
    /let\s+storage\s+<HashSetStorage<\.T>>\s+field::get\s+ready\s+"storage"[\s\S]*ok<HashSet<\.T,\.H>,\s*Diag>\s+HashSet<\.T,\.H>[\s\S]*storage\s+hasher/,
    'HashSet insert must transfer storage owner from the consumed set into the returned set',
);

assert.match(
    containsSection,
    /fn\s+contains\s+<\.T:\s*HashKey&Copy,\.H:\s*Hasher<\.T>&Copy>\s+<\(&HashSet<\.T,\.H>,\.T\)->bool>/,
    'HashSet.contains must be a borrow-based read API',
);

assert.match(
    lenSection,
    /fn\s+len\s+<\.T,\.H>\s+<\(&HashSet<\.T,\.H>\)->i32>/,
    'HashSet.len must be a borrow-based read API',
);

assert.match(
    freeSection,
    /fn\s+free\s+<\.T:\s*HashKey&Copy,\.H:\s*Hasher<\.T>&Copy>\s+<\(HashSet<\.T,\.H>\)->\(\)>/,
    'HashSet.free must expose the Copy-only key/hasher cleanup contract',
);

assert.match(
    freeSection,
    /let\s+storage\s+<HashSetStorage<\.T>>\s+field::get\s+hs\s+"storage"[\s\S]*hashset_free_storage<\.T>\s+storage/,
    'HashSet.free must release the typed storage owner',
);

console.log('hashset storage contract regression passed');
