/* https://github.com/bem130/editorsample */

/**
 * @typedef {object} Token - シンタックスハイライト用のトークン情報
 * @property {number} startIndex
 * @property {number} endIndex
 * @property {string} type - 'keyword', 'string', 'comment', 'function' など
 */
/**
 * @typedef {object} Diagnostic - 診断情報（エラーや警告）
 * @property {number} startIndex
 * @property {number} endIndex
 * @property {string} message
 * @property {'error' | 'warning'} severity
 */
/**
 * @typedef {object} HoverInfo - ホバー時に表示する情報
 * @property {string} content - 表示するテキスト
 * @property {number} startIndex
 * @property {number} endIndex
 */
/**
 * @typedef {object} DefinitionLocation - 定義位置情報
 * @property {number} targetIndex - ジャンプ先の文字インデックス
 */
/**
 * @typedef {object} CompletionItem - 補完候補の情報
 * @property {string} label - 候補リストに表示されるテキスト
 * @property {string} type - 'keyword', 'variable', 'function', 'snippet'など
 * @property {string} insertText - 実際に挿入されるテキスト
 * @property {string} [detail] - 補完候補の追加情報
 */
/**
 * @typedef {object} LanguageConfiguration - 言語ごとの設定
 * @property {boolean} highlightWhitespace - 空白文字をハイライトするか
 * @property {boolean} highlightIndent - インデントをハイライトするか
 */
/**
 * @typedef {object} FoldingRange - 折り畳み範囲の情報
 * @property {number} startLine - 開始行番号 (0-indexed)
 * @property {number} endLine - 終了行番号 (0-indexed)
 * @property {string} placeholder - 折り畳み時に表示されるテキスト
 */

/**
 * Canvasベースのテキストエディタのコアクラス。
 * エディタの状態管理、コンポーネントの統括、および中心的なAPIを提供します。
 */
class CanvasEditor {
    constructor(canvas, textarea, domElements, options = {}) {
        // Core components
        this.canvas = canvas;
        this.textarea = textarea;
        this.ctx = canvas.getContext('2d');

        // Options
        this.options = {
            autoRender: options.autoRender !== false,
            bindEvents: options.bindEvents !== false
        };
        
        // Geometry and Styling
        this.font = '22px "Space Mono", "Noto Sans JP", monospace';
        this.geom = { padding: 10, lineHeight: 30, gutterWidth: 60, h_width: 13, z_width: 26 };
        this.colors = {
            background: '#050a0cff', text: '#abb2bf', cursor: '#528bff',
            selection: 'rgba(58, 67, 88, 0.8)', imeUnderline: '#abb2bf',
            occurrenceHighlight: 'rgba(92, 99, 112, 0.5)',
            indentation: ['rgba(255, 255, 255, 0.07)', 'rgba(255, 255, 255, 0.04)'],
            trailingSpace: 'rgba(255, 82, 82, 0.4)',
            fullWidthSpace: 'rgba(100, 150, 200, 0.2)',
            tab: 'rgba(100, 150, 200, 0.2)',
            whitespaceSymbol: '#4a505e', overwriteCursor: 'rgba(82, 139, 255, 0.5)',
            errorUnderline: 'red', warningUnderline: '#d19a66',
            gutterBackground: '#171a22ff', lineNumber: '#41454eff', lineNumberActive: '#bfc9daff',
            cursorLineBorder: 'rgba(255, 255, 255, 0.49)',
            tokenColors: {
                'keyword': '#c678dd', 'string': '#98c379', 'comment': '#5c6370',
                'function': '#61afef', 'number': '#d19a66', 'boolean': '#d19a66',
                'operator': '#56b6c2', 'regex': '#d19a66', 'property': '#e06c75',
                'punctuation': '#b3a5b0ff', 'variable': '#7da5f0ff',
                'heading': '#e06c75', 'bold': '#d19a66', 'italic': '#c678dd',
                'list': '#56b6c2', 'link': '#61afef', 'inline-code': '#98c379',
                'code-block': '#5c6370', 'default': '#b5b7bbff'
            }
        };

        // Editor State
        this.text = ''; this.lines = [];
        this.cursor = 0; this.selectionStart = 0; this.selectionEnd = 0;
        this.scrollX = 0; this.scrollY = 0;
        this.isFocused = false; this.isComposing = false; this.compositionText = '';
        this.cursorBlinkState = true; this.blinkInterval = 500;
        this.preferredCursorX = -1; this.isOverwriteMode = false;
        this.visibleLines = 0; this.lineYPositions = [];
        this.undoStack = []; this.redoStack = [];
        this.foldedLines = new Set();
        
        // Language-related State
        this.languageProvider = null;
        this.tokens = []; this.diagnostics = [];
        this.langConfig = { highlightWhitespace: false, highlightIndent: false };
        this.highlightedOccurrences = []; this.bracketHighlights = [];
        this.foldingRanges = [];

        // Sub-components
        this.utils = new EditorUtils(this.geom);
        this.renderer = new EditorRenderer(this);
        this.inputHandler = new EditorInputHandler(this);
        this.domUI = new EditorDOMUI(this, domElements);

        this.init();
    }

