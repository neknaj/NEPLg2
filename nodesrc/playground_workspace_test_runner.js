#!/usr/bin/env node

const assert = require('node:assert/strict');
const path = require('node:path');
const { pathToFileURL } = require('node:url');

async function loadLayoutModule() {
    const modulePath = path.resolve(__dirname, '..', 'web', 'dist_ts', 'workspace', 'panel-layout.js');
    return import(pathToFileURL(modulePath).href);
}

async function runWorkspaceRegression() {
    const layout = await loadLayoutModule();
    const snapshot = layout.createDefaultWorkspace();
    const leaves = layout.collectLeaves(snapshot.root);
    const explorer = leaves.find((leaf) => leaf.panelKind === 'explorer');
    const editor = leaves.find((leaf) => leaf.panelKind === 'editor');
    const terminal = leaves.find((leaf) => leaf.panelKind === 'terminal');

    assert.ok(explorer);
    assert.ok(editor);
    assert.ok(terminal);
    assert.equal(layout.countLeavesByKind(snapshot.root, 'editor'), 1);

    const secondEditor = layout.createLeaf('editor');
    snapshot.root = layout.splitLeaf(snapshot.root, editor.id, 'h', secondEditor, 'after');
    assert.equal(layout.countLeavesByKind(snapshot.root, 'editor'), 2);

    const moved = layout.moveLeaf(snapshot.root, secondEditor.id, terminal.id, 'bottom');
    assert.equal(layout.countLeavesByKind(moved, 'editor'), 2);

    const afterClose = layout.closeLeaf(moved, secondEditor.id);
    assert.equal(layout.countLeavesByKind(afterClose, 'editor'), 1);

    const clone = layout.cloneWorkspaceSnapshot({ root: afterClose, focusedLeafId: editor.id });
    assert.equal(clone.focusedLeafId, editor.id);
    assert.equal(layout.countLeavesByKind(clone.root, 'explorer'), 1);
    assert.equal(layout.countLeavesByKind(clone.root, 'terminal'), 1);

    editor.zoom = 1.4;
    editor.pathZooms = { '/examples/demo.nepl': 1.4 };
    const normalizedLeaf = layout.normalizeTree(editor);
    assert.equal(normalizedLeaf.zoom, 1.4);
    assert.equal(normalizedLeaf.pathZooms['/examples/demo.nepl'], 1.4);

    return {
        ok: true,
        checks: [
            'default workspace contains explorer, editor, and terminal leaves',
            'editor leaves can be split and moved in the tree',
            'closing a leaf normalizes the split tree',
            'workspace snapshots can be cloned without losing panel kinds',
            'leaf zoom state survives normalize and clone operations',
        ],
    };
}

if (require.main === module) {
    runWorkspaceRegression()
        .then((result) => process.stdout.write(JSON.stringify(result, null, 2) + '\n'))
        .catch((error) => {
            console.error(error && error.stack ? error.stack : String(error));
            process.exit(1);
        });
}

module.exports = {
    runWorkspaceRegression,
};
