#!/usr/bin/env node

const { parentPort, workerData } = require('node:worker_threads');
const { createRunner, runSingle } = require('./run_test');

let loadedPromise = null;

function ensureLoaded() {
    if (!loadedPromise) {
        loadedPromise = createRunner(workerData?.distHint || '');
    }
    return loadedPromise;
}

parentPort.on('message', async (msg) => {
    if (!msg || msg.kind !== 'run') return;
    const idx = Number(msg.index);
    try {
        const loaded = await ensureLoaded();
        const result = await runSingle(msg.req, loaded, (progress) => {
            parentPort.postMessage({
                kind: 'progress',
                index: idx,
                progress,
            });
        });
        parentPort.postMessage({
            kind: 'result',
            index: idx,
            result,
        });
    } catch (e) {
        parentPort.postMessage({
            kind: 'result',
            index: idx,
            result: {
                ok: false,
                status: 'error',
                error: String(e?.stack || e?.message || e),
            },
        });
    }
});

