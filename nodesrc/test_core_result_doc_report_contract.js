#!/usr/bin/env node

const assert = require('node:assert/strict');
const path = require('node:path');
const { parseFile } = require('./parser');

const repoRoot = path.resolve(__dirname, '..');
const parsed = parseFile(path.join(repoRoot, 'stdlib', 'core', 'result.nepl'));

const expected = [
    { index: 0, name: 'core_result_basic', count: 5 },
    { index: 2, name: 'core_result_map', count: 2 },
    { index: 3, name: 'core_result_and_then', count: 2 },
    { index: 6, name: 'core_result_uwok', count: 1 },
];

assert.ok(parsed.doctests.length >= 7, 'core/result must keep its public doc-comment doctests');
assert.ok(parsed.doctests[1].tags.includes('compile_fail'), 'core/result doctest#2 must remain a compile_fail diagnostic fixture');
assert.ok(parsed.doctests[5].tags.includes('compile_fail'), 'core/result doctest#6 must remain a compile_fail diagnostic fixture');

for (const { index, name, count } of expected) {
    const doctest = parsed.doctests[index];
    assert.equal(doctest.ret, null, `${name} must not use ret as test-success metadata`);
    assert.equal(doctest.exit_code, 0, `${name} must pin exit_code: 0`);
    assert.match(
        doctest.stdout,
        new RegExp(`^test_report name="${name}" count=${count} failed=0\\n`),
        `${name} must pin canonical stdout report`,
    );
    assert.match(doctest.code, new RegExp(`test_report_new "${name}"`), `${name} must construct a named TestReport`);
    assert.match(doctest.code, /test_report_print_stdout report/, `${name} must print the report`);
    assert.match(doctest.code, /test_report_exit_code shown/, `${name} must derive exit code from the shown report`);
    assert.doesNotMatch(doctest.code, /checks_exit_code\s+checks/, `${name} must not hide report details behind checks_exit_code`);
}

console.log('core result doc report contract passed');
