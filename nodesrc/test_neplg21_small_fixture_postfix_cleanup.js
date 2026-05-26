#!/usr/bin/env node

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const { parseFile } = require('./parser');

const repoRoot = path.resolve(__dirname, '..');

const compilerFixtures = [
    {
        relPath: 'tests/compiler/list_dot_map.n.md',
        forbiddenCodePatterns: [
            ['list new generic postfix', /\blist::new<[^>\r\n]+>/],
            ['list push generic postfix', /\blist::push<[^>\r\n]+>/],
            ['list map generic postfix', /\blist::map<[^>\r\n]+>/],
            ['result map generic postfix', /\bmap<[^>\r\n]+>/],
            ['vec new generic postfix', /\bnew<i32>/],
            ['vec push generic postfix', /\bpush<i32>/],
            ['namespaced type annotation', /%[A-Za-z_][A-Za-z0-9_]*::[A-Za-z_]/],
        ],
    },
    {
        relPath: 'tests/compiler/overload_nested_generic_push.n.md',
        forbiddenCodePatterns: [
            ['nested new generic postfix', /\bnew<Result<unit,str>>/],
            ['nested len generic postfix', /\blen<Result<unit,str>>/],
            ['nested free generic postfix', /\bfree<Result<unit,str>>/],
        ],
        forbiddenTextPatterns: [
            ['old nested Vec type notation', /Vec<Result<unit,str>>/],
            ['old nested Result type notation', /Result<unit,str>/],
        ],
    },
];

for (const fixture of compilerFixtures) {
    const filePath = path.join(repoRoot, fixture.relPath);
    const text = fs.readFileSync(filePath, 'utf8');
    const parsed = parseFile(filePath);
    const code = parsed.doctests.map((doctest) => doctest.code).join('\n');

    for (const [label, pattern] of fixture.forbiddenCodePatterns) {
        assert.doesNotMatch(
            code,
            pattern,
            `${fixture.relPath} doctests must not contain ${label}`,
        );
    }

    for (const [label, pattern] of fixture.forbiddenTextPatterns || []) {
        assert.doesNotMatch(
            text,
            pattern,
            `${fixture.relPath} prose and doctests must not contain ${label}`,
        );
    }
}

console.log('NEPLg2.1 small fixture postfix cleanup regression passed');
