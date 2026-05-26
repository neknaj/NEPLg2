#!/usr/bin/env node

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const { parseFile } = require('./parser');

const repoRoot = path.resolve(__dirname, '..');
const suitePath = path.join(repoRoot, 'tests', 'stdlib', 'selfhost_cliarg_parser.n.md');
const source = fs.readFileSync(suitePath, 'utf8');
const parsed = parseFile(suitePath);

assert.equal(
    parsed.doctests.length,
    1,
    'selfhost CLI arg parser suite must keep one aggregated doctest to avoid recompiling the same dependency graph',
);

const doctest = parsed.doctests[0];
assert.deepEqual(
    doctest.tags,
    ['stdio', 'normalize_newlines'],
    'aggregated selfhost CLI arg parser doctest must normalize stdout report output',
);
assert.equal(doctest.exit_code, 0, 'aggregated selfhost CLI arg parser doctest must use exit_code metadata');
assert.equal(doctest.ret, null, 'aggregated selfhost CLI arg parser doctest must not use ret as test status');
assert.match(
    doctest.stdout,
    /^test_report name="selfhost_cliarg_parser" count=10 failed=0\n/,
    'aggregated selfhost CLI arg parser doctest must publish all parser checks to stdout',
);
assert.match(
    doctest.code,
    /\btest_report_print_stdout\b[\s\S]*\btest_report_exit_code\b/,
    'aggregated selfhost CLI arg parser doctest must separate stdout reporting from exit-code conversion',
);
assert.doesNotMatch(
    doctest.code,
    /\bv::(?:new|push)<str>/,
    'aggregated selfhost CLI arg parser doctest must rely on Vec str expected type or receiver evidence instead of explicit producer or mutator postfixes',
);
assert.doesNotMatch(
    source,
    /neplg2:test\s*\r?\nret:/,
    'selfhost CLI arg parser doctests must not regress to ret-only status checks',
);

const assertionLines = doctest.stdout
    .split(/\r?\n/)
    .filter((line) => line.startsWith('assertion index='));
assert.equal(
    assertionLines.length,
    10,
    'selfhost CLI arg parser report must keep one assertion line for each parser behavior',
);

console.log('selfhost cliarg parser doctest contract ok');
