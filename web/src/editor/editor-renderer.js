export class EditorRenderer {
    constructor(editor) {
        this.editor = editor;
        this.canvas = editor.canvas;
        this.ctx = editor.ctx;
        this.colors = editor.colors;
        this.lastBlinkTime = 0;
    }

    recalculateLinePositions() {
        this.editor.lineYPositions = [];
        let currentY = this.editor.geom.padding;
        for (let i = 0; i < this.editor.lines.length; i++) {
            this.editor.lineYPositions[i] = currentY;
            // Fold logic skipped for MVP
            currentY += this.editor.geom.lineHeight;
        }
    }

    renderLoop(timestamp) {
        this.updateCursorBlink(timestamp);
        this.render();
        this.editor.updateTextareaPosition();
        requestAnimationFrame(this.renderLoop.bind(this));
    }

    updateCursorBlink(timestamp) {
        if (!this.editor.isFocused) return;
        if (timestamp - this.lastBlinkTime > this.editor.blinkInterval) {
            this.editor.cursorBlinkState = !this.editor.cursorBlinkState;
            this.lastBlinkTime = timestamp;
        }
    }

    render() {
        this.recalculateLinePositions();
        const dpr = window.devicePixelRatio || 1;
        const rect = this.canvas.parentElement.getBoundingClientRect(); // Use parent size

        // Ensure canvas size matches valid rect
        if (rect.width !== this.canvas.width / dpr || rect.height !== this.canvas.height / dpr) {
            // Let resizeEditor handle it usually, but failsafe here maybe?
        }

        // Fill Background
        this.ctx.fillStyle = this.colors.background;
        this.ctx.fillRect(0, 0, this.canvas.width / dpr, this.canvas.height / dpr);

        // Gutter
        this.ctx.fillStyle = this.colors.gutterBackground;
        this.ctx.fillRect(0, 0, this.editor.geom.gutterWidth, this.canvas.height / dpr);

        this.ctx.save();
        this.ctx.translate(-this.editor.scrollX, -this.editor.scrollY);

        const selection = this.editor.getSelectionRange();
        const cursorPosition = this.editor.utils.getPosFromIndex(this.editor.cursor, this.editor.lines);

        for (let i = 0; i < this.editor.lines.length; i++) {
            const y = this.editor.lineYPositions[i];
            // Viewport culling
            if (y + this.editor.geom.lineHeight < this.editor.scrollY || y > this.editor.scrollY + (this.canvas.height / dpr)) {
                continue;
            }

            const line = this.editor.lines[i];
            const textY = y + this.editor.geom.lineHeight / 2;

            // Line Highlight
            if (this.editor.isFocused && !this.editor.hasSelection() && cursorPosition.row === i) {
                this.ctx.strokeStyle = this.colors.cursorLineBorder;
                this.ctx.lineWidth = 1;
                this.ctx.globalAlpha = 0.5;
                this.ctx.beginPath();
                this.ctx.moveTo(this.editor.geom.gutterWidth + this.editor.scrollX, y); // +scrollX to keep fixed relative to view
                this.ctx.lineTo(this.editor.scrollX + (this.canvas.width / dpr), y);
                this.ctx.stroke();
                this.ctx.beginPath();
                this.ctx.moveTo(this.editor.geom.gutterWidth + this.editor.scrollX, y + this.editor.geom.lineHeight);
                this.ctx.lineTo(this.editor.scrollX + (this.canvas.width / dpr), y + this.editor.geom.lineHeight);
                this.ctx.stroke();
                this.ctx.globalAlpha = 1.0;
            }

            // Line Number
            this.ctx.textAlign = 'right';
            this.ctx.fillStyle = (this.editor.isFocused && cursorPosition.row === i) ? this.colors.lineNumberActive : this.colors.lineNumber;
            this.ctx.fillText(String(i + 1), this.editor.geom.gutterWidth - this.editor.geom.padding, textY);
            this.ctx.textAlign = 'left';

            // Text Drawing
            let currentX = this.editor.geom.padding + this.editor.geom.gutterWidth;
            const lineStartIndex = this.editor.utils.getIndexFromPos(i, 0, this.editor.lines);

            // IME Handling
            if (this.editor.isFocused && this.editor.isComposing && cursorPosition.row === i) {
                // Pre-IME part
                const pre = line.substring(0, cursorPosition.col);
                this.drawTextSegment(pre, currentX, y, textY, lineStartIndex, selection);
                currentX += this.editor.utils.measureText(pre);

                // IME part
                const imeX = currentX;
                this.ctx.fillStyle = this.colors.text;
                for (const char of this.editor.compositionText) {
                    this.ctx.fillText(char, currentX, textY);
                    currentX += this.editor.utils.getCharWidth(char);
                }
                const imeWidth = this.editor.utils.measureText(this.editor.compositionText);
                this.ctx.strokeStyle = this.colors.imeUnderline;
                this.ctx.beginPath();
                this.ctx.moveTo(imeX, y + this.editor.geom.lineHeight - 2);
                this.ctx.lineTo(imeX + imeWidth, y + this.editor.geom.lineHeight - 2);
                this.ctx.stroke();

                // Post-IME part
                const post = line.substring(cursorPosition.col);
                this.drawTextSegment(post, currentX, y, textY, lineStartIndex + pre.length, selection);
            } else {
                this.drawTextSegment(line, currentX, y, textY, lineStartIndex, selection);
            }
        }

        // Cursor
        if (this.editor.isFocused && !this.editor.isComposing && this.editor.cursorBlinkState) {
            const cursorPos = this.editor.utils.getCursorCoords(this.editor.cursor, this.editor.lines, this.editor.lineYPositions);
            if (cursorPos.y > -1) {
                this.ctx.fillStyle = this.colors.cursor;
                this.ctx.fillRect(cursorPos.x, cursorPos.y, 2, this.editor.geom.lineHeight);
            }
        }

        this.ctx.restore();
    }

    drawTextSegment(text, startX, rowY, textY, startIndexOffset, selection) {
        let currentX = startX;
        for (let j = 0; j < text.length; j++) {
            const char = text[j];
            const charWidth = this.editor.utils.getCharWidth(char);
            const charIndex = startIndexOffset + j;

            // Selection Background
            if (charIndex >= selection.start && charIndex < selection.end) {
                this.ctx.fillStyle = this.colors.selection;
                this.ctx.fillRect(currentX, rowY, charWidth, this.editor.geom.lineHeight);
            }

            // Syntax Highlighting (Basic mock for starts)
            const token = this.editor.tokens.find(t => charIndex >= t.startIndex && charIndex < t.endIndex);
            this.ctx.fillStyle = token ? (this.colors.tokenColors[token.type] || this.colors.text) : this.colors.text;

            this.ctx.fillText(char, currentX, textY);
            currentX += charWidth;
        }
    }
}
