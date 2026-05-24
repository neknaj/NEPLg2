#!/usr/bin/env node

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const { legacyTypeSyntaxView } = require('./source_policy/nepl_source_view');

const repoRoot = path.resolve(__dirname, '..');

function functionBlock(file, name) {
    const src = fs.readFileSync(path.join(repoRoot, file), 'utf8');
    const lines = src.split(/\r?\n/);
    const declaration = new RegExp(`^(?:pub\\s+)?fn\\s+${name}\\s+`);
    const start = lines.findIndex((line) => declaration.test(line));
    assert.notEqual(start, -1, `${name} must exist in ${file}`);

    let end = lines.length;
    for (let i = start + 1; i < lines.length; i += 1) {
        if (/^(?:pub\s+)?fn\s+/.test(lines[i])) {
            end = i;
            break;
        }
    }

    return legacyTypeSyntaxView(lines
        .slice(start, end)
        .join('\n'));
}

function sourceWithoutComments(file) {
    const src = fs.readFileSync(path.join(repoRoot, file), 'utf8');
    return legacyTypeSyntaxView(src);
}

const forbidden = [
    /\bunwrap\b/,
    /\bunwrap_ok\b/,
    /\bunwrap_err\b/,
    /\buwok\b/,
    /\buwerr\b/,
    /#intrinsic\s+"unreachable"/,
];

function assertNoUnsafeUnwraps(file, names) {
    for (const name of names) {
        const code = functionBlock(file, name);
        for (const pattern of forbidden) {
            assert.doesNotMatch(code, pattern, `${file} ${name} must propagate grow errors without ${pattern}`);
        }
    }
}

const btreeMapApiFile = 'stdlib/alloc/collections/btreemap/api.nepl';
const btreeMapCreateFile = 'stdlib/alloc/collections/btreemap/api/create.nepl';
const btreeMapObserverFile = 'stdlib/alloc/collections/btreemap/api/observer.nepl';
const btreeMapInsertFile = 'stdlib/alloc/collections/btreemap/api/insert.nepl';
const btreeMapRemoveFile = 'stdlib/alloc/collections/btreemap/api/remove.nepl';
const btreeMapCleanupFile = 'stdlib/alloc/collections/btreemap/api/cleanup.nepl';
const btreeMapTypesFile = 'stdlib/alloc/collections/btreemap/types.nepl';
const btreeMapStorageFile = 'stdlib/alloc/collections/btreemap/storage.nepl';
const btreeMapSearchFile = 'stdlib/alloc/collections/btreemap/search.nepl';
const btreeMapRootSource = sourceWithoutComments('stdlib/alloc/collections/btreemap.nepl');
assert.doesNotMatch(btreeMapRootSource, /\bfn\s+/, 'BTreeMap root facade must not keep implementation bodies');
for (const submodule of ['types', 'api', 'alias']) {
    assert.match(
        btreeMapRootSource,
        new RegExp(`pub\\s+#import\\s+"\\.\\/btreemap\\/${submodule}"\\s+as\\s+@merge`),
        `BTreeMap root facade must re-export btreemap/${submodule}`,
    );
}
const btreeMapApiSource = sourceWithoutComments(btreeMapApiFile);
assert.doesNotMatch(btreeMapApiSource, /\bfn\s+/, 'BTreeMap api facade must not keep implementation bodies');
for (const submodule of ['create', 'observer', 'insert', 'remove', 'cleanup']) {
    assert.match(
        btreeMapApiSource,
        new RegExp(`pub\\s+#import\\s+"\\.\\/api\\/${submodule}"\\s+as\\s+@merge`),
        `BTreeMap api facade must re-export api/${submodule}`,
    );
}

