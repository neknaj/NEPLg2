#!/usr/bin/env node

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const { legacyTypeSyntaxView } = require('./source_policy/nepl_source_view');

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
        new RegExp(`pub\\s+#import\\s+"alloc/collections/hashset/${submodule}"\\s+as\\s+@merge`),
        `HashSet root facade must publicly merge ${submodule}`,
    );
}

for (const submodule of ['storage', 'probe', 'rehash']) {
    assert.doesNotMatch(
        codes.root,
        new RegExp(`pub\\s+#import\\s+"alloc/collections/hashset/${submodule}"\\s+as\\s+@merge`),
        `HashSet root facade must not publicly merge internal ${submodule}`,
    );
}

assert.doesNotMatch(
    codes.root,
    /\b(fn|struct|enum)\s+\w+/,
    'HashSet root facade must not keep implementation bodies after module split',
);

assert.doesNotMatch(
    codes.root,
    /^\s*#import\s+/m,
    'HashSet root facade must not keep private implementation imports',
);

const allocStorageStart = codes.storage.indexOf('fn hashset_alloc_storage ');
assert.notEqual(allocStorageStart, -1, 'missing section start: fn hashset_alloc_storage ');
const allocStorageSection = codes.storage.slice(allocStorageStart);
const findPresentSection = between(codes.probe, 'fn hashset_find_present ', 'fn hashset_find_insert_slot ');
const findInsertSlotSection = between(codes.probe, 'fn hashset_find_insert_slot ', 'fn hashset_insert_entry_into_storage ');
const rehashSection = between(codes.rehash, 'fn hashset_rehash_to ', 'fn hashset_prepare_insert ');
const rehashErrSection = between(rehashSection, 'Result::Err d:', 'Result::Ok new_storage:');
const insertSection = between(codes.api, 'fn insert ', 'fn contains ');
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
    codes.types,
    /struct\s+HashSetUpdateError<\.T,\.H>:\s+owner\s+<HashSet<\.T,\.H>>\s+diag\s+<Diag>/,
    'HashSet owner-consuming update errors must carry the consumed set owner and diagnostic',
);

assert.match(
    codes.types,
    /fn\s+hashset_update_error_owner\s+<\.T:\s*HashKey&Copy,\.H:\s*Hasher<\.T>&Copy>\s+<\(HashSetUpdateError<\.T,\.H>\)->HashSet<\.T,\.H>>/,
    'HashSet update error must expose an owner-returning accessor with the collection cleanup contract',
);

assert.match(
    allocStorageSection,
    /vec::filled\s+cap\s+HashSetBucketState::Empty[\s\S]*vec::filled\s+cap\s+none[\s\S]*ok\s+HashSetStorage<\.T>\s+states\s+keys/,
    'HashSet storage allocation must initialize all state/key slots',
);

assert.match(
    allocStorageSection,
    /Result::Err\s+_e:[\s\S]*vec::free\s+states/,
    'HashSet key storage allocation failure must release already allocated state Vec owner',
);

assert.match(
    findPresentSection,
    /match\s+hashset_state_at\s+states\s+cur:[\s\S]*HashSetBucketState::Empty:[\s\S]*HashSetBucketState::Tombstone:[\s\S]*HashSetBucketState::Full:/,
    'HashSet lookup must branch on bucket state with exhaustive enum match',
);

assert.match(
    findPresentSection,
    /fn\s+hashset_find_present[\s\S]*->Option<i32>/,
    'HashSet lookup absence must be represented by Option<i32> instead of a numeric sentinel',
);

assert.doesNotMatch(
    findPresentSection,
    /\b-1\b|\b(?:not\s+)?lt\s+\w+\s+0\b/,
    'HashSet lookup must not encode absence or presence with -1 / index < 0 checks',
);

assert.match(
    findInsertSlotSection,
    /match\s+hashset_state_at\s+states\s+cur:[\s\S]*HashSetBucketState::Empty:[\s\S]*HashSetBucketState::Tombstone:[\s\S]*HashSetBucketState::Full:/,
    'HashSet insertion slot search must branch on bucket state with exhaustive enum match',
);

assert.match(
    findInsertSlotSection,
    /fn\s+hashset_find_insert_slot[\s\S]*->Option<HashSetInsertSlot>/,
    'HashSet insertion slot search must expose full-table or invariant failure as Option<HashSetInsertSlot>',
);

assert.doesNotMatch(
    findInsertSlotSection,
    /\b-1\b|\b(?:not\s+)?lt\s+\w+\s+0\b/,
    'HashSet insertion slot search must not use -1 or index < 0 as a hidden search state',
);

