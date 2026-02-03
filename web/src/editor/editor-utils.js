export class EditorUtils {
    /**
     * @param {object} geometry - { h_width, z_width, lineHeight, gutterWidth, padding }
     */
    constructor(geometry) {
        this.geom = geometry;
        this.charWidthCache = new Map();
    }

    getCharWidth(char) {
        if (this.charWidthCache.has(char)) {
            return this.charWidthCache.get(char);
        }
        const code = char.charCodeAt(0);
        // Basic check for half-width chars (ASCII + half-width kana)
        const isHalfWidth = (code >= 0x0020 && code <= 0x007e) || (code >= 0xff61 && code <= 0xff9f);
        const width = isHalfWidth ? this.geom.h_width : this.geom.z_width;
        this.charWidthCache.set(char, width);
        return width;
    }

    measureText(text) {
        let totalWidth = 0;
        for (const char of text) {
            totalWidth += this.getCharWidth(char);
        }
        return totalWidth;
    }

    getPosFromIndex(index, lines) {
        let count = 0;
        for (let i = 0; i < lines.length; i++) {
            const lineLength = lines[i].length + 1; // +1 for newline
            if (count + lineLength > index) {
                return { row: i, col: index - count };
            }
            count += lineLength;
        }
        // If at the very end
        return { row: lines.length - 1, col: lines[lines.length - 1].length };
    }

    getIndexFromPos(row, col, lines) {
        if (row < 0) return 0;
        if (row >= lines.length) return this.getIndexFromPos(lines.length - 1, lines[lines.length - 1].length, lines);
        
        let index = 0;
        for (let i = 0; i < row; i++) {
            index += lines[i].length + 1;
        }
        // Clamp col
        const safeCol = Math.min(col, lines[row].length);
        return index + safeCol;
    }

    getCursorCoords(index, lines, lineYPositions) {
        const { row, col } = this.getPosFromIndex(index, lines);
        const y = lineYPositions[row];
        if (y === -1) return { x: -1, y: -1 };

        const textBefore = lines[row].substring(0, col);
        const x = this.geom.padding + this.geom.gutterWidth + this.measureText(textBefore);
        return { x, y };
    }

    getCursorIndexFromCoords(x, y, lines, lineYPositions, scrollX, scrollY, isGutterClick = false) {
        const logicalY = y + scrollY;
        let row = -1;
        for (let i = 0; i < lineYPositions.length; i++) {
            const lineY = lineYPositions[i];
            if (lineY !== -1 && logicalY >= lineY && logicalY < lineY + this.geom.lineHeight) {
                row = i;
                break;
            }
        }
        if (row === -1) {
            // If below last line, pick last line
            if (logicalY > lineYPositions[lines.length - 1] + this.geom.lineHeight) {
                row = lines.length - 1;
            } else {
                // If above first line or in gap, clamp
                row = 0; // Simplified
            }
        }

        if (isGutterClick) {
            return this.getIndexFromPos(row, 0, lines);
        }

        const logicalX = (x - this.geom.gutterWidth - this.geom.padding) + scrollX;
        const line = lines[row];
        let minDelta = Infinity;
        let col = 0;

        for (let i = 0; i <= line.length; i++) {
            const w = this.measureText(line.substring(0, i));
            const delta = Math.abs(logicalX - w);
            if (delta < minDelta) {
                minDelta = delta;
                col = i;
            }
        }
        return this.getIndexFromPos(row, col, lines);
    }
}
