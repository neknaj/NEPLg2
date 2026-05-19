#!/usr/bin/env node

const assert = require('node:assert/strict');
const path = require('node:path');
const { parseFile } = require('./parser');

const repoRoot = path.resolve(__dirname, '..');
const file = path.join(repoRoot, 'stdlib', 'tests', 'rand.n.md');
const parsed = parseFile(file);

assert.equal(parsed.doctests.length, 1, 'stdlib/tests/rand.n.md doctest count changed');

const doctest = parsed.doctests[0];
assert.equal(doctest.ret, null, 'rand_main must not use ret-only success reporting');
assert.equal(doctest.exit_code, 0, 'rand_main must pin exit_code: 0');
assert.match(
    doctest.stdout,
    /^test_report name="rand_main" count=4 failed=0\n/,
    'rand_main must pin canonical stdout report',
);
for (const label of [
    'first generated state is nonzero',
    'second generated state is nonzero',
    'successive states differ',
    'zero seed escapes zero state',
]) {
    assert.match(doctest.stdout, new RegExp(`label="${label}"`), `rand_main must pin ${label}`);
}
assert.match(doctest.code, /test_report_new "rand_main"/, 'rand_main must construct a named TestReport');
assert.match(doctest.code, /test_report_print_stdout\b/, 'rand_main must print the report');
assert.match(doctest.code, /test_report_exit_code\b/, 'rand_main must derive exit code from the shown report');
assert.doesNotMatch(doctest.code, /\bchecks_exit_code\b/, 'rand_main must not hide report details behind checks_exit_code');
assert.doesNotMatch(doctest.code, /\bchecks_print_report\b/, 'rand_main must not use legacy Checks report output');
assert.doesNotMatch(doctest.code, /\bchecks_new\b/, 'rand_main must not use legacy Checks construction');

console.log('stdlib rand report contract passed');