    init() {
        this.ctx.font = this.font;
        this.ctx.textBaseline = 'middle';
        this.geom.z_width = this.geom.h_width * 2;
        this.updateLines();

        if (this.options.bindEvents) {
            this.inputHandler.bindEvents();
        }

        if (this.options.autoRender) {
            requestAnimationFrame(this.renderer.renderLoop.bind(this.renderer));
        }
    }
    
    registerLanguageProvider(languageId, provider) {
        this.languageProvider = provider;
        this.languageProvider.onUpdate((data) => {
            this.tokens = data.tokens || [];
            this.diagnostics = data.diagnostics || [];
            this.foldingRanges = data.foldingRanges || [];
            this.langConfig = { ...this.langConfig, ...data.config };
            this.domUI.updateProblemsPanel();
        });
        // Clear previous language-specific state
        this.tokens = []; this.diagnostics = []; this.foldingRanges = [];
        this.langConfig = {}; this.highlightedOccurrences = [];
        this.bracketHighlights = []; this.domUI.updateProblemsPanel();
    }
    
    /**
     * エディタのテキストコンテンツを完全に置き換え、状態をリセットします。
     * @param {string} text - 新しいテキストコンテンツ
     */
    setText(text) {
        this.text = text; this.cursor = 0;
        this.selectionStart = 0; this.selectionEnd = 0;
        this.undoStack = []; this.redoStack = [];
        this.foldedLines.clear();
        this.scrollX = 0; this.scrollY = 0;
        this.updateLines();
        if (this.languageProvider) {
            this.languageProvider.updateText(this.text);
        }
        this.scrollToCursor();
    }

    /**
     * 言語プロバイダにテキストの更新を通知します。
     * @param {string} text - 更新されたテキスト
     */
    updateText(text) {
        if (this.languageProvider) {
            this.languageProvider.updateText(text);
        }
    }

    resizeEditor() {
        const container = this.canvas.parentElement;
        if (!container) return;
        const dpr = window.devicePixelRatio || 1;
        const rect = container.getBoundingClientRect();
        const newWidth = Math.round(rect.width * dpr);
        const newHeight = Math.round(rect.height * dpr);
        if (this.canvas.width === newWidth && this.canvas.height === newHeight) {
            return;
        }
        this.canvas.style.width = `${rect.width}px`; this.canvas.style.height = `${rect.height}px`;
        this.canvas.width = newWidth; this.canvas.height = newHeight;
        this.ctx.scale(dpr, dpr);
        this.ctx.font = this.font; this.ctx.textBaseline = 'middle';
        this.visibleLines = Math.floor((rect.height - this.geom.padding * 2) / this.geom.lineHeight);
        this.scrollToCursor();
    }

    focus() { if(this.isFocused) return; this.isFocused = true; this.textarea.focus(); this.resetCursorBlink(); }
    blur() { this.isFocused = false; this.textarea.blur(); this.domUI.hidePopup(); this.domUI.hideCompletion(); }
    
    // --- Text and State Manipulation ---

    insertText(newText) {
        this.recordHistory();
        if (this.hasSelection()) { this.deleteSelection(false); }
        if (this.isOverwriteMode && this.cursor < this.text.length && newText !== '\n') {
            const end = this.cursor + newText.length;
            this.text = this.text.slice(0, this.cursor) + newText + this.text.slice(end);
            this.setCursor(this.cursor + newText.length);
        } else {
            const prevCursor = this.cursor;
            this.text = this.text.slice(0, prevCursor) + newText + this.text.slice(prevCursor);
            this.setCursor(prevCursor + newText.length);
        }
        this.selectionStart = this.selectionEnd = this.cursor;
        this.updateLines(); this.updateText(this.text); this.updateOccurrencesHighlight();
    }

