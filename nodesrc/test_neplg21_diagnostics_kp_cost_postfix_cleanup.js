#!/usr/bin/env node

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');

const repoRoot = path.resolve(__dirname, '..');

const fixtures = [
    {
        relPath: 'tests/stdlib/btree_array_cost.n.md',
        forbiddenPatterns: [
            ['sorted array BTreeMap helper postfix', /\bsorted_array_map_(?:new|insert|len|get|free)<[^>\r\n]+>/],
            ['sorted array BTreeSet helper postfix', /\bsorted_array_set_(?:new|insert|len|contains|free)<[^>\r\n]+>/],
        ],
    },
    {
        relPath: 'tests/stdlib/capacity_stack.n.md',
        forbiddenPatterns: [
            ['Vec i32 or Kind helper postfix', /\b(?:new|push|len|free)<(?:i32|Kind)>/],
        ],
    },
    {
        relPath: 'tests/stdlib/collections_diag.n.md',
        forbiddenPatterns: [
            ['Queue or RingBuffer diagnostic helper postfix', /\b(?:new|pop)<i32>/],
        ],
    },
    {
        relPath: 'tests/stdlib/kp.n.md',
        forbiddenPatterns: [
            ['KP Vec helper postfix', /\b(?:filled|with_capacity|get|replace|len|free)<i32>/],
        ],
    },
    {
        relPath: 'stdlib/kp/kpsearch.nepl',
        forbiddenPatterns: [
            ['KP search Vec helper postfix', /\b(?:with_capacity|get|replace|len|free)<i32>/],
        ],
    },
    {
        relPath: 'stdlib/kp/kpprefix.nepl',
        forbiddenPatterns: [
            ['KP prefix Vec helper postfix', /\b(?:v::(?:new|push)|vec::(?:get|replace|len|filled|free))<i32>/],
        ],
    },
    {
        relPath: 'stdlib/alloc/diag/diag.nepl',
        forbiddenPatterns: [
            ['Diag Vec observer postfix', /\bv::(?:len|get)<Diag>/],
        ],
    },
    {
        relPath: 'stdlib/alloc/diag/error/diags.nepl',
        forbiddenPatterns: [
            ['Diags Vec helper postfix', /\bv::(?:vec_empty|new|push|free|vec_push_error_vec|len|get)<Diag>/],
        ],
    },
    {
        relPath: 'stdlib/alloc/diag/error/outcome.nepl',
        forbiddenPatterns: [
            ['Outcome doctest constructor postfix', /\boutcome_ok<[^>\r\n]+>/],
        ],
    },
    {
        relPath: 'tests/compiler/neplg2.n.md',
        forbiddenPatterns: [
            ['List helper postfix in compiler fixture', /\b(?:new|cons|get|free)<i32>/],
        ],
    },
    {
        relPath: 'tests/compiler/overload.n.md',
        forbiddenPatterns: [
            ['Vec helper postfix in overload fixture', /\bv::(?:new|push)<i32>/],
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
    `NEPLg2.1 diagnostics/KP/cost fixtures must not reintroduce selected generic postfixes:\n${violations.join('\n')}`,
);

console.log('NEPLg2.1 diagnostics/KP/cost postfix cleanup regression passed');
