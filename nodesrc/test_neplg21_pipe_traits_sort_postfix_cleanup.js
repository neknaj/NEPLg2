#!/usr/bin/env node

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');

const repoRoot = path.resolve(__dirname, '..');

const fixtures = [
    {
        relPath: 'tests/stdlib/pipe_collections.n.md',
        forbiddenPatterns: [
            ['BTreeMap insert error helper postfix', /\bbtreemap_insert_error_(?:diag|owner)<[^>\r\n]+>/],
            ['BTreeSet insert error helper postfix', /\bbtreeset_insert_error_(?:diag|owner)<[^>\r\n]+>/],
            ['HashMap update error owner postfix', /\bhashmap_update_error_owner<[^>\r\n]+>/],
            ['HashSet update error owner postfix', /\bhashset_update_error_owner<[^>\r\n]+>/],
        ],
    },
    {
        relPath: 'tests/stdlib/traits_order.n.md',
        forbiddenPatterns: [
            ['Vec i32 observer/cleanup postfix', /\b(?:get|free)<i32>/],
        ],
    },
    {
        relPath: 'tests/stdlib/sort.n.md',
        forbiddenPatterns: [
            ['VecSortError constructor postfix', /\bVecSortError<i32>/],
        ],
    },
    {
        relPath: 'tutorials/getting_started/02_test_harness.n.md',
        forbiddenPatterns: [
            ['old Result unit str prose type notation', /Result<unit,str>/],
        ],
    },
    {
        relPath: 'tutorials/getting_started/91_sort_search_prefixsum.n.md',
        forbiddenPatterns: [
            ['old Vec i32 prose type notation', /Vec<i32>/],
        ],
    },
    {
        relPath: 'tests/stdlib/selfhost_req.n.md',
        forbiddenPatterns: [
            ['old Vec u8 prose type notation', /Vec<u8>/],
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
    `NEPLg2.1 pipe/traits/sort fixtures must not reintroduce selected generic postfixes:\n${violations.join('\n')}`,
);

console.log('NEPLg2.1 pipe/traits/sort postfix cleanup regression passed');
