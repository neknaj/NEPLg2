#!/usr/bin/env node

const assert = require('node:assert/strict');
const path = require('node:path');
const { parseFile } = require('./parser');

const repoRoot = path.resolve(__dirname, '..');
const file = path.join(repoRoot, 'stdlib', 'tests', 'fs.n.md');
const parsed = parseFile(file);

assert.equal(parsed.doctests.length, 1, 'stdlib/tests/fs.n.md doctest count changed');

const doctest = parsed.doctests[0];
assert.equal(doctest.ret, null, 'fs_main must not use ret-only success reporting');
assert.equal(doctest.exit_code, 0, 'fs_main must pin exit_code: 0');
assert.match(
    doctest.stdout,
    /^test_report name="fs_main" count=1 failed=0\n/,
    'fs_main must pin canonical stdout report',
);
assert.match(doctest.stdout, /label="missing file returns error"/, 'fs_main must pin missing-file assertion label');
assert.match(doctest.code, /test_report_new "fs_main"/, 'fs_main must construct a named TestReport');
assert.match(doctest.code, /test_report_print_stdout\b/, 'fs_main must print the report');
assert.match(doctest.code, /test_report_exit_code\b/, 'fs_main must derive exit code from the shown report');
assert.match(doctest.code, /test_consume_str s/, 'fs_main must consume unexpected success payload before reporting failure');
assert.doesNotMatch(doctest.code, /\bchecks_exit_code\b/, 'fs_main must not hide report details behind checks_exit_code');
assert.doesNotMatch(doctest.code, /\bchecks_print_report\b/, 'fs_main must not use legacy Checks report output');
assert.doesNotMatch(doctest.code, /\bchecks_new\b/, 'fs_main must not use legacy Checks construction');

console.log('stdlib fs.n.md report contract passed');
