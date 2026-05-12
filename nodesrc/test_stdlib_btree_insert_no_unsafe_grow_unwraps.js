#!/usr/bin/env node

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');

const repoRoot = path.resolve(__dirname, '..');

function functionBlock(file, name) {
    const src = fs.readFileSync(path.join(repoRoot, file), 'utf8');
    const lines = src.split(/\r?\n/);
    const start = lines.findIndex((line) => line.startsWith(`fn ${name} `));
    assert.notEqual(start, -1, `${name} must exist in ${file}`);

    let end = lines.length;
    for (let i = start + 1; i < lines.length; i += 1) {
        if (lines[i].startsWith('fn ')) {
            end = i;
            break;
        }
    }

    return lines
        .slice(start, end)
        .filter((line) => !/^\s*\/\//.test(line))
        .join('\n');
}

function sourceWithoutComments(file) {
    const src = fs.readFileSync(path.join(repoRoot, file), 'utf8');
    return src
        .split(/\r?\n/)
        .filter((line) => !/^\s*\/\//.test(line))
        .join('\n');
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
const btreeMapTypesFile = 'stdlib/alloc/collections/btreemap/types.nepl';
const btreeMapStorageFile = 'stdlib/alloc/collections/btreemap/storage.nepl';
const btreeMapRootSource = sourceWithoutComments('stdlib/alloc/collections/btreemap.nepl');
assert.doesNotMatch(btreeMapRootSource, /\bfn\s+/, 'BTreeMap root facade must not keep implementation bodies');
for (const submodule of ['types', 'api', 'alias']) {
    assert.match(
        btreeMapRootSource,
        new RegExp(`pub\\s+#import\\s+"\\.\\/btreemap\\/${submodule}"\\s+as\\s+@merge`),
        `BTreeMap root facade must re-export btreemap/${submodule}`,
    );
}

const btreeMapInsert = functionBlock(btreeMapApiFile, 'insert');
assert.match(btreeMapInsert, /match\s+btreemap_grow<\.K,\.V>\s+hm:/, 'BTreeMap.insert must match grow result');
assert.match(btreeMapInsert, /Result::Err\s+d:/, 'BTreeMap.insert must keep an Err arm');
assert.match(btreeMapInsert, /err<BTreeMap<\.K,\.V>,\s*Diag>\s+d/, 'BTreeMap.insert must return grow Err');
assertNoUnsafeUnwraps(btreeMapApiFile, ['insert', 'btreemap_insert_ready']);

const btreeSetApiFile = 'stdlib/alloc/collections/btreeset/api.nepl';
const btreeSetTypesFile = 'stdlib/alloc/collections/btreeset/types.nepl';
const btreeSetStorageFile = 'stdlib/alloc/collections/btreeset/storage.nepl';
const btreeSetRootSource = sourceWithoutComments('stdlib/alloc/collections/btreeset.nepl');
assert.doesNotMatch(btreeSetRootSource, /\bfn\s+/, 'BTreeSet root facade must not keep implementation bodies');
for (const submodule of ['types', 'api', 'alias']) {
    assert.match(
        btreeSetRootSource,
        new RegExp(`pub\\s+#import\\s+"\\.\\/btreeset\\/${submodule}"\\s+as\\s+@merge`),
        `BTreeSet root facade must re-export btreeset/${submodule}`,
    );
}

const btreeSetInsert = functionBlock(btreeSetApiFile, 'insert');
assert.match(btreeSetInsert, /match\s+btreeset_grow<\.T>\s+set0:/, 'BTreeSet.insert must match grow result');
assert.match(btreeSetInsert, /Result::Err\s+d:/, 'BTreeSet.insert must keep an Err arm');
assert.match(btreeSetInsert, /err<BTreeSet<\.T>,\s*Diag>\s+d/, 'BTreeSet.insert must return grow Err');
assertNoUnsafeUnwraps(btreeSetApiFile, ['insert', 'btreeset_insert_ready']);

const btreeMapTypesSource = sourceWithoutComments(btreeMapTypesFile);
const btreeMapStorageSource = sourceWithoutComments(btreeMapStorageFile);
const btreeMapSource = [
    btreeMapRootSource,
    btreeMapTypesSource,
    btreeMapStorageSource,
    sourceWithoutComments('stdlib/alloc/collections/btreemap/search.nepl'),
    sourceWithoutComments(btreeMapApiFile),
    sourceWithoutComments('stdlib/alloc/collections/btreemap/alias.nepl'),
].join('\n');
assert.match(btreeMapSource, /struct BTreeMapStorage<\.K,\.V>:/, 'BTreeMap must keep typed storage wrapper');
assert.match(btreeMapSource, /keys\s+<Vec<Option<\.K>>>/, 'BTreeMap keys must use Vec<Option<K>> storage');
assert.match(btreeMapSource, /values\s+<Vec<Option<\.V>>>/, 'BTreeMap values must use Vec<Option<V>> storage');
assert.match(btreeMapStorageSource, /match\s+btreemap_key_at<\.K>/, 'BTreeMap storage must branch on Option key slots');

const btreeSetTypesSource = sourceWithoutComments(btreeSetTypesFile);
const btreeSetStorageSource = sourceWithoutComments(btreeSetStorageFile);
const btreeSetSource = [
    btreeSetRootSource,
    btreeSetTypesSource,
    btreeSetStorageSource,
    sourceWithoutComments('stdlib/alloc/collections/btreeset/search.nepl'),
    sourceWithoutComments(btreeSetApiFile),
    sourceWithoutComments('stdlib/alloc/collections/btreeset/alias.nepl'),
].join('\n');
assert.match(btreeSetSource, /struct BTreeSetStorage<\.T>:/, 'BTreeSet must keep typed storage wrapper');
assert.match(btreeSetSource, /keys\s+<Vec<Option<\.T>>>/, 'BTreeSet keys must use Vec<Option<T>> storage');
assert.match(btreeSetStorageSource, /match\s+btreeset_key_at<\.T>/, 'BTreeSet storage must branch on Option key slots');

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
