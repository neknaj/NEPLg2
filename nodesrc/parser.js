// nodesrc/parser.js
// 目的:
// - .n.md（拡張 Markdown）から doctest と本文を抽出する
// - .nepl 内のドキュメントコメント（//:）から doctest と本文を抽出する
// - HTML 生成向けに、.n.md を最小 AST（見出しネスト / ルビ / Gloss / リンク / コードフェンス等）へ変換する
//
// 方針:
// - doctest は「neplg2:test」行の直後の ```neplg2 コードブロックを 1 テストケースとして扱う
// - コードブロック内の行頭 "|" は「本文では隠したい前置き」だが、テスト実行には含める
// - HTML 生成の品質を落とさないため、本文は AST 化して html_gen 側で描画する

const fs = require('node:fs');

// -------------------------
// doctest 抽出（従来互換）
// -------------------------

function parseTags(s) {
    // 例: neplg2:test[compile_fail,should_panic]
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

function parseMetaValue(raw) {
    const s = String(raw || '').trim();
    if (!s) return '';
    // 優先: JSON 文字列として解釈（"\\n" などを正しく展開）
    if (s.startsWith('"') || s.startsWith("'")) {
        try {
            if (s.startsWith("'")) {
                const body = s.slice(1, s.endsWith("'") ? -1 : undefined);
                return body
                    .replace(/\\'/g, "'")
                    .replace(/\\\\/g, "\\")
                    .replace(/\\n/g, "\n")
                    .replace(/\\r/g, "\r")
                    .replace(/\\t/g, "\t");
            }
            return JSON.parse(s);
        } catch {
            return s;
        }
    }
    return s;
}

function scanForDoctests(lines, opts) {
    // opts:
    // - lineTransform: (rawLine)=>string
    // - isHiddenLine: (rawLine)=>boolean

    const doctests = [];

    for (let i = 0; i < lines.length; i++) {
        const raw = lines[i];
        const line = opts.lineTransform(raw);

        const m = line.match(/^\s*neplg2:test(?:\[[^\]]+\])?\s*$/);
        if (!m) continue;

        const tags = parseTags(line);

        const meta = {
            stdin: null,
            stdout: null,
            stderr: null,
        };

        // 次の ```neplg2 を探す
        let j = i + 1;
        while (j < lines.length) {
            const l2 = opts.lineTransform(lines[j]);
            const mm = l2.match(/^\s*(stdin|stdout|stderr)\s*:\s*(.*?)\s*$/);
            if (mm) {
                const k = mm[1];
                meta[k] = parseMetaValue(mm[2]);
            }
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
            stdin: meta.stdin,
            stdout: meta.stdout,
            stderr: meta.stderr,
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

    const doctests = scanForDoctests(lines, {
        lineTransform: (raw) => {
            const m = raw.match(/^\s*\/\/:(\|)?\s?(.*)$/);
            if (!m) return '';
            return (m[1] ? '|' : '') + m[2];
        },
        isHiddenLine: (raw) => /^\s*\/\/:\|/.test(raw),
    });

    // ドキュメント本文抽出: 先頭から連続する //: ブロックをすべて連結（暫定）
    const docLines = [];
    for (const raw of lines) {
        const m = raw.match(/^\s*\/\/:(\|)?\s?(.*)$/);
        if (m) {
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

// -----------------------------------------
// .n.md -> AST（HTML 生成の品質を回復）
// -----------------------------------------

function isHeadingLine(line) {
    const m = /^(#{1,6})\s+(.*)$/.exec(line);
    if (!m) return null;
    return { level: m[1].length, text: m[2] };
}

function isFenceStart(line) {
    const m = /^```([^\s`]*)\s*$/.exec(line);
    if (!m) return null;
    return { lang: m[1] || '' };
}

function isFenceEnd(line) {
    return /^```\s*$/.test(line);
}

function isBreakClose1(line) {
    return line === ';;;';
}

function isBreakHrClose1(line) {
    return line === '---';
}

function isHrOnly(line) {
    // ---- 以上は hr だが Nest に干渉しない
    return /^-{4,}\s*$/.test(line);
}

function normalizeNewlinesLiteral(s) {
    // 文字列としての "\n" を実改行にする
    return s.replace(/\\n/g, "\n");
}

// インライン解析（最小実装）
// - Math: $...$ / $$...$$
// - InlineCode: `...`
// - Link: [text](url)
// - Ruby: [漢字/よみ]（ただし [text](url) は Link）
// - Gloss: {A/b/β}
function parseInlines(text) {
    const s = normalizeNewlinesLiteral(text);
    const out = [];
    let i = 0;

    const pushText = (t) => {
        if (!t) return;
        const prev = out[out.length - 1];
        if (prev && prev.type === 'text') prev.text += t;
        else out.push({ type: 'text', text: t });
    };

    const readUntil = (needle, start) => {
        const idx = s.indexOf(needle, start);
        if (idx === -1) return null;
        return { end: idx, content: s.slice(start, idx) };
    };

    const splitGlossParts = (inner) => {
        // [] の中では / を区切りとして扱わない
        const parts = [];
        let buf = '';
        let bracket = 0;
        for (let k = 0; k < inner.length; k++) {
            const ch = inner[k];
            if (ch === '[') bracket++;
            if (ch === ']') bracket = Math.max(0, bracket - 1);
            if (ch === '/' && bracket === 0) {
                parts.push(buf);
                buf = '';
                continue;
            }
            buf += ch;
        }
        parts.push(buf);
        return parts;
    };

    while (i < s.length) {
        // $$...$$
        if (s.startsWith('$$', i)) {
            const r = readUntil('$$', i + 2);
            if (r) {
                out.push({ type: 'math', display: true, text: r.content });
                i = r.end + 2;
                continue;
            }
        }
        // $...$
        if (s[i] === '$') {
            const r = readUntil('$', i + 1);
            if (r) {
                out.push({ type: 'math', display: false, text: r.content });
                i = r.end + 1;
                continue;
            }
        }
        // `...`
        if (s[i] === '`') {
            const r = readUntil('`', i + 1);
            if (r) {
                out.push({ type: 'code_inline', text: r.content });
                i = r.end + 1;
                continue;
            }
        }
        // Gloss { ... }
        if (s[i] === '{') {
            const r = readUntil('}', i + 1);
            if (r) {
                const parts = splitGlossParts(r.content).map(p => p.trim());
                if (parts.length >= 2) {
                    out.push({
                        type: 'gloss',
                        base: parseInlines(parts[0]),
                        notes: parts.slice(1).map(p => parseInlines(p)),
                    });
                    i = r.end + 1;
                    continue;
                }
            }
        }
        // Link or Ruby
        if (s[i] === '[') {
            const r = readUntil(']', i + 1);
            if (r) {
                const after = s[r.end + 1] || '';
                if (after === '(') {
                    const r2 = readUntil(')', r.end + 2);
                    if (r2) {
                        const textInner = r.content;
                        const href = r2.content;
                        out.push({
                            type: 'link',
                            text: parseInlines(textInner),
                            href: href,
                        });
                        i = r2.end + 1;
                        continue;
                    }
                }
                const inner = r.content;
                const slash = inner.indexOf('/');
                if (slash !== -1) {
                    const base = inner.slice(0, slash);
                    const ruby = inner.slice(slash + 1);
                    out.push({
                        type: 'ruby',
                        base: parseInlines(base),
                        ruby: parseInlines(ruby),
                    });
                    i = r.end + 1;
                    continue;
                }
            }
        }

        // 既定: 1 文字進める
        pushText(s[i]);
        i += 1;
    }

    return out;
}

function newDocument() {
    return { type: 'document', children: [] };
}

function newSection(level, headingInlines) {
    return { type: 'section', level, heading: headingInlines, children: [] };
}

function newParagraph(lines) {
    // 段落内の改行は保持（HTML 側で <br/> 化）
    return { type: 'paragraph', inlines: parseInlines(lines.join('\n')) };
}

function newHr() {
    return { type: 'hr' };
}

function newCodeBlock(lang, codeText) {
    return { type: 'code', lang, text: codeText };
}

function parseNmdAst(source) {
    const lines = source.replace(/\r\n/g, '\n').replace(/\r/g, '\n').split('\n');

    const doc = newDocument();
    const stack = [{ level: 0, node: doc }];
    const curContainer = () => stack[stack.length - 1].node;

    const closeToLevel = (level) => {
        while (stack.length > 1 && stack[stack.length - 1].level >= level) {
            stack.pop();
        }
    };

    const closeOne = () => {
        if (stack.length > 1) stack.pop();
    };

    let i = 0;
    while (i < lines.length) {
        const line = lines[i];

        // Fence
        const fs0 = isFenceStart(line);
        if (fs0) {
            let j = i + 1;
            const codeLines = [];
            while (j < lines.length && !isFenceEnd(lines[j])) {
                codeLines.push(lines[j]);
                j += 1;
            }
            if (j < lines.length && isFenceEnd(lines[j])) j += 1;
            curContainer().children.push(newCodeBlock(fs0.lang, codeLines.join('\n')));
            i = j;
            continue;
        }

        // Heading
        const h = isHeadingLine(line);
        if (h) {
            closeToLevel(h.level);
            const sec = newSection(h.level, parseInlines(h.text));
            curContainer().children.push(sec);
            stack.push({ level: h.level, node: sec });
            i += 1;
            continue;
        }

        // Breaks / Hr
        if (isBreakHrClose1(line)) {
            curContainer().children.push(newHr());
            closeOne();
            i += 1;
            continue;
        }
        if (isBreakClose1(line)) {
            closeOne();
            i += 1;
            continue;
        }
        if (isHrOnly(line)) {
            curContainer().children.push(newHr());
            i += 1;
            continue;
        }

        // Blank
        if (line.trim() === '') {
            i += 1;
            continue;
        }

        // List item
        const lm = /^-\s+(.*)$/.exec(line);
        if (lm) {
            const items = [];
            let j = i;
            while (j < lines.length) {
                const m2 = /^-\s+(.*)$/.exec(lines[j]);
                if (!m2) break;
                items.push(parseInlines(m2[1]));
                j += 1;
            }
            curContainer().children.push({ type: 'list', items });
            i = j;
            continue;
        }

        // Paragraph
        const para = [];
        let j = i;
        while (j < lines.length) {
            const ln = lines[j];
            if (ln.trim() === '') break;
            if (isFenceStart(ln) || isHeadingLine(ln) || isBreakHrClose1(ln) || isBreakClose1(ln) || isHrOnly(ln) || /^-\s+/.test(ln)) break;
            para.push(ln);
            j += 1;
        }
        curContainer().children.push(newParagraph(para));
        i = j;
    }

    return doc;
}

module.exports = {
    // doctest / doc 抽出
    parseFile,
    parseNmdText,
    parseNeplText,

    // HTML 生成向け
    parseNmdAst,
    parseInlines,
};
