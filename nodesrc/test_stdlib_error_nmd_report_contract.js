#!/usr/bin/env node

const assert = require('node:assert/strict');
const path = require('node:path');
const { parseFile } = require('./parser');

const repoRoot = path.resolve(__dirname, '..');
const file = path.join(repoRoot, 'stdlib', 'tests', 'error.n.md');
const parsed = parseFile(file);

const expectedReports = [
    ['std_error_kind_and_diag_value_model', 8],
    ['outcome_helpers_keep_result_and_diags_separate', 14],
    ['result_and_outcome_common_helpers', 8],
];

assert.equal(parsed.doctests.length, expectedReports.length, 'stdlib/tests/error.n.md doctest count changed');

for (const [index, [name, count]] of expectedReports.entries()) {
    const doctest = parsed.doctests[index];
    assert.equal(doctest.ret, null, `${name} must not use ret-only success reporting`);
    assert.equal(doctest.exit_code, 0, `${name} must pin exit_code: 0`);
    assert.match(
        doctest.stdout,
        new RegExp(`^test_report name="${name}" count=${count} failed=0\\n`),
        `${name} must pin canonical stdout report and assertion count`,
    );
    assert.match(doctest.code, new RegExp(`test_report_new "${name}"`), `${name} must construct a named TestReport`);
    assert.match(doctest.code, /test_report_print_stdout\b/, `${name} must print the report`);
    assert.match(doctest.code, /test_report_exit_code\b/, `${name} must derive exit code from the shown report`);
    assert.doesNotMatch(doctest.code, /\bchecks_exit_code\b/, `${name} must not hide report details behind checks_exit_code`);
    assert.doesNotMatch(doctest.code, /\bchecks_print_report\b/, `${name} must not use legacy Checks report output`);
    assert.doesNotMatch(doctest.code, /\bchecks_new\b/, `${name} must not use legacy Checks construction`);
}

assert.match(parsed.doctests[0].stdout, /label="diag kind string"/, 'Diag kind assertion label must be pinned');
assert.match(parsed.doctests[0].stdout, /label="span file id"/, 'Diag span assertion label must be pinned');
assert.match(parsed.doctests[1].stdout, /label="err0 kind"/, 'Outcome IoError assertion label must be pinned');
assert.match(parsed.doctests[1].stdout, /label="err1 kind"/, 'Outcome ParseError assertion label must be pinned');
assert.match(parsed.doctests[1].code, /\bmatch kind:\s+StdErrorKind::IoError:/m, 'IoError branch must stay enum-match checked');
assert.match(parsed.doctests[1].code, /\bmatch kind:\s+StdErrorKind::ParseError:/m, 'ParseError branch must stay enum-match checked');
assert.match(parsed.doctests[2].stdout, /label="o2 diags length"/, 'Result-like Outcome diag assertion label must be pinned');

console.log('stdlib error.n.md report contract passed');
