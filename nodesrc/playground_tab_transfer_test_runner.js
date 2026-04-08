#!/usr/bin/env node

const assert = require('node:assert/strict');
const path = require('node:path');
const { pathToFileURL } = require('node:url');

class FakeElement {
    constructor(tagName) {
        this.tagName = tagName;
        this.children = [];
        this.className = '';
        this.textContent = '';
        this.draggable = false;
        this.onclick = null;
    }

    appendChild(child) {
        this.children.push(child);
        return child;
    }

    addEventListener() {}

    removeEventListener() {}

    set innerHTML(_value) {
        this.children = [];
    }
}

function createEditorStub() {
    return {
        text: '',
        path: null,
        editable: false,
        focused: false,
        setText(value) {
            this.text = String(value ?? '');
        },
        getText() {
            return this.text;
        },
        setEditable(value) {
            this.editable = Boolean(value);
        },
        setPath(pathValue) {
            this.path = pathValue;
        },
        focus() {
            this.focused = true;
        },
    };
}

function createVfs(files) {
    const store = new Map(Object.entries(files));
    const writable = [];
    return {
        exists(pathValue) {
            return store.has(pathValue);
        },
        readFile(pathValue) {
            return store.get(pathValue);
        },
        writeFile(pathValue, content) {
            writable.push({ path: pathValue, content });
            store.set(pathValue, content);
        },
        isEditable(pathValue) {
            return !String(pathValue).startsWith('/stdlib') && pathValue !== '/README';
        },
        writes: writable,
    };
}

async function loadTabsModule() {
    const modulePath = path.resolve(__dirname, '..', 'web', 'dist_ts', 'library', 'tabs.js');
    return import(pathToFileURL(modulePath).href);
}

async function runTabTransferRegression() {
    global.document = {
        createElement(tagName) {
            return new FakeElement(tagName);
        },
    };

    const { TabManager } = await loadTabsModule();
    const vfs = createVfs({
        '/examples/a.nepl': 'let a 1;',
        '/examples/b.nepl': 'let b 2;',
    });

    const sourceContainer = new FakeElement('div');
    const targetContainer = new FakeElement('div');
    const sourceEditor = createEditorStub();
    const targetEditor = createEditorStub();

    const source = new TabManager(sourceContainer, sourceEditor, vfs);
    const target = new TabManager(targetContainer, targetEditor, vfs);

    source.restoreTabs(['/examples/a.nepl'], '/examples/a.nepl', { '/examples/a.nepl': 1.25 });
    sourceEditor.setText('let a 99;');
    const detached = source.detachTabByPath('/examples/a.nepl');
    assert.ok(detached);
    assert.equal(detached.content, 'let a 99;');
    assert.equal(detached.zoom, 1.25);
    assert.equal(vfs.writes.length, 1);
    assert.equal(source.tabs.length, 0);

    target.attachTab(detached, { activate: true, focusEditor: false });
    assert.equal(target.tabs.length, 1);
    assert.equal(target.activeTab.path, '/examples/a.nepl');
    assert.equal(target.activeTab.content, 'let a 99;');
    assert.equal(target.activeTab.zoom, 1.25);
    assert.equal(targetEditor.getText(), 'let a 99;');

    target.mergeFrom({
        exportTabs() {
            return [
                {
                    path: '/examples/b.nepl',
                    content: 'let b 123;',
                    isPermanent: true,
                    isEditable: true,
                    zoom: 1.1,
                },
            ];
        },
    });
    assert.equal(target.tabs.length, 2);
    assert.equal(target.tabs[1].content, 'let b 123;');
    assert.equal(target.tabs[1].zoom, 1.1);

    return {
        ok: true,
        checks: [
            'active tab detach preserves edited content and zoom',
            'detached tabs attach into another editor panel without losing state',
            'panel merge imports real tab snapshots instead of re-reading stale VFS content',
        ],
    };
}

if (require.main === module) {
    runTabTransferRegression()
        .then((result) => process.stdout.write(JSON.stringify(result, null, 2) + '\n'))
        .catch((error) => {
            console.error(error && error.stack ? error.stack : String(error));
            process.exit(1);
        });
}

module.exports = {
    runTabTransferRegression,
};
