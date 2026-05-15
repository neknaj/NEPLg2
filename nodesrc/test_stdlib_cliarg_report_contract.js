#!/usr/bin/env node

const assert = require('node:assert/strict');
const path = require('node:path');
const { parseFile } = require('./parser');

const repoRoot = path.resolve(__dirname, '..');
const file = path.join(repoRoot, 'stdlib', 'tests', 'cliarg.n.md');
const parsed = parseFile(file);

const reports = new Map([
    [0, 'cliarg_basic'],
    [3, 'cliarg_get_rejects_out_of_range'],
]);

assert.equal(parsed.doctests.length, 6, 'stdlib/tests/cliarg.n.md doctest count changed');

for (const [index, name] of reports.entries()) {
    const doctest = parsed.doctests[index];
    assert.equal(doctest.ret, null, `${name} must not use ret-only success reporting`);
    assert.equal(doctest.exit_code, 0, `${name} must pin exit_code: 0`);
    assert.match(
        doctest.stdout,
        new RegExp(`^test_report name="${name}" count=\\d+ failed=0\\n`),
        `${name} must pin canonical stdout report`,
    );
    assert.match(doctest.code, new RegExp(`test_report_new "${name}"`), `${name} must construct a named TestReport`);
    assert.match(doctest.code, /test_report_print_stdout\b/, `${name} must print the report`);
    assert.match(doctest.code, /test_report_exit_code\b/, `${name} must derive exit code from the shown report`);
}

for (const index of [1, 2]) {
    const doctest = parsed.doctests[index];
    assert.equal(doctest.ret, null, `cliarg IO doctest#${index + 1} must not use ret`);
    assert.ok(doctest.stdout && doctest.stdout.length > 0, `cliarg IO doctest#${index + 1} must keep stdout expectation`);
}

console.log('stdlib cliarg report contract passed');
