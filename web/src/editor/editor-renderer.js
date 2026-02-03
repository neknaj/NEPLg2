/**
 * Canvasへの描画処理をすべて担当します。
 * エディタの状態（テキスト、カーソル、選択範囲など）を受け取り、それをCanvas上にレンダリングします。
 */
class EditorRenderer {
    /**
     * @param {CanvasEditor} editor - 親となるCanvasEditorのインスタンス
     */
    constructor(editor) {
        this.editor = editor;
        this.canvas = editor.canvas;
        this.ctx = editor.ctx;
        this.colors = editor.colors;
        this.lastBlinkTime = 0;
    }
    
    /**
     * エディタの現在の状態に基づいて、行ごとのY座標を再計算します。
     * 折り畳まれた行はY座標を-1としてマークされます。
     */
    recalculateLinePositions() {
        this.editor.lineYPositions = [];
        let currentY = this.editor.geom.padding;
        for (let i = 0; i < this.editor.lines.length; i++) {
            this.editor.lineYPositions[i] = currentY;
            const range = this.editor.foldingRanges.find(r => r.startLine === i);
            if (range && this.editor.foldedLines.has(i)) {
                for (let j = i + 1; j <= range.endLine; j++) {
                    this.editor.lineYPositions[j] = -1; // Mark as hidden
                }
                i = range.endLine;
            }
            currentY += this.editor.geom.lineHeight;
        }
    }

    /**
     * エディタのメインレンダリングループ。
     * 毎フレーム呼び出され、エディタの全要素を描画します。
     * @param {number} timestamp - requestAnimationFrameから渡されるタイムスタンプ
     */
    renderLoop(timestamp) {
        this.updateCursorBlink(timestamp);
        this.render();
        this.editor.updateTextareaPosition();
        requestAnimationFrame(this.renderLoop.bind(this));
    }

    /**
     * カーソルの点滅状態を更新します。
     * @param {number} timestamp - 現在のタイムスタンプ
     */
    updateCursorBlink(timestamp) {
        if (!this.editor.isFocused || this.editor.isOverwriteMode) return;
        if (timestamp - this.lastBlinkTime > this.editor.blinkInterval) {
            this.editor.cursorBlinkState = !this.editor.cursorBlinkState;
            this.lastBlinkTime = timestamp;
        }
    }

