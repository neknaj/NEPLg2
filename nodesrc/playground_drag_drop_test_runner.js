#!/usr/bin/env node

const assert = require('node:assert/strict');
const path = require('node:path');
const { pathToFileURL } = require('node:url');

async function loadDragDropModule() {
    const modulePath = path.resolve(__dirname, '..', 'web', 'dist_ts', 'workspace', 'drag-drop.js');
    return import(pathToFileURL(modulePath).href);
}

async function runDragDropRegression() {
    const dragDrop = await loadDragDropModule();

    assert.equal(
        dragDrop.resolveTabbarDropAction({ kind: 'editor-tab', leafId: 'panel-1', path: '/examples/a.nepl' }, 'editor'),
        'attach-tab',
    );
    assert.equal(
        dragDrop.resolveTabbarDropAction({ kind: 'explorer-file', path: '/examples/a.nepl' }, 'editor'),
        'open-file',
    );
    assert.equal(
        dragDrop.resolveTabbarDropAction({ kind: 'panel', leafId: 'panel-2' }, 'editor'),
        'merge-panel',
    );
    assert.equal(
        dragDrop.resolveTabbarDropAction({ kind: 'editor-tab', leafId: 'panel-1', path: '/examples/a.nepl' }, 'terminal'),
        null,
    );
    assert.equal(
        dragDrop.resolveTabbarDropAction(null, 'editor'),
        null,
    );

    return {
        ok: true,
        checks: [
            'editor tab dropped on tabbar resolves to tab attach',
            'explorer file dropped on tabbar resolves to open-file',
            'editor panel dropped on tabbar resolves to panel merge',
            'non-editor targets reject tabbar merge actions',
        ],
    };
}

if (require.main === module) {
    runDragDropRegression()
        .then((result) => process.stdout.write(JSON.stringify(result, null, 2) + '\n'))
        .catch((error) => {
            console.error(error && error.stack ? error.stack : String(error));
            process.exit(1);
        });
}

module.exports = {
    runDragDropRegression,
};
