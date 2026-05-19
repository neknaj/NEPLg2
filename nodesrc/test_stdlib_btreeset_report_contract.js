#!/usr/bin/env node

const assert = require('node:assert/strict');
const path = require('node:path');
const { parseFile } = require('./parser');

const repoRoot = path.resolve(__dirname, '..');
const file = path.join(repoRoot, 'stdlib', 'tests', 'btreeset.n.md');
const parsed = parseFile(file);

const expectedReports = [
    'btreeset_insert_and_len',
    'btreeset_insert_growth_boundary',
    'btreeset_contains_and_remove',
    'btreeset_duplicate_insert',
    'btreeset_borrowed_reads_keep_owner',
];

assert.equal(parsed.doctests.length, expectedReports.length, 'stdlib/tests/btreeset.n.md doctest count changed');

for (const [index, name] of expectedReports.entries()) {
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
    assert.doesNotMatch(doctest.code, /\bchecks_exit_code\b/, `${name} must not hide report details behind checks_exit_code`);
    assert.doesNotMatch(doctest.code, /\bchecks_print_report\b/, `${name} must not use legacy Checks report output`);
    assert.doesNotMatch(doctest.code, /\bchecks_new\b/, `${name} must not use legacy Checks construction`);
}

console.log('stdlib btreeset report contract passed');
