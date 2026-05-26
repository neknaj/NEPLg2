#!/usr/bin/env node

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');

const repoRoot = path.resolve(__dirname, '..');

const fixtures = [
    {
        relPath: 'stdlib/kp/kpgraph.nepl',
        forbiddenPatterns: [
            ['BFS Vec API postfix', /\bv::(?:filled|get|replace|free)<i32>/],
            ['old Vec i32 prose type notation', /Vec<i32>/],
        ],
    },
    {
        relPath: 'tests/compiler/overload.n.md',
        forbiddenPatterns: [
            ['generic Vec constructor postfix in pair helper', /\bv::new<\.T>/],
            ['pair helper call postfix', /\bpair_with_empty<i32>/],
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
    `NEPLg2.1 kpgraph/overload fixtures must not reintroduce selected generic postfixes:\n${violations.join('\n')}`,
);

console.log('NEPLg2.1 kpgraph/overload postfix cleanup regression passed');
