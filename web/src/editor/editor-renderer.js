"use strict";
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
        this.ctx = editor.ctx;
        this.lastBlinkTime = 0;
        this.lastFrameTime = 0;
        this.targetFPS = 60;
    }

    /**
     * 各行のY座標を再計算します。折り畳みされた行は -1 としてマークします。
     */
    recalculateLinePositions() {
        let currentY = this.editor.geom.padding;
        const lineCount = this.editor.lines.length;

        this.editor.lineYPositions = new Array(lineCount).fill(-1);

        for (let i = 0; i < lineCount; i++) {
            // 折り畳み範囲の開始行は必ず表示する
            this.editor.lineYPositions[i] = currentY;
            currentY += this.editor.geom.lineHeight;

            const foldRange = this.editor.foldingRangeByStartLine
                ? this.editor.foldingRangeByStartLine.get(i)
                : this.editor.foldingRanges.find(r => r.startLine === i);

            if (this.editor.foldedLines.has(i) && foldRange) {
                for (let j = foldRange.startLine + 1; j <= foldRange.endLine; j++) {
                    this.editor.lineYPositions[j] = -1; // Folded
                }
                i = foldRange.endLine; // 次の行へジャンプ
            }
        }

        const dpr = window.devicePixelRatio || 1;
        const cssHeight = this.editor.canvas.height / dpr;
        this.editor.visibleLines = Math.ceil(cssHeight / this.editor.geom.lineHeight);
    }

    /**
     * レンダリングループを開始します。
     */
    start() {
        this.lastFrameTime = performance.now();
        requestAnimationFrame(this.renderLoop.bind(this));
    }

    renderLoop(timestamp) {
        const deltaTime = timestamp - this.lastFrameTime;
        if (deltaTime > (1000 / this.targetFPS)) {
            this.lastFrameTime = timestamp - (deltaTime % (1000 / this.targetFPS));
            this.render();
            this.updateCursorBlink(timestamp);
        }
        requestAnimationFrame(this.renderLoop.bind(this));
    }

    updateCursorBlink(timestamp) {
        if (timestamp - this.lastBlinkTime > 500) {
            this.editor.cursorBlinkState = !this.editor.cursorBlinkState;
            this.lastBlinkTime = timestamp;
        }
    }

    /**
     * エディタの状態をCanvasに描画します。
     * 横に長い行を軽快にするため、各行の「可視範囲の列」だけを描画します。
     */
    render() {
        this.recalculateLinePositions();

        const dpr = window.devicePixelRatio || 1;
        const rect = this.editor.canvas.parentElement.getBoundingClientRect();
        const { gutterWidth, padding, lineHeight } = this.editor.geom;

        // 背景
        this.ctx.clearRect(0, 0, rect.width, rect.height);
        this.ctx.fillStyle = this.editor.colors.background;
        this.ctx.fillRect(0, 0, rect.width, rect.height);

        // ガター背景（水平スクロールの影響を受けない）
        this.ctx.fillStyle = this.editor.colors.gutterBg;
        this.ctx.fillRect(0, 0, gutterWidth, rect.height);

        // 可視範囲（X）は「行先頭からの相対X」に変換して扱う
        // 画面上のテキスト領域は [gutterWidth, rect.width) をクリップし、
        // 描画座標は translate(-scrollX, -scrollY) の「ワールド座標」にする。
        const viewLeftRel = this.editor.scrollX - padding;
        const viewRightRel = this.editor.scrollX + rect.width - gutterWidth - padding;

        // 選択範囲
        const selection = this.editor.hasSelection()
            ? this.editor.getSelectionRange()
            : null;

        // カーソル（行/列）
        const cursorPos = this.editor.indexToRowCol
            ? this.editor.indexToRowCol(this.editor.cursor)
            : this.editor.utils.getPosFromIndex(this.editor.cursor, this.editor.lines);

        // ========== ガター描画（水平スクロールなし、垂直のみ） ==========
        this.ctx.save();
        this.ctx.translate(0, -this.editor.scrollY);
        this.ctx.textAlign = 'right';
        this.ctx.textBaseline = 'middle';
        this.ctx.font = `${this.editor.fontSize}px ${this.editor.fontFamily}`;

        for (let i = 0; i < this.editor.lines.length; i++) {
            const y = this.editor.lineYPositions[i];
            if (y === -1 || y + lineHeight < this.editor.scrollY || y > this.editor.scrollY + rect.height) {
                continue;
            }

            const textY = y + lineHeight / 2;

            const isFoldable = this.editor.foldingRangeByStartLine
                ? this.editor.foldingRangeByStartLine.has(i)
                : this.editor.foldingRanges.some(r => r.startLine === i);

            if (isFoldable) {
                this.ctx.fillStyle = this.editor.foldedLines.has(i) ? this.editor.colors.foldMarker : this.editor.colors.foldMarkerHover;
                this.ctx.fillText(this.editor.foldedLines.has(i) ? '+' : '-', gutterWidth - 45, textY);
            }

            this.ctx.fillStyle = this.editor.colors.lineNumber;
            this.ctx.fillText((i + 1).toString(), gutterWidth - 10, textY);
        }
        this.ctx.restore();

        // ========== テキスト描画（水平/垂直スクロール） ==========
        this.ctx.save();

        // テキスト領域だけ描く（ガターは除外）
        this.ctx.beginPath();
        this.ctx.rect(gutterWidth, 0, rect.width - gutterWidth, rect.height);
        this.ctx.clip();

        // ワールド座標へ（ここから先はスクロールが効く）
        this.ctx.translate(-this.editor.scrollX, -this.editor.scrollY);

        const lineStartX = padding + gutterWidth;

        // 行ループ（可視行のみ）
        for (let i = 0; i < this.editor.lines.length; i++) {
            const y = this.editor.lineYPositions[i];
            if (y === -1 || y + lineHeight < this.editor.scrollY || y > this.editor.scrollY + rect.height) {
                continue;
            }

            const textY = y + lineHeight / 2;
            const line = this.editor.lines[i];
            const lineStartIndex = this.editor.lineStartIndices[i] !== undefined
                ? this.editor.lineStartIndices[i]
                : this.editor.utils.getIndexFromPos(i, 0, this.editor.lines);

            // 折り畳み表示の疑似テキスト
            const foldRange = this.editor.foldingRangeByStartLine
                ? this.editor.foldingRangeByStartLine.get(i)
                : this.editor.foldingRanges.find(r => r.startLine === i);

            const isFolded = this.editor.foldedLines.has(i) && foldRange;
            const placeholder = isFolded ? ` ... (${foldRange.endLine - foldRange.startLine} lines)` : '';

            // IME合成中は「表示用の行文字列」を差し込む（実テキストとは別）
            let lineToDraw = isFolded ? line + placeholder : line;
            let lineStartIndexOffset = 0;

            if (this.editor.isComposing && cursorPos.row === i) {
                const ccol = cursorPos.col;
                const before = line.substring(0, ccol);
                const after = line.substring(ccol);
                lineStartIndexOffset = this.editor.compositionText.length;
                lineToDraw = before + this.editor.compositionText + after;
            }

            // カレント行ハイライト
            if (cursorPos.row === i) {
                const yLine = y + lineHeight - 0.5;
                this.ctx.strokeStyle = this.editor.colors.currentLine;
                this.ctx.lineWidth = 1 / dpr;
                this.ctx.beginPath();
                this.ctx.moveTo(this.editor.scrollX + gutterWidth, yLine);
                this.ctx.lineTo(this.editor.scrollX + rect.width, yLine);
                this.ctx.stroke();
            }

            // ===== 横方向の「部分描画」: 可視範囲に対応する startCol / endCol を決める =====
            // viewLeftRel / viewRightRel は行先頭（lineStartX）からの相対X
            const startCol = Math.max(0, this.editor.utils.getColFromX(lineToDraw, viewLeftRel));
            const endCol = Math.min(lineToDraw.length, this.editor.utils.getColFromX(lineToDraw, viewRightRel) + 2);

            // インデント/末尾空白判定の事前計算
            const indentEndCol = line.match(/^\s*/)[0].length;
            const lastNonSpaceIndex = (line.match(/\s*$/)?.index ?? line.length) - 1;

            // 先頭空白の色分け（startColが途中からの場合の初期化）
            let isLeading = startCol < indentEndCol;
            let spaceCountInIndent = 0;
            if (this.editor.langConfig.highlightIndent) {
                const upto = Math.min(startCol, indentEndCol);
                for (let k = 0; k < upto; k++) {
                    if (line[k] === ' ') {
                        spaceCountInIndent++;
                    }
                }
            }

            // トークン（行ごとのセグメント）を取得
            const tokenSegments = (this.editor.tokensByLine && this.editor.tokensByLine[i]) ? this.editor.tokensByLine[i] : [];
            let tokenSegIdx = 0;
            while (tokenSegIdx < tokenSegments.length && tokenSegments[tokenSegIdx].endCol <= startCol) {
                tokenSegIdx++;
            }

            // 文字描画ループ（可視範囲のみ）
            let currentX = lineStartX + this.editor.utils.getXFromCol(lineToDraw, startCol);

            for (let j = startCol; j < endCol; j++) {
                // 折り畳みプレースホルダはまとめて描画して終了
                if (isFolded && j >= line.length) {
                    const placeholderWidth = this.editor.utils.measureText(placeholder);
                    this.ctx.fillStyle = this.editor.colors.tokenColors.comment;
                    this.ctx.fillRect(currentX, y, placeholderWidth, lineHeight);
                    this.ctx.fillStyle = this.editor.colors.background;
                    this.ctx.fillText(placeholder, currentX, textY);
                    break;
                }

                const char = lineToDraw[j];
                const charWidth = this.editor.utils.getCharWidth(char);

                // 文字インデックス（selectionやoccurrenceに使う）
                const charIndex = lineStartIndex + lineStartIndexOffset + j;

                // トークンの進行（jを前進させながらセグメントも前進）
                while (tokenSegIdx < tokenSegments.length && j >= tokenSegments[tokenSegIdx].endCol) {
                    tokenSegIdx++;
                }
                const tokenType = (tokenSegIdx < tokenSegments.length &&
                    j >= tokenSegments[tokenSegIdx].startCol &&
                    j < tokenSegments[tokenSegIdx].endCol)
                    ? tokenSegments[tokenSegIdx].type
                    : null;

                const inSelection = selection && charIndex >= selection.start && charIndex < selection.end;
                const isCurrentChar = charIndex === this.editor.cursor && this.editor.cursorBlinkState && this.editor.isFocused;
                const isOccurrence = this.editor.highlightedOccurrences.some(r => charIndex >= r.startIndex && charIndex < r.endIndex);
                const isBracket = this.editor.bracketHighlights.some(r => charIndex >= r.startIndex && charIndex < r.endIndex);
                const isTrailing = j >= lastNonSpaceIndex;

                // 背景（優先度順）
                if (inSelection) {
                    this.ctx.fillStyle = this.editor.colors.selection;
                    this.ctx.fillRect(currentX, y, charWidth, lineHeight);
                }
                else if (isCurrentChar) {
                    this.ctx.fillStyle = this.editor.colors.cursor;
                    this.ctx.fillRect(currentX, y, charWidth, lineHeight);
                }
                else if (isOccurrence) {
                    this.ctx.fillStyle = this.editor.colors.occurrenceHighlight;
                    this.ctx.fillRect(currentX, y, charWidth, lineHeight);
                }
                else if (isBracket) {
                    this.ctx.fillStyle = this.editor.colors.bracketHighlight;
                    this.ctx.fillRect(currentX, y, charWidth, lineHeight);
                }
                else if (this.editor.langConfig.highlightTrailingSpaces && isTrailing && (char === ' ' || char === '\t')) {
                    this.ctx.fillStyle = this.editor.colors.trailingSpaceHighlight;
                    this.ctx.fillRect(currentX, y, charWidth, lineHeight);
                }
                else if (this.editor.langConfig.highlightIndent && isLeading && (char === ' ' || char === '\t' || char === '　')) {
                    if (char === ' ') {
                        this.ctx.fillStyle = Math.floor((spaceCountInIndent - 1) / 4) % 2 === 0
                            ? this.editor.colors.indentHighlight1
                            : this.editor.colors.indentHighlight2;
                        this.ctx.fillRect(currentX, y, charWidth, lineHeight);
                        spaceCountInIndent++;
                    }
                    else if (char === '\t') {
                        this.ctx.fillStyle = this.editor.colors.tabHighlight;
                        this.ctx.fillRect(currentX, y, charWidth, lineHeight);
                    }
                    else {
                        this.ctx.fillStyle = this.editor.colors.fullWidthSpaceHighlight;
                        this.ctx.fillRect(currentX, y, charWidth, lineHeight);
                    }
                }

                // 字色
                if (tokenType) {
                    this.ctx.fillStyle = this.editor.colors.tokenColors[tokenType] || this.editor.colors.tokenColors.default;
                }
                else {
                    this.ctx.fillStyle = this.editor.colors.text;
                }

                // 可視の空白記号
                if (this.editor.langConfig.highlightWhitespace && (char === ' ' || char === '\t')) {
                    this.ctx.fillStyle = this.editor.colors.whitespaceHighlight;
                    this.ctx.fillText(char === ' ' ? '·' : '→', currentX, textY);
                }
                else {
                    this.ctx.fillText(char, currentX, textY);
                }

                // 先頭空白を抜けたらリセット
                if (isLeading && char !== ' ' && char !== '\t' && char !== '　') {
                    isLeading = false;
                }

                currentX += charWidth;
            }

            // 改行記号（可視範囲内なら描画）
            if (this.editor.langConfig.showNewlineSymbols && !isFolded) {
                const newlineX = lineStartX + this.editor.utils.getXFromCol(line, line.length);
                if (newlineX >= this.editor.scrollX + gutterWidth && newlineX <= this.editor.scrollX + rect.width) {
                    const newlineIndex = lineStartIndex + line.length;
                    const inSelection = selection && newlineIndex >= selection.start && newlineIndex < selection.end;

                    if (inSelection) {
                        this.ctx.fillStyle = this.editor.colors.selection;
                        this.ctx.fillRect(newlineX, y, this.editor.utils.getCharWidth(' '), lineHeight);
                    }
                    this.ctx.fillStyle = this.editor.colors.whitespaceHighlight;
                    this.ctx.fillText('↵', newlineX, textY);
                }
            }

            // 診断（行ごとのセグメント）: 下線だけを描く
            const diagSegments = (this.editor.diagnosticsByLine && this.editor.diagnosticsByLine[i]) ? this.editor.diagnosticsByLine[i] : [];
            if (diagSegments.length > 0) {
                for (const diag of diagSegments) {
                    const s = Math.max(diag.startCol, startCol);
                    const e = Math.min(diag.endCol, endCol);
                    if (s >= e) {
                        continue;
                    }

                    const diagX = lineStartX + this.editor.utils.getXFromCol(line, s);
                    const diagW = this.editor.utils.measureTextRange(line, s, e);

                    this.ctx.strokeStyle = diag.severity === 'error'
                        ? this.editor.colors.diagnosticError
                        : this.editor.colors.diagnosticWarning;

                    this.ctx.lineWidth = 2 / dpr;
                    this.ctx.beginPath();
                    this.ctx.moveTo(diagX, y + lineHeight - 2);
                    this.ctx.lineTo(diagX + diagW, y + lineHeight - 2);
                    this.ctx.stroke();
                }
            }
        }

        // ========== カーソル描画 ==========
        if (this.editor.cursorBlinkState && this.editor.isFocused) {
            const lineY = this.editor.lineYPositions[cursorPos.row];
            if (lineY !== -1) {
                const cursorX = (padding + gutterWidth) + this.editor.utils.getXFromCol(this.editor.lines[cursorPos.row], cursorPos.col);
                this.ctx.fillStyle = this.editor.colors.cursor;
                this.ctx.fillRect(cursorX, lineY, this.editor.isOverwriteMode ? this.editor.utils.getCharWidth(' ') : 1 / dpr, lineHeight);
            }
        }

        this.ctx.restore();

        // hidden textarea 等の位置更新
        this.editor.updateTextareaPosition();
    }
}
//# sourceMappingURL=editor-renderer.js.map
