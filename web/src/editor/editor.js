import { EditorUtils } from './editor-utils.js';
import { EditorRenderer } from './editor-renderer.js';
import { EditorInputHandler } from './editor-input-handler.js';
import { EditorDOMUI } from './editor-dom-ui.js';

export class CanvasEditor {
    constructor(canvas, textarea, domElements, options = {}) {
        this.canvas = canvas;
        this.textarea = textarea;
        this.ctx = canvas.getContext('2d', { alpha: false }); // Optimize

        this.font = '14px "JetBrains Mono", "Space Mono", monospace';
        this.geom = {
            padding: 10,
            lineHeight: 22,
            gutterWidth: 50,
            h_width: 8.4, // Will be calculated dynamically usually, but hardcoding for now based on 14px mono
            z_width: 16.8
        };

        // Theme - Rich Dark
        this.colors = {
            background: '#1e2229',
            text: '#e6edf3',
            cursor: '#58a6ff',
            selection: 'rgba(88, 166, 255, 0.3)',
            imeUnderline: '#58a6ff',
            gutterBackground: '#161b22',
            lineNumber: '#484f58',
            lineNumberActive: '#e6edf3',
            cursorLineBorder: '#30363d',
            tokenColors: {
                'keyword': '#ff7b72', // Red/Pink
                'string': '#a5d6ff', // Light Blue
                'comment': '#8b949e', // Grey
                'function': '#d2a8ff', // Purple
                'number': '#79c0ff', // Blue
                'operator': '#ff7b72',
                'variable': '#e6edf3'
            }
        };

        // State
        this.text = "";
        this.lines = [""];
        this.cursor = 0;
        this.selectionStart = 0;
        this.selectionEnd = 0;
        this.scrollX = 0;
        this.scrollY = 0;
        this.isFocused = false;
        this.isComposing = false;
        this.compositionText = "";
        this.cursorBlinkState = true;
        this.blinkInterval = 500;

        this.undoStack = [];
        this.redoStack = [];

        this.tokens = [];
        this.diagnostics = [];
        this.languageProvider = null;

        // Components
        this.utils = new EditorUtils(this.geom);
        this.renderer = new EditorRenderer(this);
        this.inputHandler = new EditorInputHandler(this);
        this.domUI = new EditorDOMUI(this, domElements);

        this.init();
    }

    init() {
        this.ctx.font = this.font;
        this.ctx.textBaseline = 'middle';

        // Measure char width for 'M' to set geom accurately
        const metrics = this.ctx.measureText('M');
        this.geom.h_width = metrics.width;
        this.geom.z_width = metrics.width * 2;

        this.inputHandler.bindEvents();
        requestAnimationFrame(this.renderer.renderLoop.bind(this.renderer));

        // Initial resize
        this.resizeEditor();
    }

    resizeEditor() {
        const parent = this.canvas.parentElement;
        if (!parent) return;
        const rect = parent.getBoundingClientRect();
        const dpr = window.devicePixelRatio || 1;

        this.canvas.width = rect.width * dpr;
        this.canvas.height = rect.height * dpr;
        this.canvas.style.width = `${rect.width}px`;
        this.canvas.style.height = `${rect.height}px`;

        this.ctx.scale(dpr, dpr);
        this.ctx.font = this.font;
        this.renderer.render(); // Force render
    }

    setText(text) {
        this.text = text;
        this.updateLines();
        this.cursor = 0;
        this.selectionStart = 0;
        this.selectionEnd = 0;
        this.scrollX = 0;
        this.scrollY = 0;

        if (this.languageProvider) {
            this.languageProvider.updateText(this.text);
        }
    }

    updateLines() {
        this.lines = this.text.split('\n');
    }

    updateTextareaPosition() {
        if (!this.isFocused) return;
        const coords = this.utils.getCursorCoords(this.cursor, this.lines, this.renderer.editor.lineYPositions); // access stored linesY
        if (!coords || coords.y === -1) return;

        const relativeX = coords.x - this.scrollX;
        const relativeY = coords.y - this.scrollY;

        this.textarea.style.left = `${relativeX}px`;
        this.textarea.style.top = `${relativeY}px`;

        // Popup position logic if needed
        if (this.domUI.completionList) {
            const list = this.domUI.completionList;
            if (!list.classList.contains('hidden')) {
                list.style.left = `${relativeX}px`;
                list.style.top = `${relativeY + this.geom.lineHeight}px`;
            }
        }
    }

