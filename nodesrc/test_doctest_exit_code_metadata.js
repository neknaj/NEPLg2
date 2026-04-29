#!/usr/bin/env node

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const { spawnSync } = require('node:child_process');
const { parseNmdText, parseNeplText } = require('./parser');

const repoRoot = path.resolve(__dirname, '..');

function sourceBetween(source, startMarker, endMarker) {
    const start = source.indexOf(startMarker);
    assert.notEqual(start, -1, `missing source marker: ${startMarker}`);
    const end = source.indexOf(endMarker, start);
    assert.notEqual(end, -1, `missing source marker: ${endMarker}`);
    return source.slice(start, end);
}

const nmdSource = `# doctest exit metadata

neplg2:test
ret: 11
exit_code: 7
\`\`\`neplg2
#entry main
#target wasm
fn main <()->i32> ():
    7
\`\`\`
`;

{
    const parsed = parseNmdText(nmdSource);
    assert.equal(parsed.doctests.length, 1);
    const dt = parsed.doctests[0];
    assert.equal(dt.ret, 11);
    assert.equal(dt.exit_code, 7);
}

const neplDocSource = `//: neplg2:test
//: exit_code: 3
//: \`\`\`neplg2
//:| #entry main
//:| #target wasm
//: fn main <()->i32> ():
//:     3
//: \`\`\`
`;

{
    const parsed = parseNeplText(neplDocSource);
    assert.equal(parsed.doctests.length, 1);
    assert.equal(parsed.doctests[0].exit_code, 3);
    assert.equal(parsed.doctests[0].ret, null);
}

{
    const runDoctestSource = fs.readFileSync(path.join(__dirname, 'run_doctest.js'), 'utf8');
    const focusedExitCodeExpectation = sourceBetween(
        runDoctestSource,
        "if (Object.prototype.hasOwnProperty.call(testCase, 'expected_exit_code')",
        "if (Object.prototype.hasOwnProperty.call(testCase, 'expected_ret')",
    );
    assert.doesNotMatch(focusedExitCodeExpectation, /return_value/);

    const aggregateRunnerSource = fs.readFileSync(path.join(__dirname, 'tests.js'), 'utf8');
    const aggregateExitCodeExpectation = sourceBetween(
        aggregateRunnerSource,
        'if (wantsExitCode) {',
        'if (wantsRet) {',
    );
    assert.doesNotMatch(aggregateExitCodeExpectation, /return_value/);
}

const distDir = path.join(repoRoot, 'web', 'dist');
if (fs.existsSync(distDir)) {
    const tmpDir = path.join(repoRoot, 'tmp');
    fs.mkdirSync(tmpDir, { recursive: true });
    const goodFixture = path.join(tmpDir, 'doctest-exit-code-good.n.md');
    const badFixture = path.join(tmpDir, 'doctest-exit-code-bad.n.md');
    const baseCase = `neplg2:test
exit_code: EXIT_CODE_PLACEHOLDER
\`\`\`neplg2
#entry main
#target wasm
fn main <()->i32> ():
    7
\`\`\`
`;
    fs.writeFileSync(goodFixture, baseCase.replace('EXIT_CODE_PLACEHOLDER', '7'));
    fs.writeFileSync(badFixture, baseCase.replace('EXIT_CODE_PLACEHOLDER', '8'));

    const runDoctest = (fixture) => spawnSync(process.execPath, [
        path.join(__dirname, 'run_doctest.js'),
        '-i',
        fixture,
        '-n',
        '1',
        '--dist',
        distDir,
    ], {
        cwd: repoRoot,
        encoding: 'utf8',
        env: {
            ...process.env,
            NO_COLOR: 'true',
        },
    });
    const runAggregate = (fixture, outputPath) => spawnSync(process.execPath, [
        path.join(__dirname, 'tests.js'),
        '-i',
        fixture,
        '--no-tree',
        '-o',
        outputPath,
        '-j',
        '1',
        '--dist',
        distDir,
    ], {
        cwd: repoRoot,
        encoding: 'utf8',
        env: {
            ...process.env,
            NO_COLOR: 'true',
        },
    });

    const good = runDoctest(goodFixture);
    assert.equal(
        good.status,
        0,
        `expected matching exit_code to pass\nstdout:\n${good.stdout}\nstderr:\n${good.stderr}`,
    );

    const bad = runDoctest(badFixture);
    assert.equal(
        bad.status,
        1,
        `expected mismatching exit_code to fail\nstdout:\n${bad.stdout}\nstderr:\n${bad.stderr}`,
    );
    assert.match(bad.stdout, /exit code mismatch/);
    assert.match(bad.stdout, /expected: 8/);
    assert.match(bad.stdout, /actual:\s+7/);

    const aggregateGoodOut = path.join(tmpDir, 'doctest-exit-code-good-tests.json');
    const aggregateGood = runAggregate(goodFixture, aggregateGoodOut);
    assert.equal(
        aggregateGood.status,
        0,
        `expected aggregate runner with matching exit_code to pass\nstdout:\n${aggregateGood.stdout}\nstderr:\n${aggregateGood.stderr}`,
    );
    const aggregateGoodJson = JSON.parse(fs.readFileSync(aggregateGoodOut, 'utf8'));
    assert.equal(aggregateGoodJson.summary.passed, 1);
    assert.equal(aggregateGoodJson.summary.failed, 0);

    const aggregateBadOut = path.join(tmpDir, 'doctest-exit-code-bad-tests.json');
    const aggregateBad = runAggregate(badFixture, aggregateBadOut);
    assert.equal(
        aggregateBad.status,
        1,
        `expected aggregate runner with mismatching exit_code to fail\nstdout:\n${aggregateBad.stdout}\nstderr:\n${aggregateBad.stderr}`,
    );
    const aggregateBadJson = JSON.parse(fs.readFileSync(aggregateBadOut, 'utf8'));
    assert.equal(aggregateBadJson.summary.passed, 0);
    assert.equal(aggregateBadJson.summary.failed, 1);
    assert.match(aggregateBadJson.results[0].error, /exit code mismatch/);
    assert.match(aggregateBadJson.results[0].error, /expected: 8/);
    assert.match(aggregateBadJson.results[0].error, /actual:\s+7/);
}

console.log('doctest exit_code metadata regression passed');
