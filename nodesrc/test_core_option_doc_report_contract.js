#!/usr/bin/env node

const assert = require('node:assert/strict');
const path = require('node:path');
const { parseFile } = require('./parser');

const repoRoot = path.resolve(__dirname, '..');
const parsed = parseFile(path.join(repoRoot, 'stdlib', 'core', 'option.nepl'));

const expected = [
    'core_option_basic',
    'core_option_map',
    'core_option_and_then',
];

assert.ok(parsed.doctests.length >= expected.length, 'core/option must keep its public doc-comment doctests');

const doctestCode = parsed.doctests.map((doctest) => doctest.code).join('\n');
assert.doesNotMatch(
    doctestCode,
    /\b(?:map|and_then)<[^>\r\n]+>/,
    'core/option doctests must use typed locals or result annotations instead of map/and_then generic postfixes',
);

for (let i = 0; i < expected.length; i++) {
    const doctest = parsed.doctests[i];
    const name = expected[i];
    assert.equal(doctest.ret, null, `${name} must not use ret as test-success metadata`);
    assert.equal(doctest.exit_code, 0, `${name} must pin exit_code: 0`);
    assert.match(doctest.stdout, new RegExp(`^test_report name="${name}" count=`), `${name} must pin canonical stdout report`);
    assert.match(doctest.code, new RegExp(`test_report_new "${name}"`), `${name} must construct a named TestReport`);
    assert.match(doctest.code, /test_report_print_stdout report/, `${name} must print the report`);
    assert.match(doctest.code, /test_report_exit_code shown/, `${name} must derive exit code from the shown report`);
    assert.doesNotMatch(doctest.code, /checks_exit_code\s+checks/, `${name} must not hide report details behind checks_exit_code`);
}

console.log('core option doc report contract passed');
