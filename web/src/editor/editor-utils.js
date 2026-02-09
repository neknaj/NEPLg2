"use strict";
/**
 * エディタの複数のコンポーネントで使用される汎用的なユーティリティ関数を提供します。
 * これらの関数は、ステートレスであり、与えられた引数に基づいて計算や変換を行います。
 */
class EditorUtils {
    /**
     * @param {object} geometry - { h_width, z_width, lineHeight, gutterWidth, padding } を含むジオメトリ情報
     */
    constructor(geometry) {
        this.geom = geometry;
        this.charWidthCache = new Map();

        // 行の累積幅（prefix sum）キャッシュ: Map<lineString, Float32Array>
        // 横に長い行で「部分描画」や「二分探索」に使う
        this.linePrefixCache = new Map();

        // キャッシュの無制限成長を避けるための上限
        this.maxLinePrefixCacheSize = 2000;
        this.linePrefixCacheKeys = [];
    }

    /**
     * 文字が半角か全角かを判定し、キャッシュされた幅を返します。
     * @param {string} char - 幅を測定する文字
     * @returns {number} 文字のピクセル幅
     */
    getCharWidth(char) {
        if (this.charWidthCache.has(char)) {
            return this.charWidthCache.get(char);
        }
        const code = char.charCodeAt(0);
        const isHalfWidth = (code >= 0x0020 && code <= 0x007e) ||
            (code >= 0xff61 && code <= 0xff9f);
        const width = isHalfWidth ? this.geom.h_width : this.geom.z_width;
        this.charWidthCache.set(char, width);
        return width;
    }

    /**
     * 行文字列に対する「累積幅配列（prefix sum）」を返します。
     * prefix[i] は line.substring(0, i) の幅、prefix.length === line.length + 1
     * @param {string} line - 行全体
     * @returns {Float32Array} 累積幅
     */
    getLinePrefixWidths(line) {
        const cached = this.linePrefixCache.get(line);
        if (cached) {
            return cached;
        }

        const prefix = new Float32Array(line.length + 1);
        for (let i = 0; i < line.length; i++) {
            prefix[i + 1] = prefix[i] + this.getCharWidth(line[i]);
        }

        this.linePrefixCache.set(line, prefix);
        this.linePrefixCacheKeys.push(line);

        // FIFOで上限を超えたら削除（LRUほど厳密でなくても十分）
        if (this.linePrefixCacheKeys.length > this.maxLinePrefixCacheSize) {
            const oldest = this.linePrefixCacheKeys.shift();
            this.linePrefixCache.delete(oldest);
        }

        return prefix;
    }

    /**
     * キャッシュをクリアします。
     */
    clearCache() {
        this.charWidthCache.clear();
        this.linePrefixCache.clear();
        this.linePrefixCacheKeys = [];
    }

    /**
     * 文字列全体のピクセル幅を計算します。
     * @param {string} text - 測定する文字列
     * @returns {number} 文字列の合計ピクセル幅
     */
    measureText(text) {
        let totalWidth = 0;
        for (const char of text) {
            totalWidth += this.getCharWidth(char);
        }
        return totalWidth;
    }

    /**
     * 行の列番号からX座標（行先頭からの相対値）を返します。
     * @param {string} line - 行文字列
     * @param {number} col - 列（0..line.length）
     * @returns {number} 相対X
     */
    getXFromCol(line, col) {
        const prefix = this.getLinePrefixWidths(line);
        const c = Math.max(0, Math.min(prefix.length - 1, col));
        return prefix[c];
    }

    /**
     * 行の[startCol, endCol) 区間の幅を返します。
     * @param {string} line - 行文字列
     * @param {number} startCol - 開始列
     * @param {number} endCol - 終了列（exclusive）
     * @returns {number} 区間幅
     */
    measureTextRange(line, startCol, endCol) {
        const prefix = this.getLinePrefixWidths(line);
        const s = Math.max(0, Math.min(prefix.length - 1, startCol));
        const e = Math.max(0, Math.min(prefix.length - 1, endCol));
        return prefix[e] - prefix[s];
    }