const btreeMapInsert = functionBlock(btreeMapInsertFile, 'insert');
assert.match(btreeMapInsert, /match\s+btreemap_grow<\.K,\.V>\s+hm:/, 'BTreeMap.insert must match grow result');
assert.match(btreeMapInsert, /Result::Err\s+e:/, 'BTreeMap.insert must keep an owner-preserving Err arm');
assert.match(btreeMapInsert, /Result<BTreeMap<\.K,\.V>,\s*BTreeMapInsertError<\.K,\.V>>::Err\s+e/, 'BTreeMap.insert must return grow Err with the recovered owner through the typed Result variant');
assert.match(btreeMapInsert, /fn\s+insert\s+<\.K:\s*Ord&Copy,\.V:\s*Copy>\s+<\(BTreeMap<\.K,\.V>,\.K,\.V\)\*>Result<BTreeMap<\.K,\.V>,\s*BTreeMapInsertError<\.K,\.V>>>/, 'BTreeMap.insert must expose owner-preserving BTreeMapInsertError');
assertNoUnsafeUnwraps(btreeMapInsertFile, ['insert', 'btreemap_insert_ready']);

const btreeSetApiFile = 'stdlib/alloc/collections/btreeset/api.nepl';
const btreeSetCreateFile = 'stdlib/alloc/collections/btreeset/api/create.nepl';
const btreeSetObserverFile = 'stdlib/alloc/collections/btreeset/api/observer.nepl';
const btreeSetInsertFile = 'stdlib/alloc/collections/btreeset/api/insert.nepl';
const btreeSetRemoveFile = 'stdlib/alloc/collections/btreeset/api/remove.nepl';
const btreeSetCleanupFile = 'stdlib/alloc/collections/btreeset/api/cleanup.nepl';
const btreeSetTypesFile = 'stdlib/alloc/collections/btreeset/types.nepl';
const btreeSetStorageFile = 'stdlib/alloc/collections/btreeset/storage.nepl';
const btreeSetSearchFile = 'stdlib/alloc/collections/btreeset/search.nepl';
const btreeSetRootSource = sourceWithoutComments('stdlib/alloc/collections/btreeset.nepl');
assert.doesNotMatch(btreeSetRootSource, /\bfn\s+/, 'BTreeSet root facade must not keep implementation bodies');
for (const submodule of ['types', 'api', 'alias']) {
    assert.match(
        btreeSetRootSource,
        new RegExp(`pub\\s+#import\\s+"\\.\\/btreeset\\/${submodule}"\\s+as\\s+@merge`),
        `BTreeSet root facade must re-export btreeset/${submodule}`,
    );
}
const btreeSetApiSource = sourceWithoutComments(btreeSetApiFile);
assert.doesNotMatch(btreeSetApiSource, /\bfn\s+/, 'BTreeSet api facade must not keep implementation bodies');
for (const submodule of ['create', 'observer', 'insert', 'remove', 'cleanup']) {
    assert.match(
        btreeSetApiSource,
        new RegExp(`pub\\s+#import\\s+"\\.\\/api\\/${submodule}"\\s+as\\s+@merge`),
        `BTreeSet api facade must re-export api/${submodule}`,
    );
}

const btreeSetInsert = functionBlock(btreeSetInsertFile, 'insert');
assert.match(btreeSetInsert, /match\s+btreeset_grow<\.T>\s+set0:/, 'BTreeSet.insert must match grow result');
assert.match(btreeSetInsert, /Result::Err\s+e:/, 'BTreeSet.insert must keep an owner-preserving Err arm');
assert.match(btreeSetInsert, /Result<BTreeSet<\.T>,\s*BTreeSetInsertError<\.T>>::Err\s+e/, 'BTreeSet.insert must return grow Err with the recovered owner through the typed Result variant');
assert.match(btreeSetInsert, /fn\s+insert\s+<\.T:\s*Ord&Copy>\s+<\(BTreeSet<\.T>,\.T\)\*>Result<BTreeSet<\.T>,\s*BTreeSetInsertError<\.T>>>/, 'BTreeSet.insert must expose owner-preserving BTreeSetInsertError');
assertNoUnsafeUnwraps(btreeSetInsertFile, ['insert', 'btreeset_insert_ready']);

