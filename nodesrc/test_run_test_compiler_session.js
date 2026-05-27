#!/usr/bin/env node

const assert = require('node:assert/strict');
const fs = require('node:fs');
const os = require('node:os');
const path = require('node:path');
const { runSingle } = require('./run_test');

(async () => {
    const tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), 'nepl-session-test-'));
    const artifactPath = path.join(tmpDir, 'nepl-web-test_bg.wasm');
    fs.writeFileSync(artifactPath, '');
    const future = new Date(Date.now() + 60 * 60 * 1000);
    fs.utimesSync(artifactPath, future, future);

    let sessionCalled = false;
    const result = await runSingle(
        {
            id: 'nodesrc/run_test/compiler-session',
            source: '#target std\nfn main <()->i32> ():\n    0\n',
            tags: ['compile_fail'],
        },
        {
            api: {
                compile_source_with_vfs_and_profile() {
                    throw new Error('stateless compiler API should not be used when a session exists');
                },
            },
            meta: {
                distDir: 'stub',
                jsFile: 'stub.js',
                wasmFile: 'stub.wasm',
                wasmPath: artifactPath,
                compilerSession: {
                    compile_source_with_vfs_and_profile() {
                        sessionCalled = true;
                        throw new Error('session compile failure');
                    },
                },
            },
        },
    );

    assert.equal(result.ok, true);
    assert.equal(result.phase, 'compile');
    assert.equal(sessionCalled, true);
    assert.equal(result.timing.compiler_session, true);
    assert.equal(result.timing.stdlib_vfs_mode, 'bundled');
    assert.match(String(result.compile_error || ''), /session compile failure/);

    fs.rmSync(tmpDir, { recursive: true, force: true });
    console.log('run_test compiler session regression passed');
})().catch((err) => {
    console.error(err);
    process.exitCode = 1;
});
