#!/usr/bin/env node

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const { spawn } = require('node:child_process');

const repoRoot = path.resolve(__dirname, '..');
const tmpRoot = path.join(repoRoot, 'tmp', `tests-js-concurrent-${process.pid}`);
const fixture = path.join(tmpRoot, 'concurrent_smoke.n.md');
const outA = path.join(tmpRoot, 'a.json');
const outB = path.join(tmpRoot, 'b.json');

function runTests(outPath) {
    return new Promise((resolve) => {
        const child = spawn(process.execPath, [
            path.join(repoRoot, 'nodesrc', 'tests.js'),
            '-i',
            fixture,
            '--no-tree',
            '--no-stdlib',
            '-o',
            outPath,
            '-j',
            '1',
            '--dist',
            path.join(repoRoot, 'web', 'dist'),
        ], {
            cwd: repoRoot,
            env: {
                ...process.env,
                NEPL_TEST_PROGRESS_FLUSH_EVERY: '1',
            },
            stdio: ['ignore', 'pipe', 'pipe'],
        });
        let stdout = '';
        let stderr = '';
        child.stdout.on('data', (chunk) => { stdout += chunk.toString('utf8'); });
        child.stderr.on('data', (chunk) => { stderr += chunk.toString('utf8'); });
        child.on('close', (code, signal) => {
            resolve({ code, signal, stdout, stderr, outPath });
        });
    });
}

function assertCompletedJson(run) {
    assert.equal(run.code, 0, `tests.js exited unsuccessfully\nstdout:\n${run.stdout}\nstderr:\n${run.stderr}`);
    assert.equal(run.signal, null, `tests.js was signaled: ${run.signal}`);
    assert.equal(fs.existsSync(run.outPath), true, `missing output JSON: ${run.outPath}`);
    const parsed = JSON.parse(fs.readFileSync(run.outPath, 'utf8'));
    assert.equal(parsed.schema, 'neplg2-doctest/v1');
    assert.equal(parsed.partial, false, 'final output must not be partial');
    assert.equal(parsed.summary.total, 1);
    assert.equal(parsed.summary.passed, 1);
    assert.equal(parsed.summary.failed, 0);
    assert.equal(parsed.summary.errored, 0);
    assert.equal(Array.isArray(parsed.resolved_dist_dirs), true);
    assert.ok(parsed.resolved_dist_dirs.length >= 1, 'final JSON must include resolved dist directories');
    assert.equal(Array.isArray(parsed.results), true);
    assert.equal(parsed.results.length, 1);
    assert.equal(parsed.results[0].status, 'pass');
}

async function main() {
    fs.rmSync(tmpRoot, { recursive: true, force: true });
    fs.mkdirSync(tmpRoot, { recursive: true });
    fs.writeFileSync(fixture, [
        '# tests.js concurrent smoke',
        '',
        '## passing case',
        '',
        'neplg2:test',
        'ret: 7',
        '```neplg2',
        '#entry main',
        '#indent 4',
        '#target std',
        '',
        'fn main <()*>i32> ():',
        '    7',
        '```',
        '',
    ].join('\n'), 'utf8');

    try {
        const [a, b] = await Promise.all([runTests(outA), runTests(outB)]);
        assertCompletedJson(a);
        assertCompletedJson(b);
    } finally {
        fs.rmSync(tmpRoot, { recursive: true, force: true });
    }
}

main()
    .then(() => {
        console.log('tests.js concurrent complete JSON regression passed');
    })
    .catch((e) => {
        console.error(e && e.stack ? e.stack : String(e));
        process.exit(1);
    });
