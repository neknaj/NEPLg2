#!/usr/bin/env node

const assert = require('node:assert/strict');
const fs = require('node:fs');
const os = require('node:os');
const path = require('node:path');
const { isWasmerExecutableMissing, runWasixBytes } = require('./run_test');

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

        const result = await runWasixBytes(mainReturnsSevenWasm, '', []);
        assert.equal(result.trapped, false);
        assert.equal(result.returnValue, 7);
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
