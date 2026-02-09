// nodesrc/parser.js
// 目的:
// - .n.md（拡張 Markdown）から doctest と本文を抽出する
// - .nepl 内のドキュメントコメント（//:）から doctest と本文を抽出する
//
// 方針:
// - doctest は「neplg2:test」行の直後の ```neplg2 コードブロックを 1 テストケースとして扱う
// - コードブロック内の行頭 "|" は「本文では隠したい前置き」だが、テスト実行には含める

const fs = require('node:fs');

function parseTags(s) {
    // neplg2:test[compile_fail,should_panic]
    const m = s.match(/neplg2:test(?:\[([^\]]+)\])?/);
    if (!m) return [];
    if (!m[1]) return [];
    return m[1].split(',').map(x => x.trim()).filter(Boolean);
}

function stripHiddenPrefix(line) {
    // "| xxx" または "|xxx" を "xxx" にする
    if (line.startsWith('|')) {
        const t = line.slice(1);
        return t.startsWith(' ') ? t.slice(1) : t;
    }
    return line;
}

function scanForDoctests(lines, opts) {
    // opts:
    // - lineTransform: (rawLine)=>string  // doctest 指令やコードフェンス用に 1 行を解釈可能な文字列へ
    // - isHiddenLine: (rawLine)=>boolean // 本文で隠すための情報（今回は tests には不要だが一応保持）

    const doctests = [];

    for (let i = 0; i < lines.length; i++) {
        const raw = lines[i];
        const line = opts.lineTransform(raw);

        const m = line.match(/^\s*neplg2:test(?:\[[^\]]+\])?\s*$/);
        if (!m) continue;

        const tags = parseTags(line);

        // 次の ```neplg2 を探す
        let j = i + 1;
        while (j < lines.length) {
            const l2 = opts.lineTransform(lines[j]);
            if (/^\s*```\s*neplg2\s*$/.test(l2)) break;
            j++;
        }
        if (j >= lines.length) continue;

        // フェンス内を収集
        j++;
        const codeLines = [];
        const hiddenMap = [];
        while (j < lines.length) {
            const r3 = lines[j];
            const l3 = opts.lineTransform(r3);
            if (/^\s*```\s*$/.test(l3)) break;

            const hidden = opts.isHiddenLine(r3);
            const plain = stripHiddenPrefix(l3);
            codeLines.push(plain);
            hiddenMap.push(hidden);
            j++;
        }

        doctests.push({
            tags,
            code: codeLines.join('\n') + '\n',
            hiddenMap,
        });

        i = j;
    }

    return doctests;
}

function parseNmdText(text) {
    const lines = text.replace(/\r\n/g, '\n').split('\n');
    const doctests = scanForDoctests(lines, {
        lineTransform: (raw) => raw,
        isHiddenLine: (raw) => raw.startsWith('|'),
    });
    return { doctests };
}

function parseNeplText(text) {
    const lines = text.replace(/\r\n/g, '\n').split('\n');

    // //: と //:| を doc 用の 1 行に直す
    const doctests = scanForDoctests(lines, {
        lineTransform: (raw) => {
            const m = raw.match(/^\s*\/\/:(\|)?\s?(.*)$/);
            if (!m) return '';
            // 本文と同様に "|" 行も残し、stripHiddenPrefix が処理する
            return (m[1] ? '|' : '') + m[2];
        },
        isHiddenLine: (raw) => /^\s*\/\/:\|/.test(raw),
    });

    // ドキュメント本文抽出（簡易）: 先頭から連続する //: ブロックをすべて連結
    const docLines = [];
    for (const raw of lines) {
        const m = raw.match(/^\s*\/\/:(\|)?\s?(.*)$/);
        if (m) {
            // 本文では | 行はデフォルト非表示想定だが、HTML 側で制御する
            docLines.push((m[1] ? '|' : '') + m[2]);
        }
    }

    return {
        doctests,
        docText: docLines.join('\n') + '\n',
    };
}

function parseFile(filePath) {
    const text = fs.readFileSync(filePath, 'utf-8');
    if (filePath.endsWith('.n.md')) return { kind: 'nmd', ...parseNmdText(text), rawText: text };
    if (filePath.endsWith('.nepl')) return { kind: 'nepl', ...parseNeplText(text), rawText: text };
    return { kind: 'unknown', doctests: [], rawText: text };
}

module.exports = {
    parseFile,
    parseNmdText,
    parseNeplText,
};
