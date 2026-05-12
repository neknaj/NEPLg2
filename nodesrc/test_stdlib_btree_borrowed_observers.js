#!/usr/bin/env node

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');

const repoRoot = path.resolve(__dirname, '..');
const mapRootSrc = fs.readFileSync(path.join(repoRoot, 'stdlib/alloc/collections/btreemap.nepl'), 'utf8');
const mapApiSrc = fs.readFileSync(path.join(repoRoot, 'stdlib/alloc/collections/btreemap/api.nepl'), 'utf8');
const setRootSrc = fs.readFileSync(path.join(repoRoot, 'stdlib/alloc/collections/btreeset.nepl'), 'utf8');
const setApiSrc = fs.readFileSync(path.join(repoRoot, 'stdlib/alloc/collections/btreeset/api.nepl'), 'utf8');

const mapRootCode = stripComments(mapRootSrc);
const mapCode = stripComments(mapApiSrc);
const setRootCode = stripComments(setRootSrc);
const setCode = stripComments(setApiSrc);

assert.match(mapCode, /fn\s+len\s+<\.K,\.V>\s+<\(&BTreeMap<\.K,\.V>\)->i32>\s+\(hm\):/, 'BTreeMap.len must borrow the owner');
assert.match(mapCode, /fn\s+contains\s+<\.K:\s*Ord&Copy,\.V:\s*Copy>\s+<\(&BTreeMap<\.K,\.V>,\.K\)->bool>\s+\(hm,\s*key\):/, 'BTreeMap.contains must borrow the owner');
assert.match(mapCode, /fn\s+get\s+<\.K:\s*Ord&Copy,\.V:\s*Copy>\s+<\(&BTreeMap<\.K,\.V>,\.K\)->Option<\.V>>\s+\(hm,\s*key\):/, 'BTreeMap.get must borrow the owner');
assert.doesNotMatch(mapCode, /fn\s+(?:len_ref|contains_ref|get_ref)\b/, 'BTreeMap must not keep duplicate *_ref observers');
assert.doesNotMatch(mapCode, /fn\s+(?:len|contains|get)\s+<[^>]+>\s+<\(BTreeMap<\.K,\.V>/, 'BTreeMap read-only observers must not consume the owner');
assert.doesNotMatch(mapRootCode, /\bfn\s+/, 'BTreeMap root facade must not keep implementation bodies');
for (const submodule of ['types', 'api', 'alias']) {
    assert.match(
        mapRootCode,
        new RegExp(`pub\\s+#import\\s+"\\.\\/btreemap\\/${submodule}"\\s+as\\s+@merge`),
        `BTreeMap root facade must re-export btreemap/${submodule}`,
    );
}

assert.match(setCode, /fn\s+len\s+<\.T>\s+<\(&BTreeSet<\.T>\)->i32>\s+\(set0\):/, 'BTreeSet.len must borrow the owner');
assert.match(setCode, /fn\s+contains\s+<\.T:\s*Ord&Copy>\s+<\(&BTreeSet<\.T>,\.T\)->bool>\s+\(set0,\s*key\):/, 'BTreeSet.contains must borrow the owner');
assert.doesNotMatch(setCode, /fn\s+(?:len_ref|contains_ref)\b/, 'BTreeSet must not keep duplicate *_ref observers');
assert.doesNotMatch(setCode, /fn\s+(?:len|contains)\s+<[^>]+>\s+<\(BTreeSet<\.T>/, 'BTreeSet read-only observers must not consume the owner');
assert.doesNotMatch(setRootCode, /\bfn\s+/, 'BTreeSet root facade must not keep implementation bodies');
for (const submodule of ['types', 'api', 'alias']) {
    assert.match(
        setRootCode,
        new RegExp(`pub\\s+#import\\s+"\\.\\/btreeset\\/${submodule}"\\s+as\\s+@merge`),
        `BTreeSet root facade must re-export btreeset/${submodule}`,
    );
}

for (const testPath of [
    'stdlib/tests/btreemap.n.md',
    'stdlib/tests/btreeset.n.md',
    'tests/stdlib/pipe_collections.n.md',
]) {
    const src = fs.readFileSync(path.join(repoRoot, testPath), 'utf8');
    assert.doesNotMatch(src, /\b(?:len_ref|contains_ref|get_ref)<i32(?:,i32)?>/, `${testPath} must not use removed BTree *_ref observers`);
    assert.doesNotMatch(src, /\b(?:len|contains|get)<i32(?:,i32)?>\s+(?:m|m[0-9]|s|s[0-9])\b/, `${testPath} must not call BTree observers by value`);
}

const costFixture = fs.readFileSync(path.join(repoRoot, 'tests/stdlib/btree_array_cost.n.md'), 'utf8');
assert.doesNotMatch(costFixture, /\bsorted_array_(?:map|set)_(?:len|get|contains)<[^>]+>\s+(?:m|s)\b/, 'btree_array_cost must not call sorted-array BTree observers by value');
assert.match(costFixture, /\bsorted_array_(?:map|set)_(?:len|get|contains)<[^>]+>\s+&(?:m|s)\b/, 'btree_array_cost must exercise borrowed sorted-array observer aliases');

console.log('btree borrowed observer regression passed');

function stripComments(src) {
    return src
        .split(/\r?\n/)
        .filter((line) => !/^\s*\/\//.test(line))
        .join('\n');
}