    deleteSelection(history = true) {
        if (history) { this.recordHistory(); }
        if(!this.hasSelection()) return;
        const { start } = this.getSelectionRange();
        this.text = this.text.slice(0, start) + this.text.slice(this.getSelectionRange().end);
        this.setCursor(start);
        this.selectionStart = this.selectionEnd = this.cursor;
        this.updateLines(); this.updateText(this.text); this.updateOccurrencesHighlight();
    }
    
    setCursor(index, resetX = true) {
        this.cursor = Math.max(0, Math.min(this.text.length, index));
        if (resetX) { this.preferredCursorX = -1; }
        this.scrollToCursor(); this.resetCursorBlink();
        this.updateOccurrencesHighlight(); this.updateBracketMatching();
    }
    
    updateLines() { this.lines = this.text.split('\n'); }
    hasSelection() { return this.selectionStart !== this.selectionEnd; }
    getSelectionRange() { return { start: Math.min(this.selectionStart, this.selectionEnd), end: Math.max(this.selectionStart, this.selectionEnd) }; }

    // --- Cursor Movement Logic ---
    
    handleArrowKeys(e) {
        if (this.hasSelection() && !e.shiftKey) {
            const selection = this.getSelectionRange();
            switch (e.key) {
                case 'ArrowLeft': case 'ArrowUp': this.setCursor(selection.start); break;
                case 'ArrowRight': case 'ArrowDown': this.setCursor(selection.end); break;
            }
            this.selectionStart = this.selectionEnd = this.cursor;
            return;
        }
        switch (e.key) {
            case 'ArrowLeft': if (this.cursor > 0) this.setCursor(this.cursor - 1); break;
            case 'ArrowRight': if (this.cursor < this.text.length) this.setCursor(this.cursor + 1); break;
            case 'ArrowUp': this.moveCursorLine(-1); break;
            case 'ArrowDown': this.moveCursorLine(1); break;
        }
        if (!e.shiftKey) { this.selectionStart = this.selectionEnd = this.cursor; }
        else { this.selectionEnd = this.cursor; }
        this.updateOccurrencesHighlight();
    }

    moveCursorLine(direction) {
        const { row, col } = this.utils.getPosFromIndex(this.cursor, this.lines);
        if (this.preferredCursorX < 0) {
            this.preferredCursorX = this.utils.measureText(this.lines[row].substring(0, col));
        }
        const newRow = Math.max(0, Math.min(this.lines.length - 1, row + direction));
        if (newRow === row) { this.setCursor(direction < 0 ? 0 : this.text.length); return; }
        
        const targetLine = this.lines[newRow];
        let minDelta = Infinity; let newCol = 0;
        for (let i = 0; i <= targetLine.length; i++) {
            const w = this.utils.measureText(targetLine.substring(0, i));
            const delta = Math.abs(this.preferredCursorX - w);
            if (delta < minDelta) { minDelta = delta; newCol = i; }
            else { break; }
        }
        this.setCursor(this.utils.getIndexFromPos(newRow, newCol, this.lines), false);
    }

    handleHomeEndKeys(e) {
        const { row, col } = this.utils.getPosFromIndex(this.cursor, this.lines);
        const line = this.lines[row]; let newCol = col;
        if (e.key === 'Home') {
            const indentEndCol = line.match(/^\s*/)[0].length;
            newCol = (col !== indentEndCol && indentEndCol !== line.length) ? indentEndCol : 0;
        } else {
            newCol = line.length;
        }
        this.setCursor(this.utils.getIndexFromPos(row, newCol, this.lines));
        if (!e.shiftKey) { this.selectionStart = this.selectionEnd = this.cursor; }
        else { this.selectionEnd = this.cursor; }
        this.updateOccurrencesHighlight();
    }

