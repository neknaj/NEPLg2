#!/usr/bin/env node

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');

const repoRoot = path.resolve(__dirname, '..');

const fixtures = [
    {
        relPath: 'tests/stdlib/selfhost_cli_driver.n.md',
        forbiddenPatterns: [
            ['Vec str cleanup postfix', /\bv::free<str>/],
        ],
    },
    {
        relPath: 'tests/stdlib/neplg2_module_loader.n.md',
        forbiddenPatterns: [
            ['SelfhostModuleItem unwrap postfix', /\bunwrap<SelfhostModuleItem>/],
        ],
    },
    {
        relPath: 'tests/stdlib/neplg2_parser.n.md',
        forbiddenPatterns: [
            ['SelfhostModuleItem unwrap postfix', /\bunwrap<SelfhostModuleItem>/],
        ],
    },
    {
        relPath: 'tests/stdlib/neplg2_module_graph.n.md',
        forbiddenPatterns: [
            ['SelfhostModuleGraphEdge unwrap postfix', /\bunwrap<SelfhostModuleGraphEdge>/],
        ],
    },
    {
        relPath: 'tests/stdlib/neplg2_stdlib_map.n.md',
        forbiddenPatterns: [
            ['SelfhostModuleGraphEdge unwrap postfix', /\bunwrap<SelfhostModuleGraphEdge>/],
        ],
    },
    {
        relPath: 'tests/stdlib/selfhost_req.n.md',
        forbiddenPatterns: [
            ['u8 Vec helper postfix', /\b(?:get|free)<u8>/],
            ['HashMap update error owner postfix', /\bhashmap_update_error_owner<[^>\r\n]+>/],
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
    `NEPLg2.1 selfhost simple fixtures must not reintroduce selected generic postfixes:\n${violations.join('\n')}`,
);

console.log('NEPLg2.1 selfhost simple postfix cleanup regression passed');
