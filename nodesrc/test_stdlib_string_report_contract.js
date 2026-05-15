#!/usr/bin/env node

const assert = require('node:assert/strict');
const path = require('node:path');
const { parseFile } = require('./parser');

const repoRoot = path.resolve(__dirname, '..');
const file = path.join(repoRoot, 'stdlib', 'tests', 'string.n.md');
const parsed = parseFile(file);

const expectedReports = [
    'string_len_and_concat',
    'string_trim_and_slice',
    'string_split_and_builder',
    'string_byte_at',
    'string_find_byte_index',
    'string_result_allocation_apis',
    'string_utf8_mem_result',
    'string_to_f64_parser',
    'string_slice_utf8_boundary',
];

assert.equal(parsed.doctests.length, expectedReports.length, 'stdlib/tests/string.n.md doctest count changed');

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
}

console.log('stdlib string report contract passed');
