#!/usr/bin/env node

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');

const repoRoot = path.resolve(__dirname, '..');

const fixtures = [
    {
        relPath: 'tests/stdlib/collection_cleanup_contract.n.md',
        forbiddenPatterns: [
            ['Vec empty metadata postfix', /\bvec_empty<CleanupPayload>/],
            ['Vec empty observer postfix', /\bis_empty<CleanupPayload>/],
            ['Vec metadata observer postfix', /\b(?:len|cap|vec_partition_matched_len)<CleanupPayload>/],
            ['BTreeMap cleanup metadata len postfix', /\blen<i32,\s*CleanupPayload>/],
            ['HashMap cleanup metadata len postfix', /\blen<i32,\s*CleanupPayload,\s*DefaultHash32>/],
        ],
    },
    {
        relPath: 'tests/stdlib/traits_serde.n.md',
        forbiddenPatterns: [
            ['Deserialize primitive postfix', /\bdeserialize<(?:i32|bool)>/],
        ],
    },
    {
        relPath: 'tests/stdlib/traits_hash.n.md',
        forbiddenPatterns: [
            ['Hasher use helper postfix', /\buse_hasher_twice<i32,\s*StatefulHasher>/],
            ['old Hasher prose type notation', /Hasher<\.K>/, 'nonDeclaration'],
        ],
    },
    {
        relPath: 'tests/compiler/generic_impl_trait_args.n.md',
        forbiddenPatterns: [
            ['old Hasher prose type notation', /Hasher<\.K>/, 'nonDeclaration'],
            ['old Trait prose type notation', /Trait<\.T>/, 'nonDeclaration'],
        ],
    },
    {
        relPath: 'tests/compiler/prelude_copy.n.md',
        forbiddenPatterns: [
            ['old MemPtr prose type notation', /MemPtr<(?:\.T|i32)>/, 'nonDeclaration'],
        ],
    },
    {
        relPath: 'tests/compiler/move_effect.n.md',
        forbiddenPatterns: [
            ['old owner token prose type notation', /(?:RegionToken|MemPtr)<T>/, 'nonDeclaration'],
        ],
    },
    {
        relPath: 'tests/compiler/typeannot.n.md',
        forbiddenPatterns: [
            ['old Option prose type notation', /Option<i32>/, 'nonDeclaration'],
        ],
    },
];

const violations = [];

for (const fixture of fixtures) {
    const filePath = path.join(repoRoot, fixture.relPath);
    const text = fs.readFileSync(filePath, 'utf8');
    const lines = text.split(/\r?\n/);
    lines.forEach((line, index) => {
        const trimmed = line.trim();
        for (const [label, pattern, mode] of fixture.forbiddenPatterns) {
            if (mode === 'nonDeclaration' && /^(pub\s+)?(fn|impl|trait|struct|enum)\b/.test(trimmed)) {
                continue;
            }
            if (pattern.test(line)) {
                violations.push(`${fixture.relPath}:${index + 1}: ${label}: ${line.trim()}`);
            }
        }
    });
}

assert.deepEqual(
    violations,
    [],
    `NEPLg2.1 metadata/traits fixtures must not reintroduce selected generic postfixes:\n${violations.join('\n')}`,
);

console.log('NEPLg2.1 metadata/traits postfix cleanup regression passed');
