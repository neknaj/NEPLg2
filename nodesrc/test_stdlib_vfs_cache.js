#!/usr/bin/env node

const assert = require('node:assert/strict');
const fs = require('node:fs');
const os = require('node:os');
const path = require('node:path');

const {
    loadStdlibVfsFromFs,
    clearStdlibVfsCacheForTests,
    stdlibVfsCacheSizeForTests,
} = require('./stdlib_vfs_cache');
const cli = require('./cli');

function makeStdlibRoot(label) {
    const root = fs.mkdtempSync(path.join(os.tmpdir(), `nepl-stdlib-vfs-${label}-`));
    fs.mkdirSync(path.join(root, 'core'), { recursive: true });
    fs.writeFileSync(path.join(root, 'core', 'one.nepl'), 'fn one <()->i32> ():\n    1\n', 'utf8');
    fs.writeFileSync(path.join(root, 'ignored.txt'), 'not nepl', 'utf8');
    return root;
}

try {
    clearStdlibVfsCacheForTests();

    const rootA = makeStdlibRoot('a');
    const firstA = loadStdlibVfsFromFs(rootA);
    const secondA = loadStdlibVfsFromFs(rootA);
    assert.equal(firstA, secondA);
    assert.deepEqual(Object.keys(firstA), ['/stdlib/core/one.nepl']);
    assert.equal(stdlibVfsCacheSizeForTests(), 1);

    fs.writeFileSync(path.join(rootA, 'core', 'two.nepl'), 'fn two <()->i32> ():\n    2\n', 'utf8');
    const stillCachedA = loadStdlibVfsFromFs(rootA);
    assert.equal(stillCachedA, firstA);
    assert.deepEqual(Object.keys(stillCachedA), ['/stdlib/core/one.nepl']);

    const rootB = makeStdlibRoot('b');
    const firstB = loadStdlibVfsFromFs(rootB);
    assert.notEqual(firstB, firstA);
    assert.equal(stdlibVfsCacheSizeForTests(), 2);

    clearStdlibVfsCacheForTests();
    const refreshedA = loadStdlibVfsFromFs(rootA);
    assert.notEqual(refreshedA, firstA);
    assert.deepEqual(Object.keys(refreshedA).sort(), [
        '/stdlib/core/one.nepl',
        '/stdlib/core/two.nepl',
    ]);

    clearStdlibVfsCacheForTests();
    const cliFirst = cli.loadStdlibVfsFromFs(rootA);
    const cliSecond = cli.loadStdlibVfsFromFs(rootA);
    assert.equal(cliFirst, cliSecond);

    const missingRoot = path.join(os.tmpdir(), `nepl-stdlib-vfs-missing-${process.pid}`);
    clearStdlibVfsCacheForTests();
    assert.deepEqual(loadStdlibVfsFromFs(missingRoot, { missing: 'empty' }), {});
    assert.throws(() => cli.loadStdlibVfsFromFs(missingRoot), /stdlib root not found/);

    console.log('stdlib VFS cache tests passed');
} finally {
    clearStdlibVfsCacheForTests();
}
