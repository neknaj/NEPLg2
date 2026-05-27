#!/usr/bin/env node

const assert = require('node:assert/strict');
const { runSingle } = require('./run_test');

(async () => {
    const progress = [];
    const result = await runSingle(
        {
            id: 'nodesrc/run_test/timing-metadata',
            source: '#target std\nfn main <()->i32> ():\n    0\n',
            tags: ['compile_fail'],
        },
        {
            api: {
                compile_source_with_vfs_stdlib_and_profile() {
                    throw new Error('intentional compile failure');
                },
            },
            meta: {
                distDir: 'stub',
                jsFile: 'stub.js',
                wasmFile: 'stub.wasm',
            },
        },
        (event) => progress.push(event),
    );

    assert.equal(result.ok, true);
    assert.equal(result.phase, 'compile');
    assert.equal(typeof result.duration_ms, 'number');
    assert.equal(typeof result.timing.total_ms, 'number');
    assert.equal(typeof result.timing.load_ms, 'number');
    assert.equal(typeof result.timing.warmup_ms, 'number');
    assert.equal(typeof result.timing.collect_vfs_ms, 'number');
    assert.equal(typeof result.timing.stdlib_vfs_ms, 'number');
    assert.equal(typeof result.timing.wasm_call_ms, 'number');
    assert.equal(typeof result.timing.compile_ms, 'number');
    assert.equal(result.timing.run_ms, null);
    assert.ok(['bundled', 'fs_override', 'forced'].includes(result.timing.stdlib_vfs_mode));
    assert.match(String(result.compile_error || ''), /intentional compile failure/);

    assert.ok(progress.some((event) => event.phase === 'load' && event.event === 'start'));
    assert.ok(progress.some((event) => event.phase === 'load' && event.event === 'end'));
    assert.ok(progress.some((event) => event.phase === 'compile' && event.event === 'start'));
    assert.ok(progress.some((event) => event.phase === 'compile' && event.event === 'end'));
    assert.equal(progress.some((event) => event.phase === 'run'), false);

    console.log('run_test timing metadata regression passed');
})().catch((err) => {
    console.error(err);
    process.exitCode = 1;
});