const btreeMapTypesSource = sourceWithoutComments(btreeMapTypesFile);
const btreeMapStorageSource = sourceWithoutComments(btreeMapStorageFile);
const btreeMapGrow = functionBlock(btreeMapStorageFile, 'btreemap_grow');
const btreeMapSearchSource = sourceWithoutComments(btreeMapSearchFile);
const btreeMapSource = [
    btreeMapRootSource,
    btreeMapTypesSource,
    btreeMapStorageSource,
    btreeMapSearchSource,
    btreeMapApiSource,
    sourceWithoutComments(btreeMapCreateFile),
    sourceWithoutComments(btreeMapObserverFile),
    sourceWithoutComments(btreeMapInsertFile),
    sourceWithoutComments(btreeMapRemoveFile),
    sourceWithoutComments(btreeMapCleanupFile),
    sourceWithoutComments('stdlib/alloc/collections/btreemap/alias.nepl'),
].join('\n');
assert.match(btreeMapSource, /struct BTreeMapStorage<\.K,\.V>:/, 'BTreeMap must keep typed storage wrapper');
assert.match(btreeMapSource, /struct BTreeMapInsertError<\.K,\.V>:[\s\S]*owner\s+<BTreeMap<\.K,\.V>>[\s\S]*diag\s+<Diag>/, 'BTreeMap insert failure must carry the consumed map owner and diagnostic');
assert.match(btreeMapGrow, /fn\s+btreemap_grow\s+<\.K:\s*Copy,\.V:\s*Copy>\s+<\(BTreeMap<\.K,\.V>\)\*>Result<BTreeMap<\.K,\.V>,\s*BTreeMapInsertError<\.K,\.V>>>/, 'BTreeMap grow must return an owner-preserving insert error');
assert.match(btreeMapGrow, /Result::Err\s+d:[\s\S]*Result<BTreeMap<\.K,\.V>,\s*BTreeMapInsertError<\.K,\.V>>::Err\s+BTreeMapInsertError<\.K,\.V>\s+\(BTreeMap<\.K,\.V>\s+len0\s+cap0\s+storage\)\s+d/, 'BTreeMap grow allocation failure must return the original map owner through the typed Result variant');
assert.doesNotMatch(btreeMapGrow, /Result::Err\s+d:\s*\n\s*btreemap_free_storage<\.K,\.V>\s+storage/, 'BTreeMap grow failure must not hide owner disposal inside storage helper');
assert.match(btreeMapSource, /keys\s+<Vec<Option<\.K>>>/, 'BTreeMap keys must use Vec<Option<K>> storage');
assert.match(btreeMapSource, /values\s+<Vec<Option<\.V>>>/, 'BTreeMap values must use Vec<Option<V>> storage');
assert.match(btreeMapStorageSource, /match\s+btreemap_key_at<\.K>/, 'BTreeMap storage must branch on Option key slots');
assert.match(btreeMapSearchSource, /fn\s+btreemap_key_eq\s+<\.K:\s*Ord&Copy>\s+<\(\.K,\.K\)->bool>/, 'BTreeMap key equality must remain Copy-only until borrowed key comparison exists');
assert.doesNotMatch(btreeMapSearchSource, /fn\s+btreemap_key_eq\s+<\.K:\s*Ord>\s+<\(\.K,\.K\)->bool>/, 'BTreeMap key equality must not accept non-Copy Ord keys by value');
assert.match(btreeMapStorageSource, /fn\s+btreemap_free_storage\s+<\.K:\s*Copy,\.V:\s*Copy>\s+<\(BTreeMapStorage<\.K,\.V>\)->\(\)>/, 'BTreeMap storage cleanup must remain Copy-only until OwnedBuffer element drop traversal exists');
assert.match(sourceWithoutComments(btreeMapCleanupFile), /fn\s+free\s+<\.K:\s*Copy,\.V:\s*Copy>\s+<\(BTreeMap<\.K,\.V>\)->\(\)>/, 'BTreeMap.free must expose the same Copy-only cleanup contract as its storage');