    handlePageKeys(e) {
        const direction = e.key === 'PageUp' ? -1 : 1;
        const { row } = this.utils.getPosFromIndex(this.cursor, this.lines);
        if (this.preferredCursorX < 0) {
            this.preferredCursorX = this.utils.measureText(this.lines[row].substring(0, this.utils.getPosFromIndex(this.cursor, this.lines).col));
        }
        const newRow = Math.max(0, Math.min(this.lines.length - 1, row + direction * this.visibleLines));
        const targetLine = this.lines[newRow]; let minDelta = Infinity; let newCol = 0;
        for (let i = 0; i <= targetLine.length; i++) {
            const w = this.utils.measureText(targetLine.substring(0, i));
            const delta = Math.abs(this.preferredCursorX - w);
            if (delta < minDelta) { minDelta = delta; newCol = i; }
            else { break; }
        }
        this.setCursor(this.utils.getIndexFromPos(newRow, newCol, this.lines), false);
        if (!e.shiftKey) { this.selectionStart = this.selectionEnd = this.cursor; }
        else { this.selectionEnd = this.cursor; }
        this.updateOccurrencesHighlight();
    }
    
    // --- Feature Logic ---

    async updateOccurrencesHighlight() {
        if (!this.languageProvider || this.hasSelection()) {
            if (this.highlightedOccurrences.length > 0) this.highlightedOccurrences = [];
            this.domUI.hideCompletion();
            return;
        }
        const occurrences = await this.languageProvider.getOccurrences(this.cursor);
        this.highlightedOccurrences = occurrences || [];
    }

    async updateBracketMatching() {
        if (!this.languageProvider) { this.bracketHighlights = []; return; }
        const matches = await this.languageProvider.getBracketMatch(this.cursor);
        this.bracketHighlights = matches || [];
    }

    scrollToCursor() {
        const rect = this.canvas.parentElement.getBoundingClientRect();
        const { x: cursorX, y: cursorY } = this.utils.getCursorCoords(this.cursor, this.lines, this.lineYPositions);
        if (cursorY < 0) return; // Cursor is in a folded line
        const visibleTop = this.scrollY;
        const visibleBottom = this.scrollY + rect.height;
        if (cursorY < visibleTop) this.scrollY = cursorY;
        else if (cursorY + this.geom.lineHeight > visibleBottom) this.scrollY = cursorY + this.geom.lineHeight - rect.height;

        const visibleLeft = this.scrollX + this.geom.gutterWidth;
        const visibleRight = this.scrollX + rect.width - this.geom.padding;
        if (cursorX < visibleLeft) this.scrollX = cursorX - this.geom.gutterWidth - this.geom.padding;
        else if (cursorX > visibleRight) this.scrollX = cursorX - rect.width + this.geom.padding;
        this.scrollX = Math.max(0, this.scrollX);
    }
    
    resetCursorBlink() { this.cursorBlinkState = true; this.renderer.lastBlinkTime = performance.now(); }

    updateTextareaPosition() {
        if(!this.isFocused) return;
        const coords = this.utils.getCursorCoords(this.cursor, this.lines, this.lineYPositions);
        if (coords.y < 0) return; // Cursor is in folded code, hide textarea
        const relativeX = coords.x - this.scrollX;
        const relativeY = coords.y - this.scrollY;
        this.textarea.style.left = `${relativeX}px`; this.textarea.style.top = `${relativeY}px`;
        if (this.domUI.isCompletionVisible) {
            this.domUI.completionList.style.left = `${relativeX}px`;
            this.domUI.completionList.style.top = `${relativeY + this.geom.lineHeight}px`;
        }
    }
    
    // --- Undo/Redo ---

    recordHistory() {
        this.redoStack = [];
        const state = { text: this.text, cursor: this.cursor, selectionStart: this.selectionStart, selectionEnd: this.selectionEnd };
        const lastState = this.undoStack[this.undoStack.length - 1];
        if (lastState && lastState.text === state.text && lastState.cursor === state.cursor) return;
        this.undoStack.push(state);
        if (this.undoStack.length > 100) this.undoStack.shift();
    }

    applyState(state) {
        if (!state) return;
        this.text = state.text; this.cursor = state.cursor;
        this.selectionStart = state.selectionStart; this.selectionEnd = state.selectionEnd;
        this.updateLines(); this.scrollToCursor(); this.resetCursorBlink();
        this.updateText(this.text); this.updateOccurrencesHighlight();
    }