assert.doesNotMatch(
    `${rehashSection}\n${insertSection}\n${containsSection}\n${removeSection}`,
    /\b(?:not\s+)?lt\s+[^:\n]*hashset_find_present|\blet\s+\w+\s+<i32>\s+hashset_find_present|\b(?:not\s+)?lt\s+\w+\s+0\b/,
    'HashSet API and rehash callers must match on lookup Option instead of comparing probe results as integers',
);

assert.match(
    rehashSection,
    /fn\s+hashset_rehash_to\s+<\.T:\s*HashKey&Copy,\.H:\s*Hasher<\.T>&Copy>\s+<\(HashSet<\.T,\.H>,i32\)\*>Result<HashSet<\.T,\.H>,\s*HashSetUpdateError<\.T,\.H>>>/,
    'HashSet rehash must return owner-preserving update errors',
);

assert.match(
    rehashErrSection,
    /HashSetUpdateError<\.T,\.H>\s+\(HashSet<\.T,\.H>\s+count0\s+old_cap\s+tombstones0\s+old_storage\s+hasher\)\s+d/,
    'HashSet rehash allocation failure must return the consumed owner in the error payload',
);

assert.doesNotMatch(
    rehashErrSection,
    /hashset_free_storage<\.T>\s+old_storage/,
    'HashSet rehash allocation failure must not destroy the old owner before returning Err',
);

assert.match(
    rehashSection,
    /Result::Ok\s+new_storage:[\s\S]*hashset_free_storage<\.T>\s+old_storage/,
    'HashSet rehash success must release the old storage after moving live keys',
);

assert.match(
    insertSection,
    /fn\s+insert\s+<\.T:\s*HashKey&Copy,\.H:\s*Hasher<\.T>&Copy>\s+<\(HashSet<\.T,\.H>,\.T\)\*>Result<HashSet<\.T,\.H>,\s*HashSetUpdateError<\.T,\.H>>>[\s\S]*let\s+storage\s+<HashSetStorage<\.T>>\s+field::get\s+ready\s+"storage"[\s\S]*Result::Ok\s+HashSet<\.T,\.H>[\s\S]*storage\s+hasher/,
    'HashSet insert must transfer storage owner from the consumed set into the returned set',
);

assert.doesNotMatch(
    `${rehashSection}\n${insertSection}\n${removeSection}`,
    /\b(?:ok|err)<HashSet<\.T,\.H>,\s*HashSetUpdateError<\.T,\.H>>/,
    'HashSet owner-bearing update results must use direct Result constructors, not generic helpers',
);

assert.match(
    removeSection,
    /fn\s+remove\s+<\.T:\s*HashKey&Copy,\.H:\s*Hasher<\.T>&Copy>\s+<\(HashSet<\.T,\.H>,\.T\)\*>Result<HashSet<\.T,\.H>,\s*HashSetUpdateError<\.T,\.H>>>[\s\S]*HashSetUpdateError<\.T,\.H>\s+\(HashSet<\.T,\.H>\s+count0\s+cap0\s+tombstones0\s+storage\s+hasher\)\s+diag_key_not_found/,
    'HashSet remove-missing must return the consumed owner instead of destroying storage internally',
);

assert.doesNotMatch(
    removeSection,
    /then:[\s\S]*hashset_free_storage<\.T>\s+storage[\s\S]*diag_key_not_found/,
    'HashSet remove-missing must not free the consumed owner before reporting the error',
);

assert.match(
    containsSection,
    /fn\s+contains\s+<\.T:\s*HashKey&Copy,\.H:\s*Hasher<\.T>&Copy>\s+<\(&HashSet<\.T,\.H>,\.T\)->bool>/,
    'HashSet.contains must be a borrow-based read API',
);

assert.match(
    lenSection,
    /fn\s+len\s+<\.T,\.H>\s+<\(&HashSet<\.T,\.H>\)->i32>/,
    'HashSet.len must be a borrow-based metadata read API without Copy or HashKey bounds',
);

assert.match(
    freeSection,
    /fn\s+free\s+<\.T:\s*HashKey&Copy,\.H:\s*Hasher<\.T>&Copy>\s+<\(HashSet<\.T,\.H>\)->unit>/,
    'HashSet.free must expose the Copy-only key/hasher cleanup contract',
);

assert.match(
    freeSection,
    /let\s+storage\s+<HashSetStorage<\.T>>\s+field::get\s+hs\s+"storage"[\s\S]*hashset_free_storage<\.T>\s+storage/,
    'HashSet.free must release the typed storage owner',
);

console.log('hashset storage contract regression passed');
