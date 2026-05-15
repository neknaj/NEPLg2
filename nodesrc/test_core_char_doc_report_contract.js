#!/usr/bin/env node

const assert = require('node:assert/strict');
const path = require('node:path');
const { parseFile } = require('./parser');

const repoRoot = path.resolve(__dirname, '..');
const parsed = parseFile(path.join(repoRoot, 'stdlib', 'core', 'char.nepl'));

assert.ok(parsed.doctests.length >= 1, 'core/char must keep its public doc-comment doctest');

const doctest = parsed.doctests[0];
const name = 'core_char_basic';

assert.equal(doctest.ret, null, `${name} must not use ret as test-success metadata`);
assert.equal(doctest.exit_code, 0, `${name} must pin exit_code: 0`);
assert.match(
    doctest.stdout,
    /^test_report name="core_char_basic" count=9 failed=0\n/,
    `${name} must pin canonical stdout report`,
);
assert.match(doctest.code, /test_report_new "core_char_basic"/, `${name} must construct a named TestReport`);
assert.match(doctest.code, /test_report_print_stdout report/, `${name} must print the report`);
assert.match(doctest.code, /test_report_exit_code shown/, `${name} must derive exit code from the shown report`);
assert.doesNotMatch(doctest.code, /checks_exit_code\s+checks/, `${name} must not hide report details behind checks_exit_code`);

console.log('core char doc report contract passed');
