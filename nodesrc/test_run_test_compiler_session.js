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
    let prewarmCalled = false;
    let cacheStatsCalls = 0;
    let dependencyAggregateHits = 3;
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
                            arity_surface_bypasses: 0,
                            public_surface_hash_hits: cacheStatsCalls,
                            public_surface_hash_stores: 0,
                            public_surface_hash_bypasses: 0,
                            dependency_aggregate_public_surface_hash_hits: dependencyAggregateHits,
                            dependency_aggregate_public_surface_hash_misses: 0,
                            dependency_aggregate_public_surface_hash_stores: 0,
                            dependency_aggregate_public_surface_hash_bypasses: 0,
                            stdlib_override_bypasses: 0,
                            resource_summary_value_hits: 17,
                            resource_summary_value_misses: 0,
                            resource_summary_value_stores: 0,
                            resource_summary_value_bypasses: 2,
                            resource_summary_value_drop_traversal_forall_hits: 11,
                            resource_summary_value_drop_traversal_forall_stores: 0,
                            resource_summary_value_drop_traversal_forall_bypasses: 2,
                        });
                    },
                    prewarm_loader_cache_for_source(entryPath, source) {
                        prewarmCalled = true;
                        assert.equal(entryPath, '/virtual/entry.nepl');
                        assert.match(source, /fn main/);
                        return 1;
                    },
                    compile_source_with_vfs_and_profile() {
                        sessionCalled = true;
                        dependencyAggregateHits += 1;
                        throw new Error('session compile failure');
                    },
                },
            },
        },
    );

    assert.equal(result.ok, true);
    assert.equal(result.phase, 'compile');
    assert.equal(prewarmCalled, true);
    assert.equal(sessionCalled, true);
    assert.equal(result.timing.compiler_session, true);
    assert.equal(result.timing.stdlib_vfs_mode, 'bundled');
    assert.equal(result.timing.compiler_session_prewarm_count, 1);
    assert.equal(result.timing.compiler_session_prewarm_skipped_reason, null);
    assert.equal(result.timing.compiler_session_cache_before.parsed_module_hits, 3);
    assert.equal(result.timing.compiler_session_cache_after.parsed_module_hits, 4);
    assert.equal(result.timing.compiler_session_cache_before.arity_surface_hits, 3);
    assert.equal(result.timing.compiler_session_cache_after.arity_surface_hits, 4);
    assert.equal(result.timing.compiler_session_prewarm_cache_before.dependency_aggregate_public_surface_hash_hits, 3);
    assert.equal(result.timing.compiler_session_prewarm_cache_after.dependency_aggregate_public_surface_hash_hits, 3);
    assert.equal(result.timing.compiler_session_cache_before.dependency_aggregate_public_surface_hash_hits, 3);
    assert.equal(result.timing.compiler_session_cache_after.dependency_aggregate_public_surface_hash_hits, 4);
    assert.equal(result.timing.compiler_session_cache_before.resource_summary_value_hits, 17);
    assert.equal(result.timing.compiler_session_cache_after.resource_summary_value_drop_traversal_forall_bypasses, 2);
    assert.match(String(result.compile_error || ''), /session compile failure/);

    fs.rmSync(tmpDir, { recursive: true, force: true });

    const tmpDirPrewarmReuse = fs.mkdtempSync(path.join(os.tmpdir(), 'nepl-session-prewarm-reuse-'));
    const prewarmReuseArtifactPath = path.join(tmpDirPrewarmReuse, 'nepl-web-test_bg.wasm');
    fs.writeFileSync(prewarmReuseArtifactPath, '');
    fs.utimesSync(prewarmReuseArtifactPath, future, future);

    let reusePrewarmed = false;
    let reusePrewarmHits = 0;
    let reusePrewarmStores = 0;
    let reuseCompileCalls = 0;
    let reuseCompiledOutputHits = 0;
    let reuseDependencyAggregateHits = 5;
    let reuseResourceSummaryHits = 0;
    let reuseResourceSummaryMisses = 0;
    let reuseResourceSummaryStores = 0;
    const prewarmReuseLoaded = {
        api: {
            compile_source_with_vfs_and_profile() {
                throw new Error('stateless compiler API should not be used for prewarm reuse test');
            },
        },
        meta: {
            distDir: 'stub',
            jsFile: 'stub.js',
            wasmFile: 'stub.wasm',
            wasmPath: prewarmReuseArtifactPath,
            compilerSession: {
                loader_cache_stats_json() {
                    return JSON.stringify({
                        parsed_module_hits: 7,
                        parsed_module_misses: 0,
                        parsed_module_stores: 0,
                        parsed_module_bypasses: 0,
                        arity_surface_hits: 11,
                        arity_surface_misses: 0,
                        arity_surface_stores: 0,
                        arity_surface_bypasses: 0,
                        public_surface_hash_hits: 13,
                        public_surface_hash_stores: 2,
                        public_surface_hash_bypasses: 0,
                        dependency_aggregate_public_surface_hash_hits: reuseDependencyAggregateHits,
                        dependency_aggregate_public_surface_hash_misses: 2,
                        dependency_aggregate_public_surface_hash_stores: 2,
                        dependency_aggregate_public_surface_hash_bypasses: 0,
                        stdlib_override_bypasses: 0,
                        compiled_output_cache_hits: reuseCompiledOutputHits,
                        compiled_output_cache_stores: reuseCompileCalls,
                        prewarm_surface_hits: reusePrewarmHits,
                        prewarm_surface_stores: reusePrewarmStores,
                        resource_summary_value_hits: reuseResourceSummaryHits,
                        resource_summary_value_misses: reuseResourceSummaryMisses,
                        resource_summary_value_stores: reuseResourceSummaryStores,
                        resource_summary_value_bypasses: 0,
                        resource_summary_value_drop_traversal_forall_hits: reuseResourceSummaryHits,
                        resource_summary_value_drop_traversal_forall_stores: reuseResourceSummaryStores,
                        resource_summary_value_drop_traversal_forall_bypasses: 0,
                    });
                },
                prewarm_loader_cache_for_source() {
                    if (reusePrewarmed) {
                        reusePrewarmHits += 1;
                        return 2;
                    }
                    reusePrewarmed = true;
                    reusePrewarmStores += 1;
                    return 2;
                },
                compile_source_with_vfs_and_profile() {
                    if (reuseCompiledOutputHits > 0) {
                        throw new Error('session compiled-output cache hit failure');
                    }
                    reuseCompileCalls += 1;
                    reuseDependencyAggregateHits += 1;
                    if (reuseCompileCalls > 1) {
                        reuseResourceSummaryHits += 1;
                    } else {
                        reuseResourceSummaryMisses += 1;
                        reuseResourceSummaryStores += 1;
                    }
                    throw new Error('session compile failure after reused prewarm surface');
                },
            },
        },
    };
    await runSingle(
        {
            id: 'nodesrc/run_test/compiler-session-prewarm-reuse-first',
            source: '#target std\nfn main <()->i32> ():\n    0\n',
            tags: ['compile_fail'],
        },
        prewarmReuseLoaded,
    );
    const prewarmReuseSecond = await runSingle(
        {
            id: 'nodesrc/run_test/compiler-session-prewarm-reuse-second',
            source: '#target std\nfn main <()->i32> ():\n    1\n',
            tags: ['compile_fail'],
        },
        prewarmReuseLoaded,
    );

    assert.equal(prewarmReuseSecond.ok, true);
    assert.equal(prewarmReuseSecond.timing.compiler_session_prewarm_count, 2);
    assert.equal(prewarmReuseSecond.timing.compiler_session_prewarm_skipped_reason, null);
    assert.equal(prewarmReuseSecond.timing.compiler_session_prewarm_cache_before.prewarm_surface_hits, 0);
    assert.equal(prewarmReuseSecond.timing.compiler_session_prewarm_cache_after.prewarm_surface_hits, 1);
    assert.equal(prewarmReuseSecond.timing.compiler_session_prewarm_cache_after.prewarm_surface_stores, 1);
    assert.match(
        String(prewarmReuseSecond.compile_error || ''),
        /session compile failure after reused prewarm surface/,
    );

    reuseCompiledOutputHits = 1;
    const compiledOutputHit = await runSingle(
        {
            id: 'nodesrc/run_test/compiler-session-compiled-output-hit',
            source: '#target std\nfn main <()->i32> ():\n    1\n',
            tags: ['compile_fail'],
        },
        prewarmReuseLoaded,
    );
    assert.equal(compiledOutputHit.ok, true);
    assert.equal(compiledOutputHit.timing.compiler_session_cache_before.compiled_output_cache_hits, 1);
    assert.equal(compiledOutputHit.timing.compiler_session_cache_after.compiled_output_cache_hits, 1);
    assert.equal(
        compiledOutputHit.timing.compiler_session_cache_before.dependency_aggregate_public_surface_hash_hits,
        reuseDependencyAggregateHits,
    );
    assert.equal(
        compiledOutputHit.timing.compiler_session_cache_after.dependency_aggregate_public_surface_hash_hits,
        reuseDependencyAggregateHits,
    );
    assert.equal(
        compiledOutputHit.timing.compiler_session_cache_before.resource_summary_value_hits,
        reuseResourceSummaryHits,
    );
    assert.equal(
        compiledOutputHit.timing.compiler_session_cache_after.resource_summary_value_hits,
        reuseResourceSummaryHits,
    );
    assert.equal(
        compiledOutputHit.timing.compiler_session_cache_before.resource_summary_value_stores,
        reuseResourceSummaryStores,
    );
    assert.equal(
        compiledOutputHit.timing.compiler_session_cache_after.resource_summary_value_stores,
        reuseResourceSummaryStores,
    );

    fs.rmSync(tmpDirPrewarmReuse, { recursive: true, force: true });

    const tmpDirStdlibOverlay = fs.mkdtempSync(path.join(os.tmpdir(), 'nepl-session-stdlib-overlay-'));
    const stdlibOverlayArtifactPath = path.join(tmpDirStdlibOverlay, 'nepl-web-test_bg.wasm');
    fs.writeFileSync(stdlibOverlayArtifactPath, '');
    fs.utimesSync(stdlibOverlayArtifactPath, future, future);

    let stdlibOverlayCompileCalled = false;
    let stdlibOverlayPrewarmCalled = false;
    const stdlibOverlayResult = await runSingle(
        {
            id: 'nodesrc/run_test/compiler-session-stdlib-overlay',
            source: '#target std\nfn main <()->i32> ():\n    0\n',
            tags: ['compile_fail'],
            vfs: {
                '/stdlib/std/prelude_base.nepl': '',
            },
        },
        {
            api: {
                compile_source_with_vfs_and_profile() {
                    throw new Error('stateless compiler API should not be used for stdlib overlay test');
                },
            },
            meta: {
                distDir: 'stub',
                jsFile: 'stub.js',
                wasmFile: 'stub.wasm',
                wasmPath: stdlibOverlayArtifactPath,
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
                            arity_surface_bypasses: 0,
                            public_surface_hash_hits: 0,
                            public_surface_hash_stores: 0,
                            public_surface_hash_bypasses: 0,
                            dependency_aggregate_public_surface_hash_hits: 0,
                            dependency_aggregate_public_surface_hash_misses: 0,
                            dependency_aggregate_public_surface_hash_stores: 0,
                            dependency_aggregate_public_surface_hash_bypasses: 0,
                            stdlib_override_bypasses: 0,
                            prewarm_surface_hits: 0,
                            prewarm_surface_stores: 0,
                        });
                    },
                    prewarm_loader_cache_for_source() {
                        stdlibOverlayPrewarmCalled = true;
                        throw new Error('stdlib overlay compile must not prewarm bundled stdlib');
                    },
                    compile_source_with_vfs_and_profile() {
                        stdlibOverlayCompileCalled = true;
                        throw new Error('session stdlib overlay compile failure');
                    },
                },
            },
        },
    );

    assert.equal(stdlibOverlayResult.ok, true);
    assert.equal(stdlibOverlayCompileCalled, true);
    assert.equal(stdlibOverlayPrewarmCalled, false);
    assert.equal(stdlibOverlayResult.timing.compiler_session_prewarm_count, null);
    assert.equal(stdlibOverlayResult.timing.compiler_session_prewarm_skipped_reason, 'stdlib_overlay');
    assert.match(String(stdlibOverlayResult.compile_error || ''), /session stdlib overlay compile failure/);

    fs.rmSync(tmpDirStdlibOverlay, { recursive: true, force: true });

    const tmpDirForced = fs.mkdtempSync(path.join(os.tmpdir(), 'nepl-session-forced-stdlib-'));
    const forcedArtifactPath = path.join(tmpDirForced, 'nepl-web-test_bg.wasm');
    fs.writeFileSync(forcedArtifactPath, '');
    fs.utimesSync(forcedArtifactPath, future, future);

    let forcedStdlibSessionCalled = false;
    let forcedPrewarmCalled = false;
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
                            arity_surface_bypasses: 0,
                            public_surface_hash_hits: 0,
                            public_surface_hash_stores: 0,
                            public_surface_hash_bypasses: 0,
                            dependency_aggregate_public_surface_hash_hits: 0,
                            dependency_aggregate_public_surface_hash_misses: 0,
                            dependency_aggregate_public_surface_hash_stores: 0,
                            dependency_aggregate_public_surface_hash_bypasses: 0,
                            stdlib_override_bypasses: forcedStdlibSessionCalled ? 1 : 0,
                        });
                    },
                    prewarm_loader_cache_for_source() {
                        forcedPrewarmCalled = true;
                        throw new Error('forced stdlib compile must not prewarm bundled stdlib');
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
    assert.equal(forcedPrewarmCalled, false);
    assert.equal(forcedResult.timing.compiler_session, true);
    assert.equal(forcedResult.timing.stdlib_vfs_mode, 'forced');
    assert.equal(forcedResult.timing.compiler_session_prewarm_count, null);
    assert.equal(forcedResult.timing.compiler_session_prewarm_skipped_reason, 'forced');
    assert.equal(
        forcedResult.timing.compiler_session_cache_after.stdlib_override_bypasses,
        1,
    );
    assert.match(String(forcedResult.compile_error || ''), /session forced stdlib compile failure/);

    fs.rmSync(tmpDirForced, { recursive: true, force: true });

    const tmpDirPrewarmFailure = fs.mkdtempSync(path.join(os.tmpdir(), 'nepl-session-prewarm-failure-'));
    const prewarmFailureArtifactPath = path.join(tmpDirPrewarmFailure, 'nepl-web-test_bg.wasm');
    fs.writeFileSync(prewarmFailureArtifactPath, '');
    fs.utimesSync(prewarmFailureArtifactPath, future, future);

    const prewarmFailureResult = await runSingle(
        {
            id: 'nodesrc/run_test/compiler-session-prewarm-failure',
            source: '#target std\nfn main <()->i32> ():\n    0\n',
            tags: ['compile_fail'],
        },
        {
            api: {
                compile_source_with_vfs_and_profile() {
                    throw new Error('stateless compiler API should not be used for prewarm failure test');
                },
            },
            meta: {
                distDir: 'stub',
                jsFile: 'stub.js',
                wasmFile: 'stub.wasm',
                wasmPath: prewarmFailureArtifactPath,
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
                            arity_surface_bypasses: 0,
                            public_surface_hash_hits: 0,
                            public_surface_hash_stores: 0,
                            public_surface_hash_bypasses: 0,
                            dependency_aggregate_public_surface_hash_hits: 0,
                            dependency_aggregate_public_surface_hash_misses: 0,
                            dependency_aggregate_public_surface_hash_stores: 0,
                            dependency_aggregate_public_surface_hash_bypasses: 0,
                            stdlib_override_bypasses: 0,
                        });
                    },
                    prewarm_loader_cache_for_source() {
                        throw new Error('prewarm-only failure');
                    },
                    compile_source_with_vfs_and_profile() {
                        throw new Error('real compile failure');
                    },
                },
            },
        },
    );

    assert.equal(prewarmFailureResult.ok, true);
    assert.match(String(prewarmFailureResult.timing.compiler_session_prewarm_error || ''), /prewarm-only failure/);
    assert.match(String(prewarmFailureResult.compile_error || ''), /real compile failure/);

    fs.rmSync(tmpDirPrewarmFailure, { recursive: true, force: true });
    console.log('run_test compiler session regression passed');
})().catch((err) => {
    console.error(err);
    process.exitCode = 1;
});
