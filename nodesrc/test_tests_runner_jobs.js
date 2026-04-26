#!/usr/bin/env node

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const { spawnSync } = require('node:child_process');

const repoRoot = path.resolve(__dirname, '..');
const testsPath = path.join(__dirname, 'tests.js');

function runIoSuite(jobs) {
    const outPath = path.join(repoRoot, 'tmp', `nodesrc-tests-runner-j${jobs}.json`);
    const result = spawnSync(process.execPath, [
        testsPath,
        '-i',
        'tests/stdlib/io.n.md',
        '--no-tree',
        '-o',
        outPath,
        '-j',
        String(jobs),
    ], {
        cwd: repoRoot,
        encoding: 'utf8',
        env: {
            ...process.env,
            NEPL_TEST_CASE_TIMEOUT_MS: '60000',
        },
    });
    assert.equal(
        result.status,
        0,
        `nodesrc/tests.js -j ${jobs} failed\nstdout:\n${result.stdout}\nstderr:\n${result.stderr}`,
    );
    const json = JSON.parse(fs.readFileSync(outPath, 'utf8'));
    assert.equal(json.summary.total, 6);
    assert.equal(json.summary.passed, 6);
    assert.equal(json.summary.failed, 0);
    assert.equal(json.summary.errored, 0);
}

runIoSuite(1);
runIoSuite(2);

console.log('nodesrc tests runner job mode regression passed');
