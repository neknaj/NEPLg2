#!/usr/bin/env node

const assert = require('node:assert/strict');
const {
    applyDoctestExpectations,
    buildLlvmRunResult,
    llvmReturnValueFromProcessResult,
} = require('./tests');

assert.equal(llvmReturnValueFromProcessResult({ code: 0, signal: null }), 0);
assert.equal(llvmReturnValueFromProcessResult({ code: 123, signal: null }), 123);
assert.equal(llvmReturnValueFromProcessResult({ code: null, signal: 'SIGSEGV' }), null);
assert.equal(llvmReturnValueFromProcessResult({ code: -1, signal: null }), null);

const testCase = {
    id: 'llvm-return-smoke',
    file: 'inline.nepl',
    index: 1,
    tags: [],
    expected_ret: 123,
};
const runResult = buildLlvmRunResult(
    testCase,
    1,
    'out.ll',
    'out.exe',
    { code: 123, signal: null, stdout: '', stderr: '' },
    Date.now(),
);
assert.equal(runResult.return_value, 123);
assert.equal(runResult.status, 'pass');

const checked = applyDoctestExpectations(runResult, testCase);
assert.equal(checked.ok, true);
assert.equal(checked.status, 'pass');

const compileOnly = applyDoctestExpectations(
    { ok: true, status: 'pass', phase: 'compile_llvm_cli', tags: [] },
    testCase,
    { llvmCompileOnly: true },
);
assert.equal(compileOnly.ok, true);
assert.equal(compileOnly.status, 'pass');

const signaled = buildLlvmRunResult(
    testCase,
    1,
    'out.ll',
    'out.exe',
    { code: null, signal: 'SIGSEGV', stdout: '', stderr: 'segfault' },
    Date.now(),
);
assert.equal(signaled.return_value, null);
assert.equal(signaled.ok, false);

console.log('llvm runner return value regression passed');
