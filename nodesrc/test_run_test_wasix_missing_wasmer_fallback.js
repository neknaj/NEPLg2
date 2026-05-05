#!/usr/bin/env node

const assert = require('node:assert/strict');
const fs = require('node:fs');
const os = require('node:os');
const path = require('node:path');
const {
    decodeRunExitCode,
    isWasmerExecutableMissing,
    runtimePhaseOk,
    runWasixBytes,
} = require('./run_test');

const previousWasmerBin = process.env.WASMER_BIN;
const temp = fs.mkdtempSync(path.join(os.tmpdir(), 'nepl-missing-wasmer-'));

const mainReturnsSevenWasm = Buffer.from([
    0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00,
    0x01, 0x05, 0x01, 0x60, 0x00, 0x01, 0x7f,
    0x03, 0x02, 0x01, 0x00,
    0x05, 0x03, 0x01, 0x00, 0x01,
    0x07, 0x11, 0x02, 0x04, 0x6d, 0x61, 0x69, 0x6e, 0x00, 0x00,
    0x06, 0x6d, 0x65, 0x6d, 0x6f, 0x72, 0x79, 0x02, 0x00,
    0x0a, 0x06, 0x01, 0x04, 0x00, 0x41, 0x07, 0x0b,
]);

(async () => {
    try {
        const missingWasmer = path.join(temp, 'definitely-missing-wasmer');
        process.env.WASMER_BIN = missingWasmer;

        assert.equal(isWasmerExecutableMissing({
            trapped: true,
            spawnErrorCode: 'ENOENT',
            trapError: `Error: spawn ${missingWasmer} ENOENT`,
        }), true);
        assert.equal(decodeRunExitCode({ runner: 'wasmer', exitCode: 0, returnValue: null }), 0);
        assert.equal(decodeRunExitCode({ runner: 'wasmer', exitCode: 7, returnValue: null }), 7);
        assert.equal(decodeRunExitCode({ runner: 'node-wasi-wasix-tty-fallback', returnValue: 7 }), 7);
        assert.equal(decodeRunExitCode({ runner: 'wasmer', exitCode: null, returnValue: null }), null);
        assert.equal(runtimePhaseOk({ trapped: true, exitCode: 7 }, true, 7), true);
        assert.equal(runtimePhaseOk({ trapped: true, exitCode: 7 }, false, 7), false);
        assert.equal(runtimePhaseOk({ trapped: true, exitCode: null }, true, null), false);
        assert.equal(runtimePhaseOk({ trapped: false, returnValue: 7 }, false, 7), true);

        const result = await runWasixBytes(mainReturnsSevenWasm, '', []);
        assert.equal(result.trapped, false);
        assert.equal(result.returnValue, 7);
        assert.equal(decodeRunExitCode(result), 7);
        assert.equal(result.runner, 'node-wasi-wasix-tty-fallback');
        assert.match(result.fallbackReason, /wasmer executable not found/);

        console.log('run_test wasix missing wasmer fallback ok');
    } finally {
        if (previousWasmerBin === undefined) {
            delete process.env.WASMER_BIN;
        } else {
            process.env.WASMER_BIN = previousWasmerBin;
        }
        fs.rmSync(temp, { recursive: true, force: true });
    }
})().catch((err) => {
    console.error(err && err.stack ? err.stack : err);
    process.exit(1);
});
