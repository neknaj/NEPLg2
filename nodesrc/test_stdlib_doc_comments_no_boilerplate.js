#!/usr/bin/env node

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');

const repoRoot = path.resolve(__dirname, '..');

const targetFiles = [
    'stdlib/core/field.nepl',
    'stdlib/nm/html_gen.nepl',
    'stdlib/core/rand/xorshift32.nepl',
    'stdlib/alloc/hash/fnv1a32.nepl',
    'stdlib/alloc/collections/vec/sort.nepl',
];

const forbiddenPhrases = [
    ['generic main-use title', '\u4e3b\u306a\u7528\u9014'],
    ['predefined-process placeholder', '\u5b9a\u7fa9\u6e08\u307f\u51e6\u7406'],
    ['thin-wrapper placeholder', '\u8584\u3044\u30e9\u30c3\u30d1'],
    ['move-and-rebind placeholder', '\u518d\u5229\u7528\u6642\u306f\u675f\u7e1b\u3057\u76f4'],
    ['body-processing placeholder', '\u672c\u4f53\u51e6\u7406\u306b\u6e96\u3058\u307e\u3059'],
];

for (const relPath of targetFiles) {
    const src = fs.readFileSync(path.join(repoRoot, relPath), 'utf8');
    for (const [label, phrase] of forbiddenPhrases) {
        assert.equal(
            src.includes(phrase),
            false,
            `${relPath} must not contain generated doc boilerplate: ${label}`,
        );
    }
}

const requiredPhrasesByFile = [
    [
        'stdlib/core/field.nepl',
        [
            ['owned field move contract', 'get: field value \u3092\u6240\u6709\u5024\u3068\u3057\u3066\u53d6\u308a\u51fa\u3059'],
            ['borrowed field contract', 'aggregate owner \u3092\u6d88\u8cbb\u3057\u306a\u3044\u305f\u3081\u306b\u4f7f\u3044\u307e\u3059'],
            ['field overwrite contract', 'put: aggregate field \u3092\u66f8\u304d\u63db\u3048\u308b'],
        ],
    ],
    [
        'stdlib/nm/html_gen.nepl',
        [
            ['HTML escape scope', 'URL sanitizer \u3084 CSS sanitizer \u3067\u306f\u3042\u308a\u307e\u305b\u3093'],
            ['direct serializer contract', '`Vec<Node>` \u3092\u69cb\u7bc9\u305b\u305a'],
            ['code fence escape contract', 'code fence body \u306f inline \u5909\u63db\u305b\u305a'],
        ],
    ],
    [
        'stdlib/core/rand/xorshift32.nepl',
        [
            ['seed zero contract', 'seed 0 \u306f Xorshift \u306e\u56fa\u5b9a\u70b9'],
            ['xorshift formula', 'x ^= x << 13; x ^= x >> 17; x ^= x << 5'],
            ['non-crypto warning', '\u6697\u53f7\u8ad6\u7684\u5b89\u5168\u6027\u306f\u3042\u308a\u307e\u305b\u3093'],
        ],
    ],
    [
        'stdlib/alloc/hash/fnv1a32.nepl',
        [
            ['offset basis contract', 'FNV-1a offset basis'],
            ['update rule contract', '(hash xor byte) * 16777619'],
            ['collision/security warning', 'collision resistance \u3084 DoS \u8010\u6027\u306f\u3042\u308a\u307e\u305b\u3093'],
        ],
    ],
    [
        'stdlib/alloc/collections/vec/sort.nepl',
        [
            ['sort family contract', '`Vec` \u3068 raw slice \u306e in-place sort algorithms'],
            ['quick sort caveat', 'Lomuto quick sort'],
            ['merge sort stability', 'stable sort \u304c\u5fc5\u8981\u306a\u3089 `sort_merge`'],
            ['heap sort complexity', 'O(n log n)\u3001\u8ffd\u52a0\u9818\u57df O(1)'],
            ['raw helper boundary contract', '`0 <= idx < len(v)` \u3092\u547c\u3073\u51fa\u3057\u5074\u304c\u4fdd\u8a3c'],
        ],
    ],
];

for (const [relPath, requiredPhrases] of requiredPhrasesByFile) {
    const src = fs.readFileSync(path.join(repoRoot, relPath), 'utf8');
    for (const [label, phrase] of requiredPhrases) {
        assert.equal(src.includes(phrase), true, `${relPath} must document ${label}`);
    }
}

console.log('stdlib doc comment boilerplate regression passed');
