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
        onCursorChangeCallCount: 0,
        foldedLines: new Set([1]),
        scrollX: 12,
        scrollY: 34,
        languageProvider: {
            updateTextCallCount: 0,
            replaceDocumentTextCallCount: 0,
            updateText() {
                this.updateTextCallCount += 1;
            },
            replaceDocumentText() {
                this.replaceDocumentTextCallCount += 1;
            },
        },
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
        recordHistory() {
            this.recordHistoryCallCount = (this.recordHistoryCallCount || 0) + 1;
        },
        onCursorChange() {
            this.onCursorChangeCallCount += 1;
        },
    };
}

function runSurfaceRegression() {
    const CanvasEditor = loadCanvasEditorClass();
    const applyCoreRuntimeState = CanvasEditor.prototype.applyCoreRuntimeState;
    const applyResolvedEditorState = CanvasEditor.prototype.applyResolvedEditorState;
    const replaceTextRange = CanvasEditor.prototype.replaceTextRange;
    const setText = CanvasEditor.prototype.setText;
    const replaceDocumentText = CanvasEditor.prototype.replaceDocumentText;
    const rebuildLanguageRenderCaches = CanvasEditor.prototype.rebuildLanguageRenderCaches;

    const cursorMoveEditor = createMockEditor();
    cursorMoveEditor.applyResolvedEditorState = applyResolvedEditorState;
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
    stateOnlyEditor.applyResolvedEditorState = applyResolvedEditorState;
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
    assert.equal(stateOnlyEditor.updateBracketMatchingCallCount, 1);
    assert.equal(stateOnlyEditor.onCursorChangeCallCount, 1);
    assert.deepStrictEqual(stateOnlyEditor.tokensByLine, [[{ startCol: 0, endCol: 5, type: 'variable' }], []]);
    assert.deepStrictEqual(stateOnlyEditor.diagnosticsByLine, [[{ startCol: 0, endCol: 5, severity: 'warning', message: 'x' }], []]);

    const textEditEditor = createMockEditor();
    textEditEditor.applyResolvedEditorState = applyResolvedEditorState;
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

    const resetEditor = createMockEditor();
    resetEditor.applyResolvedEditorState = applyResolvedEditorState;
    resetEditor.replaceDocumentText = replaceDocumentText;
    const resetResult = applyResolvedEditorState.call(resetEditor, {
        text: 'gamma\n',
        cursor: 0,
        selectionStart: 0,
        selectionEnd: 0,
    }, {
        clearHistory: true,
        clearFolds: true,
        resetScroll: true,
        clearDerivedHighlights: true,
    });

    assert.equal(resetResult, true);
    assert.equal(resetEditor.scrollX, 0);
    assert.equal(resetEditor.scrollY, 0);
    assert.equal(resetEditor.undoStack.length, 0);
    assert.equal(resetEditor.redoStack.length, 0);
    assert.equal(resetEditor.foldedLines.size, 0);
    assert.equal(resetEditor.updateBracketMatchingCallCount, 1);
    assert.equal(resetEditor.onCursorChangeCallCount, 1);

    const replaceEditor = createMockEditor();
    replaceEditor.applyResolvedEditorState = applyResolvedEditorState;
    replaceEditor.selectionStart = 1;
    replaceEditor.selectionEnd = 4;
    const replaceResult = replaceTextRange.call(replaceEditor, 1, 4, 'ZZ', 3, 3);

    assert.equal(replaceResult, true);
    assert.equal(replaceEditor.recordHistoryCallCount, 1);
    assert.equal(replaceEditor.text, 'aZZa\nbeta\n');
    assert.equal(replaceEditor.cursor, 3);
    assert.equal(replaceEditor.updateLinesCallCount, 1);
    assert.equal(replaceEditor.updateTextCallCount, 1);

    const setTextEditor = createMockEditor();
    setTextEditor.applyResolvedEditorState = applyResolvedEditorState;
    setTextEditor.replaceDocumentText = replaceDocumentText;
    setText.call(setTextEditor, 'delta\n');
    assert.equal(setTextEditor.languageProvider.replaceDocumentTextCallCount, 1);
    assert.equal(setTextEditor.languageProvider.updateTextCallCount, 0);

    const cacheEditor = createMockEditor();
    cacheEditor.tokens = [
        { startIndex: 0, endIndex: 4, type: 'keyword' },
        { startIndex: 2, endIndex: 6, type: 'function' },
        { startIndex: 6, endIndex: 7, type: 'punctuation' },
    ];
    cacheEditor.diagnostics = [
        { startIndex: 1, endIndex: 3, severity: 'warning', message: 'warn' },
        { startIndex: 2, endIndex: 5, severity: 'error', message: 'err' },
    ];
    cacheEditor.lines = ['abcdefg', ''];
    cacheEditor.lineStartIndices = [0, 8];
    cacheEditor.indexToRowCol = CanvasEditor.prototype.indexToRowCol;
    rebuildLanguageRenderCaches.call(cacheEditor);
    assert.deepStrictEqual(JSON.parse(JSON.stringify(cacheEditor.tokensByLine[0])), [
        { startCol: 0, endCol: 2, type: 'keyword' },
        { startCol: 2, endCol: 6, type: 'function' },
        { startCol: 6, endCol: 7, type: 'punctuation' },
    ]);
    assert.deepStrictEqual(JSON.parse(JSON.stringify(cacheEditor.diagnosticsByLine[0])), [
        { startCol: 1, endCol: 2, severity: 'warning', message: 'warn' },
        { startCol: 2, endCol: 5, severity: 'error', message: 'err' },
    ]);

    return {
        ok: true,
        checks: [
            'cursor move preserves language render caches',
            'selection and overwrite updates preserve language render caches',
            'reset-style updates clear stale highlights and notify cursor listeners',
            'selection replacement triggers a single provider update',
            'text edit still refreshes line caches and provider text',
            'setText uses full-document replace instead of incremental analysis',
            'render caches normalize overlapping token and diagnostic segments',
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