    focus() {
        this.isFocused = true;
        this.textarea.focus();
        this.cursorBlinkState = true;
    }

    blur() {
        // this.isFocused = false; 
        // this.textarea.blur();
    }

    setCursor(index) {
        this.cursor = Math.max(0, Math.min(index, this.text.length));
        this.scrollIntoView();
        this.cursorBlinkState = true;
    }

    getSelectionRange() {
        return {
            start: Math.min(this.selectionStart, this.selectionEnd),
            end: Math.max(this.selectionStart, this.selectionEnd)
        };
    }

    hasSelection() {
        return this.selectionStart !== this.selectionEnd;
    }

    selectAll() {
        this.selectionStart = 0;
        this.selectionEnd = this.text.length;
        this.setCursor(this.text.length);
    }

    insertText(text) {
        this.recordHistory();
        if (this.hasSelection()) {
            const { start, end } = this.getSelectionRange();
            this.text = this.text.slice(0, start) + text + this.text.slice(end);
            this.setCursor(start + text.length);
        } else {
            this.text = this.text.slice(0, this.cursor) + text + this.text.slice(this.cursor);
            this.setCursor(this.cursor + text.length);
        }
        this.selectionStart = this.selectionEnd = this.cursor;
        this.updateLines();
        this.onChange();
    }

    handleBackspace() {
        if (this.hasSelection()) {
            this.insertText(""); // Delete selection
        } else if (this.cursor > 0) {
            this.recordHistory();
            this.text = this.text.slice(0, this.cursor - 1) + this.text.slice(this.cursor);
            this.setCursor(this.cursor - 1);
            this.selectionStart = this.selectionEnd = this.cursor;
            this.updateLines();
            this.onChange();
        }
    }

    handleEnterKey() {
        // Simple enter
        this.insertText('\n');
    }

    handleArrowKeys(e) {
        // Simplified navigation
        const { key, shiftKey } = e;
        let newPos = this.cursor;

        if (key === 'ArrowLeft') newPos--;
        if (key === 'ArrowRight') newPos++;
        if (key === 'ArrowUp') {
            const { row, col } = this.utils.getPosFromIndex(this.cursor, this.lines);
            if (row > 0) newPos = this.utils.getIndexFromPos(row - 1, col, this.lines); // dumb column keeping
        }
        if (key === 'ArrowDown') {
            const { row, col } = this.utils.getPosFromIndex(this.cursor, this.lines);
            if (row < this.lines.length - 1) newPos = this.utils.getIndexFromPos(row + 1, col, this.lines);
        }

        this.setCursor(newPos);
        if (shiftKey) {
            this.selectionEnd = this.cursor;
        } else {
            this.selectionStart = this.selectionEnd = this.cursor;
        }
    }

    scrollIntoView() {
        // Minimal scroll logic
        const coords = this.utils.getCursorCoords(this.cursor, this.lines, this.renderer.editor.lineYPositions || []);
        if (coords.y === -1) return;

        // ... (Implement scrolling update based on logic seen in original) ...
        // Re-using original logic simplified:
        const rect = this.canvas.parentElement.getBoundingClientRect();
        const visibleTop = this.scrollY;
        const visibleBottom = this.scrollY + rect.height;

        if (coords.y < visibleTop) this.scrollY = coords.y;
        else if (coords.y + this.geom.lineHeight > visibleBottom) this.scrollY = coords.y + this.geom.lineHeight - rect.height;
    }

    recordHistory() {
        // TODO: Undo/Redo
    }

    undo() { }
    redo() { }

    onChange() {
        if (this.languageProvider) this.languageProvider.updateText(this.text);
    }

    registerLanguageProvider(provider) {
        this.languageProvider = provider;
        provider.onUpdate((data) => {
            if (data.tokens) this.tokens = data.tokens;
            if (data.diagnostics) this.diagnostics = data.diagnostics;
        });
    }

    triggerCompletion() {
        // TODO
    }
}
