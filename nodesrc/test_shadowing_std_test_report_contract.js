#!/usr/bin/env node

const assert = require('node:assert/strict');
const path = require('node:path');
const { parseFile } = require('./parser');

const repoRoot = path.resolve(__dirname, '..');
const fixturePath = path.join(repoRoot, 'tests', 'compiler', 'shadowing.n.md');
const parsed = parseFile(fixturePath);
const doctest = parsed.doctests[22];

assert.ok(doctest, 'shadowing std/test noshadow success doctest must exist at index 23');
assert.equal(doctest.ret, null, 'std/test noshadow success doctest must not use ret as exit-code metadata');
assert.equal(doctest.exit_code, 0, 'std/test noshadow success doctest must pin exit_code');
assert.equal(
    doctest.stdout,
    'test_report name="std_test_noshadow_allows_overload_with_different_signature" count=1 failed=0\n' +
    'assertion index=0 status=ok kind=eq_i32 label="std overload remains available" expected="0" actual="0" message=""\n',
    'std/test noshadow success doctest must pin canonical assertion stdout',
);
assert.match(doctest.code, /fn assert_eq_i32 <\(str,str\)\*\>\(\)>/, 'fixture must keep the different-signature local overload');
assert.match(doctest.code, /test_report_print_stdout report/, 'fixture must print the canonical report');
assert.match(doctest.code, /test_report_exit_code shown/, 'fixture must derive exit code from the shown report');

console.log('shadowing std/test report doctest contract passed');
