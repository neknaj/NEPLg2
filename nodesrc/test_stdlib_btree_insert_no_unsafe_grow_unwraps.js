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

const btreeMapInsert = functionBlock('stdlib/alloc/collections/btreemap.nepl', 'insert');
assert.match(btreeMapInsert, /match\s+grow<\.K,\.V>\s+hm:/, 'BTreeMap.insert must match grow result');
assert.match(btreeMapInsert, /Result::Err\s+d:/, 'BTreeMap.insert must keep an Err arm');
assert.match(btreeMapInsert, /err<BTreeMap<\.K,\.V>,\s*Diag>\s+d/, 'BTreeMap.insert must return grow Err');
assertNoUnsafeUnwraps('stdlib/alloc/collections/btreemap.nepl', ['insert', 'btreemap_insert_ready']);

const btreeSetInsert = functionBlock('stdlib/alloc/collections/btreeset.nepl', 'insert');
assert.match(btreeSetInsert, /match\s+btreeset_grow<\.T>\s+set0:/, 'BTreeSet.insert must match grow result');
assert.match(btreeSetInsert, /Result::Err\s+d:/, 'BTreeSet.insert must keep an Err arm');
assert.match(btreeSetInsert, /err<BTreeSet<\.T>,\s*Diag>\s+d/, 'BTreeSet.insert must return grow Err');
assertNoUnsafeUnwraps('stdlib/alloc/collections/btreeset.nepl', ['insert', 'btreeset_insert_ready']);

console.log('btree insert grow unsafe unwrap regression passed');
