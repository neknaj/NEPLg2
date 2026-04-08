// @ts-nocheck
"use strict";
/**
 * ユーザーからのすべての入力を処理します。
 * イベントリスナーを登録し、キーボード、マウス、その他のUIイベントを解釈して、
 * CanvasEditorの対応するアクションを呼び出します。
 */
class EditorInputHandler {
    /**
     * @param {CanvasEditor} editor - 親となるCanvasEditorのインスタンス
     */
    constructor(editor) {
        this.editor = editor;
        this.canvas = editor.canvas;
        this.textarea = editor.textarea;
        this.isDragging = false;
        this.hoverTimeout = null;
        this.lastHoverIndex = -1;
        this.lastHoverClientX = 0;
        this.lastHoverClientY = 0;
    }

    getCanvasEventPoint(e) {
        const rect = this.canvas.getBoundingClientRect();
        return {
            x: e.clientX - rect.left,
            y: e.clientY - rect.top,
        };
    }
    /**
     * エディタに必要なすべてのDOMイベントリスナーを登録します。
     */
    bindEvents() {
        this.canvas.addEventListener('mousedown', this.onMouseDown.bind(this));
        this.canvas.addEventListener('mousemove', this.onMouseMove.bind(this));
        this.canvas.addEventListener('mouseleave', () => {
            clearTimeout(this.hoverTimeout);
            this.editor.domUI.hidePopup();
            this.lastHoverIndex = -1;
        });
        window.addEventListener('mouseup', this.onMouseUp.bind(this));
        this.canvas.addEventListener('wheel', this.onWheel.bind(this));
        document.addEventListener('click', (e) => {
            const editorContainer = this.canvas.parentElement;
            const problemsContainer = this.editor.domUI.problemsPanel ? this.editor.domUI.problemsPanel.parentElement : null;
            const isClickInside = (editorContainer && editorContainer.contains(e.target)) ||
                (problemsContainer && problemsContainer.contains(e.target));
            if (!isClickInside) {
                this.editor.blur();
            }
        });
        this.textarea.addEventListener('input', this.onInput.bind(this));
        this.textarea.addEventListener('keydown', this.onKeydown.bind(this));
        this.textarea.addEventListener('compositionstart', () => {
            this.editor.isComposing = true;
            this.editor.domUI.hideCompletion();
        });
        this.textarea.addEventListener('compositionupdate', (e) => {
            this.editor.compositionText = e.data;
        });
        this.textarea.addEventListener('compositionend', (e) => {
            this.editor.isComposing = false;
            this.editor.compositionText = '';
            this.onInput({ target: { value: e.data } });
        });
        this.textarea.addEventListener('copy', this.onCopy.bind(this));
        this.textarea.addEventListener('paste', this.onPaste.bind(this));
        this.textarea.addEventListener('cut', this.onCut.bind(this));
        const observer = new ResizeObserver(() => this.editor.resizeEditor());
        observer.observe(this.canvas.parentElement);
    }
    onCopy(e) {
        e.preventDefault();
        if (!this.editor.hasSelection())
            return;
        const { start, end } = this.editor.getSelectionRange();
        const selectedText = this.editor.text.substring(start, end);
        e.clipboardData.setData('text/plain', selectedText);
    }
    onPaste(e) {
        e.preventDefault();
        const pasteText = e.clipboardData.getData('text/plain');
        if (pasteText) {
            this.editor.insertText(pasteText);
        }
    }
    onCut(e) {
        e.preventDefault();
        if (!this.editor.hasSelection())
            return;
        this.onCopy(e);
        this.editor.deleteSelection();
    }
    onMouseDown(e) {
        e.preventDefault();
        this.editor.focus();
        clearTimeout(this.hoverTimeout);
        this.editor.domUI.hidePopup();
        this.lastHoverIndex = -1;
        const point = this.getCanvasEventPoint(e);
        if (point.x < this.editor.geom.gutterWidth) {
            const clickedRow = this.editor.utils.getPosFromIndex(this.editor.utils.getCursorIndexFromCoords(point.x, point.y, this.editor.lines, this.editor.lineYPositions, this.editor.scrollX, this.editor.scrollY, true, this.editor.lineStartIndices), this.editor.lines).row;
            const range = this.editor.foldingRanges.find(r => r.startLine === clickedRow);
            if (range) {
                this.editor.toggleFold(clickedRow);
            }
            return;
        }
        this.isDragging = true;
        const pos = this.editor.utils.getCursorIndexFromCoords(point.x, point.y, this.editor.lines, this.editor.lineYPositions, this.editor.scrollX, this.editor.scrollY, false, this.editor.lineStartIndices);
        this.editor.setCursor(pos);
        this.editor.selectionStart = this.editor.cursor;
        this.editor.selectionEnd = this.editor.cursor;
        this.editor.domUI.hideCompletion();
    }
    onMouseMove(e) {
        const point = this.getCanvasEventPoint(e);
        const pos = this.editor.utils.getCursorIndexFromCoords(point.x, point.y, this.editor.lines, this.editor.lineYPositions, this.editor.scrollX, this.editor.scrollY, false, this.editor.lineStartIndices);
        this.lastHoverClientX = e.clientX;
        this.lastHoverClientY = e.clientY;
        if (this.isDragging) {
            this.editor.domUI.hidePopup();
            clearTimeout(this.hoverTimeout);
            this.lastHoverIndex = -1;
            this.editor.setCursor(pos);
            this.editor.selectionEnd = this.editor.cursor;
        }
        else {
            this.lastHoverIndex = pos;
            this.editor.domUI.hidePopup();
            clearTimeout(this.hoverTimeout);
            this.hoverTimeout = setTimeout(() => this.handleHover(pos, this.lastHoverClientX, this.lastHoverClientY), 1000);
        }
    }
    async handleHover(pos, clientX, clientY) {
        const diagnostic = this.editor.diagnostics.find(d => pos >= d.startIndex && pos < d.endIndex);
        if (diagnostic) {
            this.editor.domUI.showPopup(diagnostic.message, clientX, clientY);
            return;
        }
        if (!this.editor.languageProvider)
            return;
        const hoverInfo = await this.editor.languageProvider.getHoverInfo(pos);
        if (hoverInfo && hoverInfo.content && this.lastHoverIndex === pos && this.lastHoverClientX === clientX && this.lastHoverClientY === clientY) {
            this.editor.domUI.showPopup(hoverInfo.content, clientX, clientY);
        }
    }
    onMouseUp() {
        this.isDragging = false;
        this.editor.preferredCursorX = -1;
        this.editor.updateOccurrencesHighlight();
    }
    onWheel(e) {
        e.preventDefault();
        this.editor.domUI.hideCompletion();
        if (e.shiftKey) {
            // Treat vertical wheel as horizontal scroll when shift is held
            this.editor.scrollX += e.deltaY;
            this.editor.scrollX = Math.max(0, this.editor.scrollX);
        }
        else {
            const newScrollY = this.editor.scrollY + e.deltaY;
            const maxScrollY = Math.max(0, this.editor.lines.length * this.editor.geom.lineHeight - this.canvas.height + this.editor.geom.padding * 2);
            this.editor.scrollY = Math.max(0, Math.min(newScrollY, maxScrollY));
            // Standard horizontal scroll if device supports it
            this.editor.scrollX += e.deltaX;
            this.editor.scrollX = Math.max(0, this.editor.scrollX);
        }
    }
    onInput(e) {
        if (this.editor.isComposing)
            return;
        const newText = e.target.value;
        if (newText) {
            if (!this.editor.applyCoreStateCommand({ kind: 'insert_text', text: newText })) {
                this.editor.insertText(newText);
            }
            this.textarea.value = '';
            this.editor.triggerCompletion();
        }
    }
    async onKeydown(e) {
        if (this.editor.isComposing)
            return;
        const coreBridge = this.editor.getCoreBridge ? this.editor.getCoreBridge() : null;
        if (coreBridge && typeof coreBridge.mapKeyboardEventToCoreCommand === 'function') {
            const coreCommand = coreBridge.mapKeyboardEventToCoreCommand({
                key: e.key,
                ctrlKey: e.ctrlKey,
                metaKey: e.metaKey,
                shiftKey: e.shiftKey,
                altKey: e.altKey,
            });
            if (coreCommand && this.editor.applyCoreStateCommand(coreCommand)) {
                e.preventDefault();
                return;
            }
        }
        if (this.editor.domUI.isCompletionVisible) {
            switch (e.key) {
                case 'ArrowUp':
                    e.preventDefault();
                    this.editor.domUI.updateCompletionSelection(-1);
                    return;
                case 'ArrowDown':
                    e.preventDefault();
                    this.editor.domUI.updateCompletionSelection(1);
                    return;
                case 'Enter':
                case 'Tab':
                    e.preventDefault();
                    this.editor.acceptCompletion();
                    return;
                case 'Escape':
                    e.preventDefault();
                    this.editor.domUI.hideCompletion();
                    return;
            }
        }
        if ((e.ctrlKey || e.metaKey)) {
            switch (e.key.toLowerCase()) {
                case '/':
                    e.preventDefault();
                    if (this.editor.languageProvider) {
                        const { start, end } = this.editor.getSelectionRange();
                        const result = await this.editor.languageProvider.toggleComment(start, end);
                        if (result) {
                            this.editor.applyTextEdit(result.newText, result.newSelectionStart, result.newSelectionEnd);
                        }
                    }
                    return;
            }
        }
        if (e.key === 'F12') {
            e.preventDefault();
            if (this.editor.languageProvider) {
                const location = await this.editor.languageProvider.getDefinitionLocation(this.editor.cursor);
                if (location) {
                    this.editor.setCursor(location.targetIndex);
                    this.editor.selectionStart = this.editor.selectionEnd = this.editor.cursor;
                }
            }
            return;
        }
        switch (e.key) {
            case 'Enter':
                e.preventDefault();
                await this.editor.handleEnterKey();
                return;
            case 'ArrowLeft':
            case 'ArrowRight':
                if (!e.ctrlKey && !e.metaKey) {
                    const direction = e.key === 'ArrowLeft' ? 'left' : 'right';
                    if (this.editor.applyCoreStateCommand({ kind: 'move_cursor', direction, extendSelection: e.shiftKey })) {
                        this.editor.domUI.hideCompletion();
                        e.preventDefault();
                        this.editor.preferredCursorX = -1;
                        return;
                    }
                }
                if (e.ctrlKey) {
                    e.preventDefault();
                    const direction = e.key === 'ArrowLeft' ? 'left' : 'right';
                    if (this.editor.languageProvider) {
                        const result = await this.editor.languageProvider.getNextWordBoundary(this.editor.cursor, direction);
                        if (result && typeof result.targetIndex === 'number') {
                            this.editor.setCursor(result.targetIndex);
                        }
                    }
                    else {
                        this.editor.handleArrowKeys(new KeyboardEvent('keydown', { key: e.key, shiftKey: e.shiftKey }));
                    }
                    if (e.shiftKey)
                        this.editor.selectionEnd = this.editor.cursor;
                    else
                        this.editor.selectionStart = this.editor.selectionEnd = this.editor.cursor;
                    this.editor.updateOccurrencesHighlight();
                    return;
                }
            // Fallthrough for non-ctrl movement
            case 'ArrowUp':
            case 'ArrowDown':
                if (this.editor.applyCoreStateCommand({
                    kind: 'move_cursor_vertical',
                    direction: e.key === 'ArrowUp' ? 'up' : 'down',
                    extendSelection: e.shiftKey,
                })) {
                    this.editor.domUI.hideCompletion();
                    e.preventDefault();
                    return;
                }
                this.editor.domUI.hideCompletion();
                e.preventDefault();
                this.editor.handleArrowKeys(e);
                break;
            case 'Home':
            case 'End':
                if (this.editor.applyCoreStateCommand({
                    kind: 'move_cursor_line_boundary',
                    boundary: e.key === 'Home' ? 'home' : 'end',
                    extendSelection: e.shiftKey,
                })) {
                    this.editor.domUI.hideCompletion();
                    e.preventDefault();
                    return;
                }
                this.editor.domUI.hideCompletion();
                e.preventDefault();
                this.editor.handleHomeEndKeys(e);
                break;
            case 'PageUp':
            case 'PageDown':
                if (this.editor.applyCoreStateCommand({
                    kind: 'move_cursor_page',
                    direction: e.key === 'PageUp' ? 'up' : 'down',
                    pageSize: this.editor.visibleLines || 1,
                    extendSelection: e.shiftKey,
                })) {
                    this.editor.domUI.hideCompletion();
                    e.preventDefault();
                    return;
                }
                this.editor.domUI.hideCompletion();
                e.preventDefault();
                this.editor.handlePageKeys(e);
                break;
            case 'Insert':
                return;
            case 'Backspace':
                e.preventDefault();
                if (!this.editor.applyCoreStateCommand({ kind: 'delete_backward' })) {
                    if (this.editor.hasSelection()) {
                        this.editor.deleteSelection();
                    }
                    else if (this.editor.cursor > 0) {
                        const prevCursor = this.editor.cursor - 1;
                        this.editor.replaceTextRange(prevCursor, this.editor.cursor, '', prevCursor, prevCursor);
                    }
                }
                this.editor.triggerCompletion();
                break;
            case 'Delete':
                e.preventDefault();
                if (!this.editor.applyCoreStateCommand({ kind: 'delete_forward' })) {
                    if (this.editor.hasSelection()) {
                        this.editor.deleteSelection();
                    }
                    else if (this.editor.cursor < this.editor.text.length) {
                        this.editor.replaceTextRange(this.editor.cursor, this.editor.cursor + 1, '', this.editor.cursor, this.editor.cursor);
                    }
                }
                this.editor.triggerCompletion();
                break;
            case 'Tab':
                e.preventDefault();
                if (this.editor.languageProvider) {
                    const { start, end } = this.editor.getSelectionRange();
                    const result = await this.editor.languageProvider.adjustIndentation(start, end, e.shiftKey);
                    if (result) {
                        this.editor.applyTextEdit(result.newText, result.newSelectionStart, result.newSelectionEnd);
                    }
                }
                else {
                    this.editor.insertText('\t');
                }
                return;
            default:
                this.editor.preferredCursorX = -1;
                break;
        }
    }
}
//# sourceMappingURL=editor-input-handler.js.map
