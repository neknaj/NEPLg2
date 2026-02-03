export class EditorInputHandler {
    constructor(editor) {
        this.editor = editor;
        this.canvas = editor.canvas;
        this.textarea = editor.textarea;
        this.isDragging = false;
        this.hoverTimeout = null;
        this.lastHoverIndex = -1;
    }

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
            // Simplified focus loss logic
            if (!this.editor.canvas.contains(e.target) && !this.editor.textarea.contains(e.target) && !e.target.closest('.popup-menu')) {
                // If clicking outside, blur (optional, maybe keep focus for playground convenience?)
                // this.editor.blur(); 
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

        // Copy/Paste
        this.textarea.addEventListener('copy', this.onCopy.bind(this));
        this.textarea.addEventListener('paste', this.onPaste.bind(this));
        this.textarea.addEventListener('cut', this.onCut.bind(this));

        const observer = new ResizeObserver(() => this.editor.resizeEditor());
        observer.observe(this.canvas.parentElement);
    }

    onCopy(e) {
        // e.preventDefault(); // Let default copy happen if possible? No, we handle manually because textarea is empty usually
        if (!this.editor.hasSelection()) return;
        const { start, end } = this.editor.getSelectionRange();
        const selectedText = this.editor.text.substring(start, end);
        e.clipboardData.setData('text/plain', selectedText);
        e.preventDefault();
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

        const rect = this.canvas.getBoundingClientRect();
        const offsetX = e.clientX - rect.left;
        const offsetY = e.clientY - rect.top;

        if (offsetX < this.editor.geom.gutterWidth) {
            // Fold click logic (simplified for now)
            return;
        }

        this.isDragging = true;
        const pos = this.editor.utils.getCursorIndexFromCoords(offsetX, offsetY, this.editor.lines, this.editor.lineYPositions, this.editor.scrollX, this.editor.scrollY);
        if (!e.shiftKey) {
            this.editor.setCursor(pos);
            this.editor.selectionStart = this.editor.cursor;
            this.editor.selectionEnd = this.editor.cursor;
        } else {
            this.editor.setCursor(pos);
            this.editor.selectionEnd = this.editor.cursor; // Extend selection
        }
        this.editor.domUI.hideCompletion();
    }

    onMouseMove(e) {
        const rect = this.canvas.getBoundingClientRect();
        const offsetX = e.clientX - rect.left;
        const offsetY = e.clientY - rect.top;

        const pos = this.editor.utils.getCursorIndexFromCoords(offsetX, offsetY, this.editor.lines, this.editor.lineYPositions, this.editor.scrollX, this.editor.scrollY);
        if (this.isDragging) {
            this.editor.domUI.hidePopup();
            clearTimeout(this.hoverTimeout);
            this.lastHoverIndex = -1;
            this.editor.setCursor(pos);
            this.editor.selectionEnd = this.editor.cursor;
        } else {
            // Hover logic
            if (pos !== this.lastHoverIndex) {
                this.lastHoverIndex = pos;
                this.editor.domUI.hidePopup();
                clearTimeout(this.hoverTimeout);
                this.hoverTimeout = setTimeout(() => this.handleHover(e, pos), 500);
            }
        }
    }

    async handleHover(e, pos) {
        // Implement hover tooltips if needed
    }

    onMouseUp() {
        this.isDragging = false;
        this.editor.preferredCursorX = -1;
    }

    onWheel(e) {
        e.preventDefault();
        this.editor.domUI.hideCompletion();
        // Simple scroll sensitivity
        const scrollSpeed = 0.5;
        const newScrollY = this.editor.scrollY + (e.deltaY * scrollSpeed);
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
        // Key bindings...
        if (this.editor.domUI.isCompletionVisible) {
            switch (e.key) {
                case 'ArrowUp': e.preventDefault(); this.editor.domUI.updateCompletionSelection(-1); return;
                case 'ArrowDown': e.preventDefault(); this.editor.domUI.updateCompletionSelection(1); return;
                case 'Enter': case 'Tab': e.preventDefault(); this.editor.acceptCompletion(); return;
                case 'Escape': e.preventDefault(); this.editor.domUI.hideCompletion(); return;
            }
        }

        if (e.ctrlKey || e.metaKey) {
            switch (e.key.toLowerCase()) {
                case 'a': e.preventDefault(); this.editor.selectAll(); return;
                case 'z': e.preventDefault(); this.editor.undo(); return;
                case 'y': e.preventDefault(); this.editor.redo(); return;
            }
        }

        switch (e.key) {
            case 'Enter':
                e.preventDefault();
                await this.editor.handleEnterKey();
                break;
            case 'Backspace':
                e.preventDefault();
                this.editor.handleBackspace();
                break;
            case 'Delete':
                e.preventDefault();
                this.editor.handleDelete();
                break;
            case 'ArrowLeft':
            case 'ArrowRight':
            case 'ArrowUp':
            case 'ArrowDown':
                e.preventDefault();
                this.editor.handleArrowKeys(e);
                break;
            default:
                break;
        }
    }
}
