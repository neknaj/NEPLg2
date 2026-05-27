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
    let cacheStatsCalls = 0;
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
                    loader_cache_stats_json() {
                        cacheStatsCalls += 1;
                        return JSON.stringify({
                            parsed_module_hits: cacheStatsCalls,
                            parsed_module_misses: 0,
                            parsed_module_stores: 0,
                            parsed_module_bypasses: 0,
                            arity_surface_hits: cacheStatsCalls,
                            arity_surface_misses: 0,
                            arity_surface_stores: 0,
                            stdlib_override_bypasses: 0,
                        });
                    },
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
    assert.equal(result.timing.compiler_session_cache_before.parsed_module_hits, 1);
    assert.equal(result.timing.compiler_session_cache_after.parsed_module_hits, 2);
    assert.equal(result.timing.compiler_session_cache_before.arity_surface_hits, 1);
    assert.equal(result.timing.compiler_session_cache_after.arity_surface_hits, 2);
    assert.match(String(result.compile_error || ''), /session compile failure/);

    fs.rmSync(tmpDir, { recursive: true, force: true });

    const tmpDirForced = fs.mkdtempSync(path.join(os.tmpdir(), 'nepl-session-forced-stdlib-'));
    const forcedArtifactPath = path.join(tmpDirForced, 'nepl-web-test_bg.wasm');
    fs.writeFileSync(forcedArtifactPath, '');
    fs.utimesSync(forcedArtifactPath, future, future);

    let forcedStdlibSessionCalled = false;
    const forcedResult = await runSingle(
        {
            id: 'nodesrc/run_test/compiler-session-forced-stdlib',
            source: '#target std\nfn main <()->i32> ():\n    0\n',
            tags: ['compile_fail'],
            forceStdlibVfs: true,
        },
        {
            api: {
                compile_source_with_vfs_and_profile() {
                    throw new Error('stateless compiler API should not be used for forced stdlib session compile');
                },
            },
            meta: {
                distDir: 'stub',
                jsFile: 'stub.js',
                wasmFile: 'stub.wasm',
                wasmPath: forcedArtifactPath,
                compilerSession: {
                    loader_cache_stats_json() {
                        return JSON.stringify({
                            parsed_module_hits: 0,
                            parsed_module_misses: 0,
                            parsed_module_stores: 0,
                            parsed_module_bypasses: 0,
                            arity_surface_hits: 0,
                            arity_surface_misses: 0,
                            arity_surface_stores: 0,
                            stdlib_override_bypasses: forcedStdlibSessionCalled ? 1 : 0,
                        });
                    },
                    compile_source_with_vfs_stdlib_and_profile() {
                        forcedStdlibSessionCalled = true;
                        throw new Error('session forced stdlib compile failure');
                    },
                },
            },
        },
    );

    assert.equal(forcedResult.ok, true);
    assert.equal(forcedResult.phase, 'compile');
    assert.equal(forcedStdlibSessionCalled, true);
    assert.equal(forcedResult.timing.compiler_session, true);
    assert.equal(forcedResult.timing.stdlib_vfs_mode, 'forced');
    assert.equal(
        forcedResult.timing.compiler_session_cache_after.stdlib_override_bypasses,
        1,
    );
    assert.match(String(forcedResult.compile_error || ''), /session forced stdlib compile failure/);

    fs.rmSync(tmpDirForced, { recursive: true, force: true });
    console.log('run_test compiler session regression passed');
})().catch((err) => {
    console.error(err);
    process.exitCode = 1;
});
