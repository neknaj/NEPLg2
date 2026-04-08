// @ts-nocheck
"use strict";
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
        console.log("[CanvasEditor] Initializing v2 (with setFontSize)");
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
        this.fontSize = options.fontSize || 14;
        this.fontFamily = options.fontFamily || '"HackGenConsoleNF", "JetBrains Mono", Consolas, monospace';
        this.font = `${this.fontSize}px ${this.fontFamily}`;
        // Initial measurement
        this.ctx.font = this.font;
        const metrics = this.ctx.measureText('M');
        const h_width = metrics.width;
        this.geom = {
            padding: 10,
            lineHeight: Math.round(this.fontSize * 1.4),
            gutterWidth: Math.round(h_width * 4.5),
            h_width: h_width,
            z_width: h_width * 2
        };
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
        // renderer 側が参照する「別名キー」を用意する
        // ガター
        this.colors.gutterBg = this.colors.gutterBackground;
        this.colors.foldMarker = this.colors.lineNumberActive;
        this.colors.foldMarkerHover = this.colors.lineNumber;

        // カレント行の下線（元の見た目に寄せて薄めにする）
        this.colors.currentLine = 'rgba(255, 255, 255, 0.18)';

        // 選択/一致/括弧など
        this.colors.bracketHighlight = this.colors.occurrenceHighlight;

        // 空白ハイライト（キー名の互換）
        this.colors.trailingSpaceHighlight = this.colors.trailingSpace;
        this.colors.fullWidthSpaceHighlight = this.colors.fullWidthSpace;
        this.colors.tabHighlight = this.colors.tab;
        this.colors.whitespaceHighlight = this.colors.whitespaceSymbol;

        // 診断（エラー/警告）
        this.colors.diagnosticError = this.colors.errorUnderline;
        this.colors.diagnosticWarning = this.colors.warningUnderline;

        // インデント帯（白すぎ対策：白系ではなく、落ち着いた青みグレーで薄く）
        this.colors.indentHighlight1 = 'rgba(58, 67, 88, 0.18)';
        this.colors.indentHighlight2 = 'rgba(58, 67, 88, 0.10)';

        // Editor State
        this.text = '';
        this.lines = [];
        this.cursor = 0;
        this.selectionStart = 0;
        this.selectionEnd = 0;
        this.corePreferredCursorColumn = null;
        this.scrollX = 0;
        this.scrollY = 0;
        this.isFocused = false;
        this.isComposing = false;
        this.compositionText = '';
        this.cursorBlinkState = true;
        this.blinkInterval = 500;
        this.preferredCursorX = -1;
        this.isOverwriteMode = false;
        this.isEditable = true;
        this.visibleLines = 0;
        this.lineYPositions = [];
        this.lineStartIndices = [];

        // 言語機能の描画向けキャッシュ（行ごと）
        this.tokensByLine = [];
        this.diagnosticsByLine = [];
        this.foldingRangeByStartLine = new Map();
        this.undoStack = [];
        this.redoStack = [];
        this.foldedLines = new Set();
        // Language-related State
        this.languageProvider = null;
        this.tokens = [];
        this.diagnostics = [];
        this.langConfig = { highlightWhitespace: false, highlightIndent: false };
        this.highlightedOccurrences = [];
        this.bracketHighlights = [];
        this.foldingRanges = [];
        // Sub-components
        this.utils = new EditorUtils(this.geom);
        this.renderer = new EditorRenderer(this);
        this.inputHandler = new EditorInputHandler(this);
        this.domUI = new EditorDOMUI(this, domElements);
        this.onCursorChange = options.onCursorChange || null;
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
            // 1) トークン/診断は「startIndex」でソートしておく（描画側での走査を単純化）
            this.tokens = (data.tokens || []).slice().sort((a, b) => a.startIndex - b.startIndex);
            this.diagnostics = (data.diagnostics || []).slice().sort((a, b) => a.startIndex - b.startIndex);

            // 2) 折り畳み範囲
            this.foldingRanges = data.foldingRanges || [];
            this.foldingRangeByStartLine = new Map();
            for (const r of this.foldingRanges) {
                this.foldingRangeByStartLine.set(r.startLine, r);
            }

            // 3) 言語設定
            this.langConfig = { ...this.langConfig, ...data.config };

            // 4) 行ごとのセグメント（高速描画用）を再構築
            this.rebuildLanguageRenderCaches();

            this.domUI.updateProblemsPanel();
        });
        // Clear previous language-specific state
        this.tokens = [];
        this.diagnostics = [];
        this.foldingRanges = [];
        this.foldingRangeByStartLine = new Map();
        this.tokensByLine = [];
        this.diagnosticsByLine = [];
        this.langConfig = {};
        this.highlightedOccurrences = [];
        this.bracketHighlights = [];
        this.domUI.updateProblemsPanel();
    }
    /**
     * エディタのテキストコンテンツを完全に置き換え、状態をリセットします。
     * @param {string} text - 新しいテキストコンテンツ
     */
    setText(text) {
        this.applyResolvedEditorState({
            text,
            cursor: 0,
            selectionStart: 0,
            selectionEnd: 0,
        }, {
            clearHistory: true,
            clearFolds: true,
            resetScroll: true,
            clearDerivedHighlights: true,
        });
    }
    /**
     * 言語プロバイダにテキストの更新を通知します。
     * @param {string} text - 更新されたテキスト
     */
    updateText(text) {
        if (this.languageProvider) {
            this.languageProvider.updateText(this.normalizeEditorText(text));
        }
    }
    resizeEditor() {
        const container = this.canvas.parentElement;
        if (!container)
            return;
        const dpr = window.devicePixelRatio || 1;
        const rect = container.getBoundingClientRect();
        const newWidth = Math.round(rect.width * dpr);
        const newHeight = Math.round(rect.height * dpr);
        if (this.canvas.width === newWidth && this.canvas.height === newHeight) {
            return;
        }
        this.canvas.style.width = `${rect.width}px`;
        this.canvas.style.height = `${rect.height}px`;
        this.canvas.width = newWidth;
        this.canvas.height = newHeight;
        this.ctx.setTransform(1, 0, 0, 1, 0, 0);
        this.ctx.scale(dpr, dpr);
        this.ctx.font = this.font;
        this.ctx.textBaseline = 'middle';
        this.visibleLines = Math.floor((rect.height - this.geom.padding * 2) / this.geom.lineHeight);
        this.scrollToCursor();
    }
    setFontSize(size) {
        this.fontSize = size;
        this.font = `${this.fontSize}px ${this.fontFamily}`;
        this.ctx.font = this.font;
        const metrics = this.ctx.measureText('M');
        const h_width = metrics.width;
        this.geom.h_width = h_width;
        this.geom.z_width = h_width * 2;
        this.geom.lineHeight = Math.round(this.fontSize * 1.4);
        this.geom.gutterWidth = Math.round(h_width * 4.5);
        this.utils.clearCache();
        this.resizeEditor();
    }
    focus() { if (this.isFocused)
        return; this.isFocused = true; this.textarea.focus(); this.resetCursorBlink(); }
    blur() { this.isFocused = false; this.textarea.blur(); this.domUI.hidePopup(); this.domUI.hideCompletion(); }
    setEditable(editable) {
        this.isEditable = Boolean(editable);
        this.textarea.readOnly = !this.isEditable;
        if (!this.isEditable) {
            this.domUI.hideCompletion();
        }
    }
    getEditable() {
        return this.isEditable;
    }
    normalizeEditorText(text) {
        return String(text ?? '').replace(/\r\n?/g, '\n');
    }
    getCoreBridge() {
        return typeof window !== 'undefined' ? window.NEPLPlaygroundEditorCore || null : null;
    }
    getCoreState() {
        return {
            text: this.text,
            cursor: this.cursor,
            selectionStart: this.selectionStart,
            selectionEnd: this.selectionEnd,
            preferredCursorColumn: this.corePreferredCursorColumn,
            isOverwriteMode: this.isOverwriteMode,
            undoStack: this.undoStack || [],
            redoStack: this.redoStack || [],
        };
    }
    applyResolvedEditorState(nextState, options = {}) {
        if (!nextState) {
            return false;
        }
        const normalizedText = this.normalizeEditorText(nextState.text);
        const textChanged = normalizedText !== this.text;
        this.text = normalizedText;
        this.cursor = Math.max(0, Math.min(this.text.length, Number(nextState.cursor ?? 0)));
        this.selectionStart = Math.max(0, Math.min(this.text.length, Number(nextState.selectionStart ?? this.cursor)));
        this.selectionEnd = Math.max(0, Math.min(this.text.length, Number(nextState.selectionEnd ?? this.cursor)));
        this.corePreferredCursorColumn = Object.prototype.hasOwnProperty.call(nextState, 'preferredCursorColumn')
            ? nextState.preferredCursorColumn ?? null
            : null;
        if (Object.prototype.hasOwnProperty.call(nextState, 'isOverwriteMode')) {
            this.isOverwriteMode = Boolean(nextState.isOverwriteMode);
        }
        if (Array.isArray(nextState.undoStack)) {
            this.undoStack = nextState.undoStack;
        }
        if (Array.isArray(nextState.redoStack)) {
            this.redoStack = nextState.redoStack;
        }
        if (options.clearHistory) {
            this.undoStack = [];
            this.redoStack = [];
        }
        if (options.clearFolds) {
            this.foldedLines.clear();
        }
        if (options.resetScroll) {
            this.scrollX = 0;
            this.scrollY = 0;
        }
        if (options.resetPreferredCursorX !== false) {
            this.preferredCursorX = -1;
        }
        if (textChanged) {
            this.updateLines();
        }
        if (options.clearDerivedHighlights) {
            this.highlightedOccurrences = [];
            this.bracketHighlights = [];
        }
        this.scrollToCursor();
        this.resetCursorBlink();
        if (textChanged) {
            this.updateText(this.text);
        }
        this.updateOccurrencesHighlight();
        this.updateBracketMatching();
        if (this.onCursorChange) {
            this.onCursorChange(this.cursor);
        }
        return true;
    }
    replaceTextRange(start, end, replacement, selectionStart, selectionEnd, options = {}) {
        const rangeStart = Math.max(0, Math.min(this.text.length, Number(start ?? 0)));
        const rangeEnd = Math.max(rangeStart, Math.min(this.text.length, Number(end ?? rangeStart)));
        const normalizedReplacement = this.normalizeEditorText(replacement);
        if (options.recordHistory !== false) {
            this.recordHistory();
        }
        const nextText = this.text.slice(0, rangeStart) + normalizedReplacement + this.text.slice(rangeEnd);
        const fallbackCursor = rangeStart + normalizedReplacement.length;
        const nextSelectionStart = selectionStart ?? fallbackCursor;
        const nextSelectionEnd = selectionEnd ?? nextSelectionStart;
        return this.applyResolvedEditorState({
            text: nextText,
            cursor: nextSelectionEnd,
            selectionStart: nextSelectionStart,
            selectionEnd: nextSelectionEnd,
        }, options);
    }
    applyCoreRuntimeState(runtimeState) {
        return this.applyResolvedEditorState(runtimeState);
    }
    applyCoreStateCommand(command) {
        const bridge = this.getCoreBridge();
        if (!command || !command.kind || !bridge || typeof bridge.reduceEditorCommand !== 'function') {
            return false;
        }
        const nextState = bridge.reduceEditorCommand(this.getCoreState(), command);
        return this.applyCoreRuntimeState(nextState);
    }
    // --- Text and State Manipulation ---
    insertText(newText) {
        newText = this.normalizeEditorText(newText);
        const { start, end } = this.getSelectionRange();
        const replaceStart = this.hasSelection() ? start : this.cursor;
        const replaceEnd = this.hasSelection()
            ? end
            : (this.isOverwriteMode && this.cursor < this.text.length && newText !== '\n')
                ? Math.min(this.text.length, this.cursor + newText.length)
                : this.cursor;
        const nextCursor = replaceStart + newText.length;
        this.replaceTextRange(replaceStart, replaceEnd, newText, nextCursor, nextCursor);
    }
    deleteSelection(history = true) {
        if (history) {
            this.recordHistory();
        }
        if (!this.hasSelection())
            return;
        const { start, end } = this.getSelectionRange();
        this.replaceTextRange(start, end, '', start, start, { recordHistory: false });
    }
    setCursor(index, resetX = true) {
        this.cursor = Math.max(0, Math.min(this.text.length, index));
        if (resetX) {
            this.preferredCursorX = -1;
        }
        this.scrollToCursor();
        this.resetCursorBlink();
        this.updateOccurrencesHighlight();
        this.updateBracketMatching();
        if (this.onCursorChange)
            this.onCursorChange(this.cursor);
    }
    updateLines() {
        this.text = this.normalizeEditorText(this.text);
        this.lines = this.text.split(/\r\n|\n|\r/);

        // 行の開始インデックス（文字インデックス→行/列変換や描画の高速化に使う）
        this.lineStartIndices = new Array(this.lines.length);
        let index = 0;
        for (let i = 0; i < this.lines.length; i++) {
            this.lineStartIndices[i] = index;
            index += this.lines[i].length + 1; // +1 は改行文字
        }

        // テキスト更新直後は言語側キャッシュが不整合になりやすいので一旦破棄
        this.tokensByLine = [];
        this.diagnosticsByLine = [];
        this.foldingRangeByStartLine = new Map();
    }

    /**
     * 文字インデックスを (row, col) に変換します（lineStartIndicesを使って二分探索）。
     * getPosFromIndex より高速で、長いファイルで効果が出ます。
     * @param {number} index - 文字インデックス
     * @returns {{row: number, col: number}} 行と列
     */
    indexToRowCol(index) {
        const starts = this.lineStartIndices;
        if (!starts || starts.length === 0) {
            return this.utils.getPosFromIndex(index, this.lines);
        }

        // upper_bound(starts, index) - 1
        let lo = 0;
        let hi = starts.length - 1;
        while (lo <= hi) {
            const mid = (lo + hi) >>> 1;
            if (starts[mid] <= index) {
                lo = mid + 1;
            }
            else {
                hi = mid - 1;
            }
        }
        const row = Math.max(0, lo - 1);
        const colRaw = index - starts[row];
        const col = Math.max(0, Math.min(this.lines[row].length, colRaw));
        return { row, col };
    }

    /**
     * 言語機能（トークン/診断）を行ごとのセグメントに変換し、描画を高速化します。
     * tokensByLine[row] = [{startCol, endCol, type}, ...]
     * diagnosticsByLine[row] = [{startCol, endCol, severity, message}, ...]
     */
    rebuildLanguageRenderCaches() {
        const lineCount = this.lines.length;

        const buildSegments = (items, build) => {
            const out = Array.from({ length: lineCount }, () => []);
            for (const item of items) {
                const startRC = this.indexToRowCol(item.startIndex);
                const endRC = this.indexToRowCol(item.endIndex);

                for (let r = startRC.row; r <= endRC.row; r++) {
                    const lineLen = this.lines[r].length;
                    const startCol = (r === startRC.row) ? startRC.col : 0;
                    const endCol = (r === endRC.row) ? endRC.col : lineLen;

                    const s = Math.max(0, Math.min(lineLen, startCol));
                    const e = Math.max(0, Math.min(lineLen, endCol));

                    if (s < e) {
                        out[r].push(build(item, s, e));
                    }
                }
            }

            // 描画側で単純な前進走査をするためにソート
            for (const list of out) {
                list.sort((a, b) => a.startCol - b.startCol);
            }
            return out;
        };

        this.tokensByLine = buildSegments(this.tokens, (t, s, e) => ({
            startCol: s,
            endCol: e,
            type: t.type
        }));

        this.diagnosticsByLine = buildSegments(this.diagnostics, (d, s, e) => ({
            startCol: s,
            endCol: e,
            severity: d.severity,
            message: d.message
        }));
    }

    hasSelection() { return this.selectionStart !== this.selectionEnd; }
    getSelectionRange() { return { start: Math.min(this.selectionStart, this.selectionEnd), end: Math.max(this.selectionStart, this.selectionEnd) }; }
    // --- Cursor Movement Logic ---
    handleArrowKeys(e) {
        if (this.hasSelection() && !e.shiftKey) {
            const selection = this.getSelectionRange();
            switch (e.key) {
                case 'ArrowLeft':
                case 'ArrowUp':
                    this.setCursor(selection.start);
                    break;
                case 'ArrowRight':
                case 'ArrowDown':
                    this.setCursor(selection.end);
                    break;
            }
            this.selectionStart = this.selectionEnd = this.cursor;
            return;
        }
        switch (e.key) {
            case 'ArrowLeft':
                if (this.cursor > 0)
                    this.setCursor(this.cursor - 1);
                break;
            case 'ArrowRight':
                if (this.cursor < this.text.length)
                    this.setCursor(this.cursor + 1);
                break;
            case 'ArrowUp':
                this.moveCursorLine(-1);
                break;
            case 'ArrowDown':
                this.moveCursorLine(1);
                break;
        }
        if (!e.shiftKey) {
            this.selectionStart = this.selectionEnd = this.cursor;
        }
        else {
            this.selectionEnd = this.cursor;
        }
        this.updateOccurrencesHighlight();
    }
    moveCursorLine(direction) {
        const { row, col } = this.utils.getPosFromIndex(this.cursor, this.lines);
        if (this.preferredCursorX < 0) {
            this.preferredCursorX = this.utils.getXFromCol(this.lines[row], col);
        }
        const newRow = Math.max(0, Math.min(this.lines.length - 1, row + direction));
        if (newRow === row) {
            this.setCursor(direction < 0 ? 0 : this.text.length);
            return;
        }
        const targetLine = this.lines[newRow];

        // O(n) の総当たりを避ける：prefix sum + 二分探索で列を推定
        const newCol = this.utils.getNearestColFromX(targetLine, this.preferredCursorX);

        // lineStartIndices があれば O(1) で行開始インデックスが取れる
        this.setCursor(this.lineStartIndices[newRow] + newCol, false);
    }
    handleHomeEndKeys(e) {
        const { row, col } = this.utils.getPosFromIndex(this.cursor, this.lines);
        const line = this.lines[row];
        let newCol = col;
        if (e.key === 'Home') {
            const indentEndCol = line.match(/^\s*/)[0].length;
            newCol = (col !== indentEndCol && indentEndCol !== line.length) ? indentEndCol : 0;
        }
        else {
            newCol = line.length;
        }
        this.setCursor(this.lineStartIndices[row] + newCol);
        if (!e.shiftKey) {
            this.selectionStart = this.selectionEnd = this.cursor;
        }
        else {
            this.selectionEnd = this.cursor;
        }
        this.updateOccurrencesHighlight();
    }
    handlePageKeys(e) {
        const direction = e.key === 'PageUp' ? -1 : 1;
        const { row } = this.utils.getPosFromIndex(this.cursor, this.lines);
        if (this.preferredCursorX < 0) {
            const col = this.utils.getPosFromIndex(this.cursor, this.lines).col;
            this.preferredCursorX = this.utils.getXFromCol(this.lines[row], col);
        }
        const newRow = Math.max(0, Math.min(this.lines.length - 1, row + direction * this.visibleLines));
        const targetLine = this.lines[newRow];

        // O(n) の総当たりを避ける：prefix sum + 二分探索で列を推定
        const newCol = this.utils.getNearestColFromX(targetLine, this.preferredCursorX);

        // lineStartIndices があれば O(1) で行開始インデックスが取れる
        this.setCursor(this.lineStartIndices[newRow] + newCol, false);
        if (!e.shiftKey) {
            this.selectionStart = this.selectionEnd = this.cursor;
        }
        else {
            this.selectionEnd = this.cursor;
        }
        this.updateOccurrencesHighlight();
    }
    // --- Feature Logic ---
    async updateOccurrencesHighlight() {
        if (!this.languageProvider || this.hasSelection()) {
            if (this.highlightedOccurrences.length > 0)
                this.highlightedOccurrences = [];
            this.domUI.hideCompletion();
            return;
        }
        const occurrences = await this.languageProvider.getOccurrences(this.cursor);
        this.highlightedOccurrences = (occurrences || []).slice().sort((a, b) => a.startIndex - b.startIndex);
    }
    async updateBracketMatching() {
        if (!this.languageProvider) {
            this.bracketHighlights = [];
            return;
        }
        const matches = await this.languageProvider.getBracketMatch(this.cursor);
        this.bracketHighlights = (matches || []).slice().sort((a, b) => a.startIndex - b.startIndex);
    }
    scrollToCursor() {
        const rect = this.canvas.parentElement.getBoundingClientRect();
        const { x: cursorX, y: cursorY } = this.utils.getCursorCoords(this.cursor, this.lines, this.lineYPositions);
        if (cursorY < 0)
            return; // Cursor is in a folded line
        const visibleTop = this.scrollY;
        const visibleBottom = this.scrollY + rect.height;
        if (cursorY < visibleTop)
            this.scrollY = cursorY;
        else if (cursorY + this.geom.lineHeight > visibleBottom)
            this.scrollY = cursorY + this.geom.lineHeight - rect.height;
        const visibleLeft = this.scrollX + this.geom.gutterWidth;
        const visibleRight = this.scrollX + rect.width - this.geom.padding;
        if (cursorX < visibleLeft)
            this.scrollX = cursorX - this.geom.gutterWidth - this.geom.padding;
        else if (cursorX > visibleRight)
            this.scrollX = cursorX - rect.width + this.geom.padding;
        this.scrollX = Math.max(0, this.scrollX);
    }
    resetCursorBlink() { this.cursorBlinkState = true; this.renderer.lastBlinkTime = performance.now(); }
    updateTextareaPosition() {
        if (!this.isFocused)
            return;
        const coords = this.utils.getCursorCoords(this.cursor, this.lines, this.lineYPositions);
        if (coords.y < 0)
            return; // Cursor is in folded code, hide textarea
        const relativeX = coords.x - this.scrollX;
        const relativeY = coords.y - this.scrollY;
        this.textarea.style.left = `${relativeX}px`;
        this.textarea.style.top = `${relativeY}px`;
        this.textarea.style.height = `${this.geom.lineHeight}px`;
        this.textarea.style.lineHeight = `${this.geom.lineHeight}px`;
        this.textarea.style.font = this.font;
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
        if (lastState && lastState.text === state.text && lastState.cursor === state.cursor)
            return;
        this.undoStack.push(state);
        if (this.undoStack.length > 100)
            this.undoStack.shift();
    }
    applyState(state) {
        if (!state)
            return;
        this.applyResolvedEditorState({
            text: state.text,
            cursor: state.cursor,
            selectionStart: state.selectionStart,
            selectionEnd: state.selectionEnd,
        }, {
            clearDerivedHighlights: true,
        });
    }
    undo() {
        if (this.undoStack.length === 0)
            return;
        const currentState = { text: this.text, cursor: this.cursor, selectionStart: this.selectionStart, selectionEnd: this.selectionEnd };
        this.redoStack.push(currentState);
        const prevState = this.undoStack.pop();
        this.applyState(prevState);
    }
    redo() {
        if (this.redoStack.length === 0)
            return;
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
        const { start, end } = this.getSelectionRange();
        const newCursorPos = start + cursorOffsetFromStart;
        this.replaceTextRange(start, end, text, newCursorPos, newCursorPos);
    }
    applyTextEdit(newText, newSelectionStart, newSelectionEnd) {
        this.recordHistory();
        this.applyResolvedEditorState({
            text: newText,
            cursor: newSelectionEnd,
            selectionStart: newSelectionStart,
            selectionEnd: newSelectionEnd,
        }, {
            resetPreferredCursorX: false,
        });
    }
    toggleFold(startLine) {
        if (this.foldedLines.has(startLine)) {
            this.foldedLines.delete(startLine);
        }
        else {
            this.foldedLines.add(startLine);
            const { row } = this.utils.getPosFromIndex(this.cursor, this.lines);
            const range = this.foldingRanges.find(r => r.startLine === startLine);
            if (range && row > range.startLine && row <= range.endLine) {
                this.setCursor(this.lineStartIndices[range.startLine]);
                this.selectionStart = this.selectionEnd = this.cursor;
            }
        }
    }
    async triggerCompletion() {
        if (!this.languageProvider)
            return;
        const suggestions = await this.languageProvider.getCompletions(this.cursor);
        if (suggestions && suggestions.length > 0) {
            this.domUI.showCompletion(suggestions);
        }
        else {
            this.domUI.hideCompletion();
        }
    }
    acceptCompletion() {
        const selected = this.domUI.completionSuggestions[this.domUI.selectedSuggestionIndex];
        if (!selected) {
            this.domUI.hideCompletion();
            return;
        }
        let startIndex = this.cursor;
        while (startIndex > 0 && /[\w$]/.test(this.text[startIndex - 1])) {
            startIndex--;
        }
        const rawInsertText = selected.insertText || selected.label;
        const cursorPlaceholder = '$0';
        const placeholderIndex = rawInsertText.indexOf(cursorPlaceholder);
        const finalInsertText = placeholderIndex !== -1 ? rawInsertText.replace(cursorPlaceholder, '') : rawInsertText;
        const finalCursorOffset = placeholderIndex !== -1 ? placeholderIndex : rawInsertText.length;
        const newCursorPos = startIndex + finalCursorOffset;
        this.replaceTextRange(startIndex, this.cursor, finalInsertText, newCursorPos, newCursorPos);
        this.domUI.hideCompletion();
    }
}
//# sourceMappingURL=editor.js.map