    /**
     * 相対Xから列番号を求めます（prefix sumに対して二分探索）。
     * 返り値は「prefix[col] <= x」を満たす最大のcol。
     * @param {string} line - 行文字列
     * @param {number} x - 行先頭からの相対X
     * @returns {number} 列（0..line.length）
     */
    getColFromX(line, x) {
        const prefix = this.getLinePrefixWidths(line);

        if (x <= 0) {
            return 0;
        }
        if (x >= prefix[prefix.length - 1]) {
            return prefix.length - 1;
        }

        let lo = 0;
        let hi = prefix.length - 1;
        while (lo < hi) {
            const mid = (lo + hi + 1) >>> 1;
            if (prefix[mid] <= x) {
                lo = mid;
            }
            else {
                hi = mid - 1;
            }
        }
        return lo;
    }

    /**
     * 相対Xに最も近い列番号を返します（前後2点を比較）。
     * @param {string} line - 行文字列
     * @param {number} x - 行先頭からの相対X
     * @returns {number} 近傍列
     */
    getNearestColFromX(line, x) {
        const prefix = this.getLinePrefixWidths(line);
        const col = this.getColFromX(line, x);
        const next = Math.min(prefix.length - 1, col + 1);

        const d1 = Math.abs(prefix[col] - x);
        const d2 = Math.abs(prefix[next] - x);

        return d2 < d1 ? next : col;
    }

    /**
     * テキスト全体の文字インデックスを行と列の座標に変換します。
     * @param {number} index - 変換する文字インデックス
     * @param {string[]} lines - エディタの全行の配列
     * @returns {{row: number, col: number}} 行と列のオブジェクト
     */
    getPosFromIndex(index, lines) {
        let count = 0;
        for (let i = 0; i < lines.length; i++) {
            const lineLength = lines[i].length + 1; // +1 for newline char
            if (count + lineLength > index) {
                return { row: i, col: index - count };
            }
            count += lineLength;
        }
        return { row: lines.length - 1, col: lines[lines.length - 1].length };
    }

    /**
     * 行と列の座標をテキスト全体の文字インデックスに変換します。
     * @param {number} row - 行番号
     * @param {number} col - 列番号
     * @param {string[]} lines - エディタの全行の配列
     * @returns {number} 文字インデックス
     */
    getIndexFromPos(row, col, lines) {
        let index = 0;
        for (let i = 0; i < row; i++) {
            index += lines[i].length + 1; // +1 for newline char
        }
        return index + col;
    }

    /**
     * 指定された文字インデックスのCanvas上のXY座標を取得します。
     * @param {number} index - 座標を取得する文字インデックス
     * @param {string[]} lines - エディタの全行の配列
     * @param {number[]} lineYPositions - 各行のY座標の配列
     * @returns {{x: number, y: number}} Canvas上のXY座標
     */
    getCursorCoords(index, lines, lineYPositions) {
        const { row, col } = this.getPosFromIndex(index, lines);
        const y = lineYPositions[row];
        if (y === -1) {
            return { x: -1, y: -1 }; // Folded line
        }

        // substring + measureText のO(n)を避け、prefix sumでO(1)
        const x = this.geom.padding + this.geom.gutterWidth + this.getXFromCol(lines[row], col);
        return { x, y };
    }

    /**
     * Canvas上のXY座標から最も近い文字インデックスを取得します。
     * @param {number} x - Canvas上のX座標
     * @param {number} y - Canvas上のY座標
     * @param {string[]} lines - エディタの全行の配列
     * @param {number[]} lineYPositions - 各行のY座標の配列
     * @param {number} scrollX - 現在の水平スクロール量
     * @param {number} scrollY - 現在の垂直スクロール量
     * @param {boolean} [isGutterClick=false] - ガター領域でのクリックか
     * @param {number[] | null} [lineStartIndices=null] - 行の開始インデックス配列（高速化用）
     * @returns {number} 最も近い文字インデックス
     */
    getCursorIndexFromCoords(x, y, lines, lineYPositions, scrollX, scrollY, isGutterClick = false, lineStartIndices = null) {
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
            row = lines.length - 1; // Default to last line if out of bounds
        }

        const indexFromRowCol = (r, c) => {
            if (lineStartIndices && lineStartIndices[r] !== undefined) {
                return lineStartIndices[r] + c;
            }
            return this.getIndexFromPos(r, c, lines);
        };

        if (isGutterClick) {
            return indexFromRowCol(row, 0);
        }

        const logicalX = (x - this.geom.gutterWidth - this.geom.padding) + scrollX;
        const line = lines[row];

        // O(n^2) を避ける：二分探索で列を推定
        const col = this.getNearestColFromX(line, logicalX);
        return indexFromRowCol(row, col);
    }
}
//# sourceMappingURL=editor-utils.js.map
