#!/usr/bin/env node

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');

const repoRoot = path.resolve(__dirname, '..');

const fixtures = [
    {
        relPath: 'tests/stdlib/selfhost_cliarg_parser.n.md',
        forbiddenPatterns: [
            ['Vec str cleanup postfix', /\bv::free<str>/],
        ],
    },
    {
        relPath: 'tests/stdlib/neplg2_lexer.n.md',
        forbiddenPatterns: [
            ['SelfhostToken Vec helper postfix', /\b(?:unwrap|get|len|free)<SelfhostToken>/],
        ],
    },
    {
        relPath: 'tests/stdlib/neplg2_type_arena.n.md',
        forbiddenPatterns: [
            ['SelfhostTypeId Vec helper postfix', /\b(?:new|push|vec_push_error_vec|vec_push_error_kind|free)<SelfhostTypeId>/],
        ],
    },
    {
        relPath: 'tests/stdlib/neplg2_diag_outcome.n.md',
        forbiddenPatterns: [
            ['selfhost outcome helper postfix', /\bselfhost_outcome_[A-Za-z0-9_]+<[^>\r\n]+>/],
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
    `NEPLg2.1 selfhost fixtures must not reintroduce selected generic postfixes:\n${violations.join('\n')}`,
);

console.log('NEPLg2.1 selfhost fixture postfix cleanup regression passed');
