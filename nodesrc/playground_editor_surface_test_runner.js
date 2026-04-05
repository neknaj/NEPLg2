#!/usr/bin/env node

const fs = require('node:fs');
const path = require('node:path');
const vm = require('node:vm');
const assert = require('node:assert/strict');

function loadCanvasEditorClass() {
    const editorPath = path.resolve(__dirname, '..', 'web', 'dist_ts', 'editor', 'editor.js');
    const source = fs.readFileSync(editorPath, 'utf8');
    const context = {
        console,
        performance: { now: () => 0 },
        requestAnimationFrame: () => 0,
        window: {},
        EditorUtils: class {},
        EditorRenderer: class {},
        EditorInputHandler: class {},
        EditorDOMUI: class {},
    };
    context.globalThis = context;
    vm.runInNewContext(`${source}\nthis.__CanvasEditor = CanvasEditor;`, context, { filename: editorPath });
    return context.__CanvasEditor;
}

function createMockEditor() {
    return {
        text: 'alpha\nbeta\n',
        cursor: 0,
        selectionStart: 0,
        selectionEnd: 0,
        corePreferredCursorColumn: null,
        isOverwriteMode: false,
        undoStack: [],
        redoStack: [],
        preferredCursorX: 3,
        tokensByLine: [[{ startCol: 0, endCol: 5, type: 'variable' }], []],
        diagnosticsByLine: [[{ startCol: 0, endCol: 5, severity: 'warning', message: 'x' }], []],
        normalizeEditorText(text) {
            return String(text ?? '').replace(/\r\n?/g, '\n');
        },
        updateLinesCallCount: 0,
        updateTextCallCount: 0,
        scrollToCursorCallCount: 0,
        resetCursorBlinkCallCount: 0,
        updateOccurrencesHighlightCallCount: 0,
        updateBracketMatchingCallCount: 0,
        updateLines() {
            this.updateLinesCallCount += 1;
            this.tokensByLine = [];
            this.diagnosticsByLine = [];
        },
        updateText() {
            this.updateTextCallCount += 1;
        },
        scrollToCursor() {
            this.scrollToCursorCallCount += 1;
        },
        resetCursorBlink() {
            this.resetCursorBlinkCallCount += 1;
        },
        updateOccurrencesHighlight() {
            this.updateOccurrencesHighlightCallCount += 1;
        },
        updateBracketMatching() {
            this.updateBracketMatchingCallCount += 1;
        },
        onCursorChange: null,
    };
}

function runSurfaceRegression() {
    const CanvasEditor = loadCanvasEditorClass();
    const applyCoreRuntimeState = CanvasEditor.prototype.applyCoreRuntimeState;

    const cursorMoveEditor = createMockEditor();
    const cursorMoveResult = applyCoreRuntimeState.call(cursorMoveEditor, {
        text: 'alpha\nbeta\n',
        cursor: 3,
        selectionStart: 3,
        selectionEnd: 3,
        preferredCursorColumn: 3,
        isOverwriteMode: false,
        undoStack: [],
        redoStack: [],
    });

    assert.equal(cursorMoveResult, true);
    assert.equal(cursorMoveEditor.updateLinesCallCount, 0);
    assert.equal(cursorMoveEditor.updateTextCallCount, 0);
    assert.deepStrictEqual(cursorMoveEditor.tokensByLine, [[{ startCol: 0, endCol: 5, type: 'variable' }], []]);
    assert.deepStrictEqual(cursorMoveEditor.diagnosticsByLine, [[{ startCol: 0, endCol: 5, severity: 'warning', message: 'x' }], []]);

    const stateOnlyEditor = createMockEditor();
    const stateOnlyResult = applyCoreRuntimeState.call(stateOnlyEditor, {
        text: 'alpha\nbeta\n',
        cursor: 5,
        selectionStart: 2,
        selectionEnd: 5,
        preferredCursorColumn: 5,
        isOverwriteMode: true,
        undoStack: [],
        redoStack: [],
    });

    assert.equal(stateOnlyResult, true);
    assert.equal(stateOnlyEditor.updateLinesCallCount, 0);
    assert.equal(stateOnlyEditor.updateTextCallCount, 0);
    assert.equal(stateOnlyEditor.isOverwriteMode, true);
    assert.equal(stateOnlyEditor.selectionStart, 2);
    assert.equal(stateOnlyEditor.selectionEnd, 5);
    assert.deepStrictEqual(stateOnlyEditor.tokensByLine, [[{ startCol: 0, endCol: 5, type: 'variable' }], []]);
    assert.deepStrictEqual(stateOnlyEditor.diagnosticsByLine, [[{ startCol: 0, endCol: 5, severity: 'warning', message: 'x' }], []]);

    const textEditEditor = createMockEditor();
    const textEditResult = applyCoreRuntimeState.call(textEditEditor, {
        text: 'alpha!\nbeta\n',
        cursor: 6,
        selectionStart: 6,
        selectionEnd: 6,
        preferredCursorColumn: 6,
        isOverwriteMode: false,
        undoStack: [{}],
        redoStack: [],
    });

    assert.equal(textEditResult, true);
    assert.equal(textEditEditor.updateLinesCallCount, 1);
    assert.equal(textEditEditor.updateTextCallCount, 1);

    return {
        ok: true,
        checks: [
            'cursor move preserves language render caches',
            'selection and overwrite updates preserve language render caches',
            'text edit still refreshes line caches and provider text',
        ],
    };
}

if (require.main === module) {
    try {
        const result = runSurfaceRegression();
        process.stdout.write(JSON.stringify(result, null, 2) + '\n');
    } catch (error) {
        console.error(error && error.stack ? error.stack : String(error));
        process.exit(1);
    }
}

module.exports = {
    runSurfaceRegression,
};
