#!/usr/bin/env node

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');

const repoRoot = path.resolve(__dirname, '..');

const fixtures = [
    {
        relPath: 'stdlib/tests/hashmap.n.md',
        forbiddenPatterns: [
            ['i32 HashMap update owner postfix', /\bhashmap_update_error_owner<i32,i32,DefaultHash32>/],
        ],
    },
    {
        relPath: 'stdlib/tests/hashmap_str.n.md',
        forbiddenPatterns: [
            ['str HashMap update owner postfix', /\bhashmap_update_error_owner<str,i32,DefaultHash32>/],
        ],
    },
    {
        relPath: 'stdlib/tests/error.n.md',
        forbiddenPatterns: [
            ['Outcome helper postfix', /\b(?:outcome_ok|outcome_err|result_to_outcome)<[^>\r\n]+>/],
        ],
    },
    {
        relPath: 'stdlib/tests/bloom_filter.n.md',
        forbiddenPatterns: [
            ['BloomFilter contains postfix', /\bcontains<i32,\s*DefaultHash32>/],
        ],
    },
    {
        relPath: 'stdlib/tests/string.n.md',
        forbiddenPatterns: [
            ['old Vec str prose type notation', /Vec<str>/],
        ],
    },
];

const violations = [];

for (const fixture of fixtures) {
    const filePath = path.join(repoRoot, fixture.relPath);
    const text = fs.readFileSync(filePath, 'utf8');
    const lines = text.split(/\r?\n/);
    lines.forEach((line, index) => {
        for (const [label, pattern] of fixture.forbiddenPatterns) {
            if (pattern.test(line)) {
                violations.push(`${fixture.relPath}:${index + 1}: ${label}: ${line.trim()}`);
            }
        }
    });
}

assert.deepEqual(
    violations,
    [],
    `NEPLg2.1 stdlib small fixtures must not reintroduce selected generic postfixes:\n${violations.join('\n')}`,
);

console.log('NEPLg2.1 stdlib small fixture postfix cleanup regression passed');