    undo() {
        if (this.undoStack.length === 0) return;
        const currentState = { text: this.text, cursor: this.cursor, selectionStart: this.selectionStart, selectionEnd: this.selectionEnd };
        this.redoStack.push(currentState);
        const prevState = this.undoStack.pop();
        this.applyState(prevState);
    }

    redo() {
        if (this.redoStack.length === 0) return;
        const currentState = { text: this.text, cursor: this.cursor, selectionStart: this.selectionStart, selectionEnd: this.selectionEnd };
        this.undoStack.push(currentState);
        const nextState = this.redoStack.pop();
        this.applyState(nextState);
    }

    // --- Language-Specific Actions ---

    async handleEnterKey() {
        if (this.languageProvider) {
            const { start } = this.getSelectionRange();
            const result = await this.languageProvider.getIndentation(start);
            if (result && result.textToInsert !== undefined && result.cursorOffset !== undefined) {
                this.replaceSelectionAndSetCursor(result.textToInsert, result.cursorOffset);
                return;
            }
        }
        // Fallback for simple indentation
        const { row } = this.utils.getPosFromIndex(this.cursor, this.lines);
        const currentIndent = this.lines[row].match(/^\s*/)[0];
        this.insertText('\n' + currentIndent);
    }
    
    replaceSelectionAndSetCursor(text, cursorOffsetFromStart) {
        this.recordHistory();
        const { start, end } = this.getSelectionRange();
        this.text = this.text.slice(0, start) + text + this.text.slice(end);
        const newCursorPos = start + cursorOffsetFromStart;
        this.setCursor(newCursorPos);
        this.selectionStart = this.selectionEnd = this.cursor;
        this.updateLines(); this.updateText(this.text); this.updateOccurrencesHighlight();
    }

    applyTextEdit(newText, newSelectionStart, newSelectionEnd) {
        this.recordHistory();
        this.text = newText;
        this.selectionStart = newSelectionStart;
        this.selectionEnd = newSelectionEnd;
        this.updateLines(); this.updateText(this.text);
        this.setCursor(this.selectionEnd, false);
        this.updateOccurrencesHighlight();
    }

    toggleFold(startLine) {
        if (this.foldedLines.has(startLine)) {
            this.foldedLines.delete(startLine);
        } else {
            this.foldedLines.add(startLine);
            const { row } = this.utils.getPosFromIndex(this.cursor, this.lines);
            const range = this.foldingRanges.find(r => r.startLine === startLine);
            if (range && row > range.startLine && row <= range.endLine) {
                this.setCursor(this.utils.getIndexFromPos(range.startLine, 0, this.lines));
                this.selectionStart = this.selectionEnd = this.cursor;
            }
        }
    }

    async triggerCompletion() {
        if (!this.languageProvider) return;
        const suggestions = await this.languageProvider.getCompletions(this.cursor);
        if (suggestions && suggestions.length > 0) {
            this.domUI.showCompletion(suggestions);
        } else {
            this.domUI.hideCompletion();
        }
    }
    
    acceptCompletion() {
        const selected = this.domUI.completionSuggestions[this.domUI.selectedSuggestionIndex];
        if (!selected) { this.domUI.hideCompletion(); return; }

        let startIndex = this.cursor;
        while (startIndex > 0 && /[\w$]/.test(this.text[startIndex - 1])) {
            startIndex--;
        }

        const rawInsertText = selected.insertText || selected.label;
        const cursorPlaceholder = '$0';
        const placeholderIndex = rawInsertText.indexOf(cursorPlaceholder);
        
        const finalInsertText = placeholderIndex !== -1 ? rawInsertText.replace(cursorPlaceholder, '') : rawInsertText;
        const finalCursorOffset = placeholderIndex !== -1 ? placeholderIndex : rawInsertText.length;

        const textBeforeSelection = this.text.slice(0, startIndex);
        const textAfterSelection = this.text.slice(this.cursor);
        
        this.recordHistory();
        this.text = textBeforeSelection + finalInsertText + textAfterSelection;
        const newCursorPos = startIndex + finalCursorOffset;
        this.setCursor(newCursorPos);
        this.selectionStart = this.selectionEnd = this.cursor;

        this.updateLines(); this.updateText(this.text);
        this.updateOccurrencesHighlight(); this.domUI.hideCompletion();
    }
}