    /**
     * Canvasにエディタの全要素を描画するメイン関数です。
     */
    render() {
        this.recalculateLinePositions();
        const dpr = window.devicePixelRatio || 1;
        const rect = this.canvas.parentElement.getBoundingClientRect();
        this.ctx.fillStyle = this.colors.background;
        this.ctx.fillRect(0, 0, rect.width, rect.height);
        
        this.ctx.fillStyle = this.colors.gutterBackground;
        this.ctx.fillRect(0, 0, this.editor.geom.gutterWidth, rect.height);
        
        this.ctx.save();
        this.ctx.translate(-this.editor.scrollX, -this.editor.scrollY);
        const selection = this.editor.getSelectionRange();
        const cursorPosition = this.editor.utils.getPosFromIndex(this.editor.cursor, this.editor.lines);
        
        for (let i = 0; i < this.editor.lines.length; i++) {
            const y = this.editor.lineYPositions[i];
            if (y === -1 || y + this.editor.geom.lineHeight < this.editor.scrollY || y > this.editor.scrollY + rect.height) {
                continue;
            }
    
            const line = this.editor.lines[i];
            const textY = y + this.editor.geom.lineHeight / 2;
    
            if (this.editor.isFocused && !this.editor.hasSelection() && cursorPosition.row === i) {
                this.ctx.strokeStyle = this.colors.cursorLineBorder;
                this.ctx.lineWidth = 1;
                this.ctx.beginPath();
                this.ctx.moveTo(this.editor.geom.gutterWidth, y);
                this.ctx.lineTo(this.editor.scrollX + this.canvas.width / dpr, y);
                this.ctx.stroke();
                this.ctx.beginPath();
                this.ctx.moveTo(this.editor.geom.gutterWidth, y + this.editor.geom.lineHeight);
                this.ctx.lineTo(this.editor.scrollX + this.canvas.width / dpr, y + this.editor.geom.lineHeight);
                this.ctx.stroke();
            }
    
            this.ctx.textAlign = 'right';
            const isFoldable = this.editor.foldingRanges.some(r => r.startLine === i);
            if (isFoldable) {
                this.ctx.fillStyle = this.colors.lineNumber;
                this.ctx.fillText(this.editor.foldedLines.has(i) ? '»' : '›', this.editor.geom.padding + 10, textY);
            }
            this.ctx.fillStyle = (this.editor.isFocused && cursorPosition.row === i) ? this.colors.lineNumberActive : this.colors.lineNumber;
            this.ctx.fillText(String(i + 1), this.editor.geom.gutterWidth - this.editor.geom.padding, textY);
            this.ctx.textAlign = 'left';
    
            let lineToDraw = line;
            const foldRange = this.editor.foldingRanges.find(r => r.startLine === i);
            const isFolded = foldRange && this.editor.foldedLines.has(i);
            let placeholder = '';
            if (isFolded) {
                placeholder = ` ${foldRange.placeholder || '...'}`;
                lineToDraw += placeholder;
            }
    
            const drawLineContent = (text, startX, textY, lineStartIndexOffset = 0) => {
                let currentX = startX; let isLeading = true; let spaceCountInIndent = 0;
                const lastNonSpaceIndex = (line.match(/\s*$/)?.index ?? line.length) -1;
    
                for (let j = 0; j < text.length; j++) {
                    const char = text[j];
                    const charWidth = (j >= line.length && isFolded) ? this.editor.utils.getCharWidth(' ') : this.editor.utils.getCharWidth(char);
                    const charIndex = this.editor.utils.getIndexFromPos(i, 0, this.editor.lines) + lineStartIndexOffset + j;
                    const isTrailing = (charIndex - this.editor.utils.getIndexFromPos(i, 0, this.editor.lines)) >= lastNonSpaceIndex;
    
                    if(j >= line.length && isFolded) { // Drawing placeholder
                        this.ctx.fillStyle = this.colors.comment;
                        this.ctx.fillRect(currentX, y, this.editor.utils.measureText(placeholder), this.editor.geom.lineHeight);
                        this.ctx.fillStyle = this.colors.text;
                        this.ctx.fillText(placeholder, currentX, textY);
                        break;
                    }
    
                    const isHighlightedOccurrence = this.editor.highlightedOccurrences.some(occ => charIndex >= occ.startIndex && charIndex < occ.endIndex);
                    const isBracketHighlight = this.editor.bracketHighlights.some(br => charIndex >= br.startIndex && charIndex < br.endIndex);
                    if (isHighlightedOccurrence || isBracketHighlight) { this.ctx.fillStyle = this.colors.occurrenceHighlight; this.ctx.fillRect(currentX, y, charWidth, this.editor.geom.lineHeight); }
                    
                    if (this.editor.langConfig.highlightIndent && isLeading) {
                        if (char === ' ') { spaceCountInIndent++; this.ctx.fillStyle = this.colors.indentation[Math.floor((spaceCountInIndent - 1) / 4) % 2]; this.ctx.fillRect(currentX, y, charWidth, this.editor.geom.lineHeight); }
                        else if (char === '\t') { this.ctx.fillStyle = this.colors.tab; this.ctx.fillRect(currentX, y, charWidth, this.editor.geom.lineHeight); }
                        else if (char === '　') { this.ctx.fillStyle = this.colors.fullWidthSpace; this.ctx.fillRect(currentX, y, charWidth, this.editor.geom.lineHeight); }
                        else { isLeading = false; }
                    } else { isLeading = false; }
                    if (isTrailing && (char === ' ' || char === '\t' || char === '　')) { this.ctx.fillStyle = this.colors.trailingSpace; this.ctx.fillRect(currentX, y, charWidth, this.editor.geom.lineHeight); }
                    
                    if (charIndex >= selection.start && charIndex < selection.end) { this.ctx.fillStyle = this.colors.selection; this.ctx.fillRect(currentX, y, charWidth, this.editor.geom.lineHeight); }
                    
                    const token = this.editor.tokens.find(t => charIndex >= t.startIndex && charIndex < t.endIndex);
                    this.ctx.fillStyle = token ? this.colors.tokenColors[token.type] || this.colors.tokenColors.default : this.colors.text;
                    if (char !== '　' || !this.editor.langConfig.highlightWhitespace) { this.ctx.fillText(char, currentX, textY); }
                    
                    if (this.editor.langConfig.highlightWhitespace) {
                        this.ctx.fillStyle = this.colors.whitespaceSymbol; this.ctx.textAlign = 'center';
                        if (char === ' ') { this.ctx.fillText('·', currentX + charWidth / 2, textY); }
                        else if (char === '\t') { this.ctx.fillText('»', currentX + charWidth / 2, textY); }
                        else if (char === '　') { this.ctx.fillText('◦', currentX + charWidth / 2, textY); }
                        this.ctx.textAlign = 'left';
                    }
                    currentX += charWidth;
                }
                return currentX;
            };
    
            const lineStartX = this.editor.geom.padding + this.editor.geom.gutterWidth;
            if (this.editor.isFocused && this.editor.isComposing && cursorPosition.row === i) {
                const lineBefore = line.substring(0, cursorPosition.col); const lineAfter = line.substring(cursorPosition.col);
                let currentX = drawLineContent(lineBefore, lineStartX, textY); const imeStartX = currentX;
                this.ctx.fillStyle = this.colors.text; let imeCurrentX = currentX;
                for (const char of this.editor.compositionText) { this.ctx.fillText(char, imeCurrentX, textY); imeCurrentX += this.editor.utils.getCharWidth(char); }
                const compositionWidth = this.editor.utils.measureText(this.editor.compositionText); this.ctx.strokeStyle = this.colors.imeUnderline;
                this.ctx.lineWidth = 1 / dpr; this.ctx.beginPath(); this.ctx.moveTo(imeStartX, y + this.editor.geom.lineHeight - 2);
                this.ctx.lineTo(imeStartX + compositionWidth, y + this.editor.geom.lineHeight - 2); this.ctx.stroke();
                currentX += compositionWidth; drawLineContent(lineAfter, currentX, textY, cursorPosition.col);
            } else {
                const finalX = drawLineContent(lineToDraw, lineStartX, textY);
                if (!isFolded) {
                    const newlineIndex = this.editor.utils.getIndexFromPos(i, 0, this.editor.lines) + line.length;
                    if (newlineIndex >= selection.start && newlineIndex < selection.end) { this.ctx.fillStyle = this.colors.selection; this.ctx.fillRect(finalX, y, this.editor.geom.h_width, this.editor.geom.lineHeight); }
                    if (this.editor.langConfig.highlightWhitespace) { this.ctx.fillStyle = this.colors.whitespaceSymbol; this.ctx.textAlign = 'center'; this.ctx.fillText('↲', finalX + this.editor.geom.h_width / 2, textY); this.ctx.textAlign = 'left'; }
                }
            }
    
            this.editor.diagnostics.forEach(diag => {
                const lineStartIndex = this.editor.utils.getIndexFromPos(i, 0, this.editor.lines); const lineEndIndex = lineStartIndex + line.length;
                if (diag.startIndex < lineEndIndex && diag.endIndex > lineStartIndex) {
                    const start = Math.max(diag.startIndex, lineStartIndex); const end = Math.min(diag.endIndex, lineEndIndex);
                    const textBefore = line.substring(0, start - lineStartIndex); const textDiag = line.substring(start - lineStartIndex, end - lineStartIndex);
                    const x = lineStartX + this.editor.utils.measureText(textBefore); const width = this.editor.utils.measureText(textDiag);
                    this.ctx.strokeStyle = diag.severity === 'error' ? this.colors.errorUnderline : this.colors.warningUnderline;
                    this.ctx.lineWidth = 1 / dpr; this.ctx.beginPath(); this.ctx.moveTo(x, y + this.editor.geom.lineHeight - 2); this.ctx.lineTo(x + width, y + this.editor.geom.lineHeight - 2); this.ctx.stroke();
                }
            });
        }
    
        if (this.editor.isFocused && !this.editor.isComposing) {
            const cursorPos = this.editor.utils.getCursorCoords(this.editor.cursor, this.editor.lines, this.editor.lineYPositions);
            if (cursorPos.y > -1) {
                if (this.editor.isOverwriteMode) {
                    const char = this.editor.text[this.editor.cursor] || ' '; const charWidth = this.editor.utils.getCharWidth(char);
                    this.ctx.fillStyle = this.colors.overwriteCursor; this.ctx.fillRect(cursorPos.x, cursorPos.y, charWidth, this.editor.geom.lineHeight);
                } else if (this.editor.cursorBlinkState && !this.editor.hasSelection()) {
                    this.ctx.fillStyle = this.colors.cursor; this.ctx.fillRect(cursorPos.x, cursorPos.y, 2 / dpr, this.editor.geom.lineHeight);
                }
            }
        }
        this.ctx.restore();
    }
}