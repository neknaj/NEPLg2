#!/usr/bin/env node

const assert = require('node:assert/strict');
const path = require('node:path');
const { pathToFileURL } = require('node:url');

async function loadModules() {
    const root = path.resolve(__dirname, '..', 'web', 'dist_ts');
    const [{ VFS }, { TabManager }] = await Promise.all([
        import(pathToFileURL(path.join(root, 'runtime', 'vfs.js')).href),
        import(pathToFileURL(path.join(root, 'library', 'tabs.js')).href),
    ]);
    return { VFS, TabManager };
}

function createMockContainer() {
    return {
        innerHTML: '',
        children: [],
        appendChild(child) {
            this.children.push(child);
        },
    };
}

function createMockDocument() {
    return {
        createElement(tagName) {
            return {
                tagName,
                className: '',
                textContent: '',
                draggable: false,
                onclick: null,
                appendChild() {},
                addEventListener() {},
                removeEventListener() {},
                classList: {
                    add() {},
                    remove() {},
                },
            };
        },
    };
}

function createMockEditor(initialText = '') {
    return {
        text: initialText,
        path: null,
        editable: false,
        focusCallCount: 0,
        calls: [],
        setTextCalls: [],
        setEditableCalls: [],
        setPathCalls: [],
        replaceDocumentCalls: [],
        getText() {
            return this.text;
        },
        setText(text) {
            this.text = text;
            this.calls.push({ kind: 'setText', value: text });
            this.setTextCalls.push(text);
        },
        setEditable(editable) {
            this.editable = editable;
            this.calls.push({ kind: 'setEditable', value: Boolean(editable) });
            this.setEditableCalls.push(editable);
        },
        getEditable() {
            return this.editable;
        },
        focus() {
            this.focusCallCount += 1;
        },
        setPath(pathValue) {
            this.path = pathValue;
            this.calls.push({ kind: 'setPath', value: pathValue });
            this.setPathCalls.push(pathValue);
        },
        replaceDocument(document) {
            this.path = document.path;
            this.text = String(document.text ?? '');
            this.editable = Boolean(document.editable);
            this.calls.push({
                kind: 'replaceDocument',
                path: document.path,
                text: String(document.text ?? ''),
                editable: Boolean(document.editable),
            });
            this.replaceDocumentCalls.push({
                path: document.path,
                text: String(document.text ?? ''),
                editable: Boolean(document.editable),
            });
        },
    };
}

function assertAtomicDocument(editor, pathValue, textValue, editable) {
    const found = editor.replaceDocumentCalls.some((call) => (
        call.path === pathValue
        && call.text === textValue
        && call.editable === editable
    ));
    assert.ok(found, `replaceDocument must atomically apply ${pathValue}`);
}

async function runEditabilityRegression() {
    const { VFS, TabManager } = await loadModules();
    const originalDocument = global.document;
    global.document = createMockDocument();

    try {
        const vfs = new VFS();
        vfs.writeFile('/stdlib/std/io.nepl', 'fn io', { force: true });
        vfs.setReadOnly('/stdlib/std/io.nepl', true);
        vfs.writeFile('/README', 'help text', { force: true });
        vfs.setReadOnly('/README', true);
        vfs.writeFile('/data/input.txt', 'runtime data\n', { force: true });
        vfs.writeFile('/examples/demo.nepl', '#entry main\nprint "ok"\n', { force: true });
        vfs.setReadOnly('/examples/demo.nepl', false);

        assert.equal(vfs.isEditable('/stdlib/std/io.nepl'), false);
        assert.equal(vfs.isEditable('/README'), false);
        assert.equal(vfs.isEditable('/examples/demo.nepl'), true);
        assert.throws(() => vfs.writeFile('/README', 'overwrite'), /read-only/);
        assert.deepEqual(vfs.serializeForCompile(), {
            '/examples/demo.nepl': '#entry main\nprint "ok"\n',
        });

        const editor = createMockEditor();
        const tabs = new TabManager(createMockContainer(), editor, vfs);

        tabs.openFile('/README');
        assert.equal(editor.editable, false);
        assert.equal(tabs.activeTab.isEditable, false);
        assertAtomicDocument(editor, '/README', 'help text', false);

        editor.text = 'mutated readme text';
        tabs.saveCurrentTab();
        assert.equal(vfs.readFile('/README'), 'help text');

        tabs.openFile('/examples/demo.nepl');
        assert.equal(editor.editable, true);
        assert.equal(tabs.activeTab.isEditable, true);
        assertAtomicDocument(editor, '/examples/demo.nepl', '#entry main\nprint "ok"\n', true);

        editor.text = '#entry main\nprint "edited"\n';
        tabs.saveCurrentTab();
        assert.equal(vfs.readFile('/examples/demo.nepl'), '#entry main\nprint "edited"\n');

        vfs.writeFile('/examples/second.nepl', '#entry main\nprint "second"\n', { force: true });
        tabs.openFile('/examples/second.nepl');
        assert.equal(editor.path, '/examples/second.nepl');
        assert.equal(editor.editable, true);
        assert.equal(tabs.activeTab.isEditable, true);
        assertAtomicDocument(editor, '/examples/second.nepl', '#entry main\nprint "second"\n', true);
        tabs.setActiveZoom(1.5);
        assert.equal(tabs.getActiveZoom(), 1.5);
        editor.text = '#entry main\nprint "second edited"\n';
        tabs.setActiveTab(0);
        tabs.setActiveTab(tabs.tabs.findIndex((tab) => tab.path === '/examples/second.nepl'));
        assert.equal(editor.path, '/examples/second.nepl');
        assert.equal(editor.editable, true);
        assert.equal(vfs.readFile('/examples/second.nepl'), '#entry main\nprint "second edited"\n');
        assert.equal(tabs.getActiveZoom(), 1.5);

        tabs.openFile('/stdlib/std/io.nepl');
        assert.equal(editor.editable, false);
        assert.equal(tabs.activeTab.isEditable, false);

        return {
            ok: true,
            checks: [
                'readonly files are not editable in VFS',
                'compile serialization excludes readonly bundled files and runtime data files',
                'readonly tabs disable editor mutation and skip save',
                'editable example files remain writable',
                'tab switching propagates editable state to the editor surface',
                'tab activation atomically replaces provider path text and editable state',
                'switching between editable tabs preserves editability and saves the previous tab',
                'editable tabs preserve their own zoom state across tab switches',
            ],
        };
    } finally {
        global.document = originalDocument;
    }
}

if (require.main === module) {
    runEditabilityRegression()
        .then((result) => {
            process.stdout.write(JSON.stringify(result, null, 2) + '\n');
        })
        .catch((error) => {
            console.error(error && error.stack ? error.stack : String(error));
            process.exit(1);
        });
}

module.exports = {
    runEditabilityRegression,
};
