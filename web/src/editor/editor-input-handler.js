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
        if (!this.editor.hasSelection()) return;
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
        if (!this.editor.hasSelection()) return;
        this.onCopy(e);
        this.editor.deleteSelection();
    }

    onMouseDown(e) {
        e.preventDefault();
        this.editor.focus();
    
        if (e.offsetX < this.editor.geom.gutterWidth) {
            const clickedRow = this.editor.utils.getPosFromIndex(
                this.editor.utils.getCursorIndexFromCoords(e.offsetX, e.offsetY, this.editor.lines, this.editor.lineYPositions, this.editor.scrollX, this.editor.scrollY, true),
                this.editor.lines
            ).row;
            const range = this.editor.foldingRanges.find(r => r.startLine === clickedRow);
            if (range) {
                this.editor.toggleFold(clickedRow);
            }
            return;
        }
    
        this.isDragging = true;
        const pos = this.editor.utils.getCursorIndexFromCoords(e.offsetX, e.offsetY, this.editor.lines, this.editor.lineYPositions, this.editor.scrollX, this.editor.scrollY);
        this.editor.setCursor(pos);
        this.editor.selectionStart = this.editor.cursor;
        this.editor.selectionEnd = this.editor.cursor;
        this.editor.domUI.hideCompletion();
    }
    
    onMouseMove(e) {
        const pos = this.editor.utils.getCursorIndexFromCoords(e.offsetX, e.offsetY, this.editor.lines, this.editor.lineYPositions, this.editor.scrollX, this.editor.scrollY);
        if (this.isDragging) {
            this.editor.domUI.hidePopup();
            clearTimeout(this.hoverTimeout);
            this.lastHoverIndex = -1;
            this.editor.setCursor(pos);
            this.editor.selectionEnd = this.editor.cursor;
        } else {
            if (pos !== this.lastHoverIndex) {
                this.lastHoverIndex = pos;
                this.editor.domUI.hidePopup();
                clearTimeout(this.hoverTimeout);
                this.hoverTimeout = setTimeout(() => this.handleHover(e, pos), 100);
            }
        }
    }

    async handleHover(e, pos) {
        const diagnostic = this.editor.diagnostics.find(d => pos >= d.startIndex && pos < d.endIndex);
        if (diagnostic) {
            this.editor.domUI.showPopup(diagnostic.message, e.clientX, e.clientY);
            return;
        }

        if (!this.editor.languageProvider) return;
        this.lastHoverIndex = pos;
        const hoverInfo = await this.editor.languageProvider.getHoverInfo(pos);
        if (hoverInfo && hoverInfo.content) {
            this.editor.domUI.showPopup(hoverInfo.content, e.clientX, e.clientY);
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
        const newScrollY = this.editor.scrollY + e.deltaY;
        const maxScrollY = Math.max(0, this.editor.lines.length * this.editor.geom.lineHeight - this.canvas.height + this.editor.geom.padding * 2);
        this.editor.scrollY = Math.max(0, Math.min(newScrollY, maxScrollY));
    }
    
    onInput(e) {
        if (this.editor.isComposing) return;
        const newText = e.target.value;
        if (newText) {
            this.editor.insertText(newText);
            this.textarea.value = '';
            this.editor.triggerCompletion();
        }
    }
    
    async onKeydown(e) {
        if (this.editor.isComposing) return;

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
                case 'a':
                    e.preventDefault();
                    this.editor.selectionStart = 0;
                    this.editor.selectionEnd = this.editor.text.length;
                    this.editor.setCursor(this.editor.text.length);
                    return;
                case 'z':
                    e.preventDefault();
                    this.editor.undo();
                    return;
                case 'y':
                    e.preventDefault();
                    this.editor.redo();
                    return;
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
                if (e.ctrlKey) {
                    e.preventDefault();
                    const direction = e.key === 'ArrowLeft' ? 'left' : 'right';
                    if (this.editor.languageProvider) {
                        const result = await this.editor.languageProvider.getNextWordBoundary(this.editor.cursor, direction);
                        if (result && typeof result.targetIndex === 'number') {
                            this.editor.setCursor(result.targetIndex);
                        }
                    } else {
                        this.editor.handleArrowKeys(new KeyboardEvent('keydown', { key: e.key, shiftKey: e.shiftKey }));
                    }
                    if (e.shiftKey) this.editor.selectionEnd = this.editor.cursor;
                    else this.editor.selectionStart = this.editor.selectionEnd = this.editor.cursor;
                    this.editor.updateOccurrencesHighlight();
                    return;
                }
                // Fallthrough for non-ctrl movement
            case 'ArrowUp':
            case 'ArrowDown':
                this.editor.domUI.hideCompletion();
                e.preventDefault();
                this.editor.handleArrowKeys(e);
                break;
            case 'Home':
            case 'End':
                this.editor.domUI.hideCompletion();
                e.preventDefault();
                this.editor.handleHomeEndKeys(e);
                break;
            case 'PageUp':
            case 'PageDown':
                this.editor.domUI.hideCompletion();
                e.preventDefault();
                this.editor.handlePageKeys(e);
                break;
            case 'Insert':
                e.preventDefault();
                this.editor.isOverwriteMode = !this.editor.isOverwriteMode;
                this.editor.resetCursorBlink();
                break;
            case 'Backspace':
                e.preventDefault();
                if (this.editor.hasSelection()) {
                    this.editor.deleteSelection();
                } else if (this.editor.cursor > 0) {
                    this.editor.recordHistory();
                    const prevCursor = this.editor.cursor - 1;
                    this.editor.text = this.editor.text.slice(0, prevCursor) + this.editor.text.slice(this.editor.cursor);
                    this.editor.setCursor(prevCursor);
                    this.editor.selectionStart = this.editor.selectionEnd = this.editor.cursor;
                    this.editor.updateLines();
                    this.editor.updateText(this.editor.text);
                    this.editor.updateOccurrencesHighlight();
                }
                this.editor.triggerCompletion();
                break;
            case 'Delete':
                e.preventDefault();
                if (this.editor.hasSelection()) {
                    this.editor.deleteSelection();
                } else if (this.editor.cursor < this.editor.text.length) {
                    this.editor.recordHistory();
                    this.editor.text = this.editor.text.slice(0, this.editor.cursor) + this.editor.text.slice(this.editor.cursor + 1);
                    this.editor.updateLines();
                    this.editor.updateText(this.editor.text);
                    this.editor.updateOccurrencesHighlight();
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
                } else {
                    this.editor.insertText('\t');
                }
                return;
            default:
                this.editor.preferredCursorX = -1;
                break;
        }
    }
}