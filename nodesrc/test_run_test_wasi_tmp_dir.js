#!/usr/bin/env node

const assert = require('node:assert/strict');
const fs = require('node:fs');
const os = require('node:os');
const path = require('node:path');
const { ensureWasiScratchDir } = require('./run_test');

const root = fs.mkdtempSync(path.join(os.tmpdir(), 'nepl-wasi-root-'));

try {
    const scratch = path.join(root, 'tmp');
    assert.equal(fs.existsSync(scratch), false);

    assert.equal(ensureWasiScratchDir(root), scratch);
    assert.equal(fs.statSync(scratch).isDirectory(), true);

    const marker = path.join(scratch, 'marker.txt');
    fs.writeFileSync(marker, 'keep');
    assert.equal(ensureWasiScratchDir(root), scratch);
    assert.equal(fs.readFileSync(marker, 'utf-8'), 'keep');

    console.log('run_test wasi tmp dir ok');
} finally {
    fs.rmSync(root, { recursive: true, force: true });
}
