#!/usr/bin/env node

const assert = require('node:assert/strict');
const path = require('node:path');
const { parseFile } = require('./parser');

const repoRoot = path.resolve(__dirname, '..');
const parsed = parseFile(path.join(repoRoot, 'stdlib', 'alloc', 'string', 'slice', 'trim.nepl'));

assert.ok(parsed.doctests.length >= 1, 'string trim module must keep its public doc-comment doctest');

const doctest = parsed.doctests[0];
const name = 'string_trim_doc';

assert.equal(doctest.ret, null, `${name} must not use ret as test-success metadata`);
assert.equal(doctest.exit_code, 0, `${name} must pin exit_code: 0`);
assert.match(
    doctest.stdout,
    /^test_report name="string_trim_doc" count=6 failed=0\n/,
    `${name} must pin canonical stdout report`,
);
assert.match(doctest.code, /test_report_new "string_trim_doc"/, `${name} must construct a named TestReport`);
assert.match(doctest.code, /str_trim_suffix_cr\s+"abc\\r"/, `${name} must cover suffix CR trimming`);
assert.match(doctest.code, /str_slice_trim_suffix_cr\s+"abcd\\r"\s+1\s+5/, `${name} must cover slice-plus-trim behavior`);
assert.match(doctest.code, /str_trim\s+" \\tabc\\r\\n"/, `${name} must cover ASCII whitespace trimming`);
assert.match(doctest.code, /test_report_print_stdout report/, `${name} must print the report`);
assert.match(doctest.code, /test_report_exit_code shown/, `${name} must derive exit code from the shown report`);
assert.doesNotMatch(doctest.code, /checks_exit_code\s+checks/, `${name} must not hide report details behind checks_exit_code`);

console.log('string trim doc report contract passed');