const btreeSetTypesSource = sourceWithoutComments(btreeSetTypesFile);
const btreeSetStorageSource = sourceWithoutComments(btreeSetStorageFile);
const btreeSetGrow = functionBlock(btreeSetStorageFile, 'btreeset_grow');
const btreeSetSearchSource = sourceWithoutComments(btreeSetSearchFile);
const btreeSetSource = [
    btreeSetRootSource,
    btreeSetTypesSource,
    btreeSetStorageSource,
    btreeSetSearchSource,
    btreeSetApiSource,
    sourceWithoutComments(btreeSetCreateFile),
    sourceWithoutComments(btreeSetObserverFile),
    sourceWithoutComments(btreeSetInsertFile),
    sourceWithoutComments(btreeSetRemoveFile),
    sourceWithoutComments(btreeSetCleanupFile),
    sourceWithoutComments('stdlib/alloc/collections/btreeset/alias.nepl'),
].join('\n');
assert.match(btreeSetSource, /struct BTreeSetStorage<\.T>:/, 'BTreeSet must keep typed storage wrapper');
assert.match(btreeSetSource, /struct BTreeSetInsertError<\.T>:[\s\S]*owner\s+<BTreeSet<\.T>>[\s\S]*diag\s+<Diag>/, 'BTreeSet insert failure must carry the consumed set owner and diagnostic');
assert.match(btreeSetGrow, /fn\s+btreeset_grow\s+<\.T:\s*Copy>\s+<\(BTreeSet<\.T>\)\*>Result<BTreeSet<\.T>,\s*BTreeSetInsertError<\.T>>>/, 'BTreeSet grow must return an owner-preserving insert error');
assert.match(btreeSetGrow, /Result::Err\s+d:[\s\S]*Result<BTreeSet<\.T>,\s*BTreeSetInsertError<\.T>>::Err\s+BTreeSetInsertError<\.T>\s+\(BTreeSet<\.T>\s+len0\s+cap0\s+storage\)\s+d/, 'BTreeSet grow allocation failure must return the original set owner through the typed Result variant');
assert.doesNotMatch(btreeSetGrow, /Result::Err\s+d:\s*\n\s*btreeset_free_storage<\.T>\s+storage/, 'BTreeSet grow failure must not hide owner disposal inside storage helper');
assert.match(btreeSetSource, /keys\s+<Vec<Option<\.T>>>/, 'BTreeSet keys must use Vec<Option<T>> storage');
assert.match(btreeSetStorageSource, /match\s+btreeset_key_at<\.T>/, 'BTreeSet storage must branch on Option key slots');
assert.match(btreeSetSearchSource, /fn\s+btreeset_key_eq\s+<\.T:\s*Ord&Copy>\s+<\(\.T,\.T\)->bool>/, 'BTreeSet key equality must remain Copy-only until borrowed key comparison exists');
assert.doesNotMatch(btreeSetSearchSource, /fn\s+btreeset_key_eq\s+<\.T:\s*Ord>\s+<\(\.T,\.T\)->bool>/, 'BTreeSet key equality must not accept non-Copy Ord keys by value');
assert.match(btreeSetStorageSource, /fn\s+btreeset_free_storage\s+<\.T:\s*Copy>\s+<\(BTreeSetStorage<\.T>\)->\(\)>/, 'BTreeSet storage cleanup must remain Copy-only until OwnedBuffer element drop traversal exists');
assert.match(sourceWithoutComments(btreeSetCleanupFile), /fn\s+free\s+<\.T:\s*Copy>\s+<\(BTreeSet<\.T>\)->\(\)>/, 'BTreeSet.free must expose the same Copy-only cleanup contract as its storage');

const rawStoragePatterns = [
    /\bMemPtr\b/,
    /\balloc_raw\b/,
    /\bdealloc_raw\b/,
    /\balloc_ptr\b/,
    /\brealloc_ptr\b/,
    /\bload_i32\b/,
    /\bstore_i32\b/,
    /\bmem_ptr_addr\b/,
    /\bhdr\s+<i32>/,
    /\bkeys_ptr\b/,
    /\bvalues_ptr\b/,
];

for (const [name, src] of [['BTreeMap', btreeMapSource], ['BTreeSet', btreeSetSource]]) {
    for (const pattern of rawStoragePatterns) {
        assert.doesNotMatch(src, pattern, `${name} must not return to raw header or raw pointer storage: ${pattern}`);
    }
}

console.log('btree insert grow unsafe unwrap regression passed');
