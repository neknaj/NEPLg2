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
        const isHalfWidth = (char.charCodeAt(0) >= 0x0020 && char.charCodeAt(0) <= 0x007e) ||
                            (char.charCodeAt(0) >= 0xff61 && char.charCodeAt(0) <= 0xff9f);
        const width = isHalfWidth ? this.geom.h_width : this.geom.z_width;
        this.charWidthCache.set(char, width);
        return width;
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
        if (y === -1) return { x: -1, y: -1 }; // Folded line

        const textBefore = lines[row].substring(0, col);
        const x = this.geom.padding + this.geom.gutterWidth + this.measureText(textBefore);
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
     * @returns {number} 最も近い文字インデックス
     */
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
            row = lines.length - 1; // Default to last line if out of bounds
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