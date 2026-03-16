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

const fs = require("node:fs");

// -------------------------
// doctest 抽出（従来互換）
// -------------------------

function parseTags(s) {
  // 例: neplg2:test[compile_fail,should_panic]
  const m = s.match(/neplg2:test(?:\[([^\]]+)\])?/);
  if (!m) return [];
  if (!m[1]) return [];
  return m[1]
    .split(",")
    .map((x) => x.trim())
    .filter(Boolean);
}

function stripHiddenPrefix(line) {
  // "| xxx" または "|xxx" を "xxx" にする
  if (line.startsWith("|")) {
    const t = line.slice(1);
    return t.startsWith(" ") ? t.slice(1) : t;
  }
  return line;
}

function parseMetaValue(raw) {
  const s = String(raw || "").trim();
  if (!s) return "";
  // 優先: JSON 文字列として解釈（"\\n" などを正しく展開）
  if (
    s.startsWith('"') ||
    s.startsWith("'") ||
    s.startsWith("[") ||
    s.startsWith("{")
  ) {
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

function parseRetValue(raw) {
  const parsed = parseMetaValue(raw);
  if (typeof parsed === "number") return parsed;
  if (typeof parsed !== "string") return parsed;
  const s = parsed.trim();
  if (/^-?\d+$/.test(s)) return parseInt(s, 10);
  if (/^-?(?:\d+\.\d*|\d*\.\d+)$/.test(s)) return parseFloat(s);
  return parsed;
}

function parseDiagSpanEntry(raw) {
  if (raw && typeof raw === "object" && !Array.isArray(raw)) {
    const line = Number(raw.line);
    const col = Number(raw.col);
    if (!Number.isFinite(line) || !Number.isFinite(col)) return null;
    const fileRaw = raw.file;
    return {
      file:
        fileRaw === undefined || fileRaw === null
          ? null
          : String(fileRaw).trim(),
      line,
      col,
    };
  }
  const s = String(raw || "").trim();
  if (!s) return null;
  let m = s.match(/^(\d+)\s*:\s*(\d+)$/);
  if (m) {
    return {
      file: null,
      line: parseInt(m[1], 10),
      col: parseInt(m[2], 10),
    };
  }
  m = s.match(/^(.+):(\d+):(\d+)$/);
  if (m) {
    return {
      file: String(m[1]).trim(),
      line: parseInt(m[2], 10),
      col: parseInt(m[3], 10),
    };
  }
  return null;
}

function parseDiagSpanList(raw) {
  const parsed = parseMetaValue(raw);
  const vals = Array.isArray(parsed)
    ? parsed
    : String(parsed)
        .split(",")
        .map((x) => x.trim())
        .filter(Boolean);
  const out = [];
  for (const v of vals) {
    const one = parseDiagSpanEntry(v);
    if (one) out.push(one);
  }
  return out;
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
      argv: null,
      stdout: null,
      stderr: null,
      ret: null,
      diag_ids: [],
      diag_spans: [],
    };

    // 次の ```neplg2 を探す
    let j = i + 1;
    while (j < lines.length) {
      const l2 = opts.lineTransform(lines[j]);
      const mm = l2.match(
        /^\s*(stdin|argv|stdout|stderr|ret|diag_id|diag_ids|diag_span|diag_spans)\s*:\s*(.*?)\s*$/,
      );
      if (mm) {
        const k = mm[1];
        if (k === "diag_id") {
          const v = parseInt(String(parseMetaValue(mm[2] ?? "")).trim(), 10);
          if (Number.isFinite(v)) meta.diag_ids.push(v);
        } else if (k === "diag_ids") {
          const rawVals = parseMetaValue(mm[2] ?? "");
          const vals = Array.isArray(rawVals)
            ? rawVals
            : String(rawVals)
                .split(",")
                .map((x) => x.trim())
                .filter(Boolean);
          for (const x of vals) {
            const v = parseInt(String(x).trim(), 10);
            if (Number.isFinite(v)) meta.diag_ids.push(v);
          }
        } else if (k === "diag_span") {
          const one = parseDiagSpanEntry(parseMetaValue(mm[2] ?? ""));
          if (one) meta.diag_spans.push(one);
        } else if (k === "diag_spans") {
          meta.diag_spans.push(...parseDiagSpanList(mm[2] ?? ""));
        } else {
          meta[k] = k === "ret" ? parseRetValue(mm[2]) : parseMetaValue(mm[2]);
        }
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
      code: codeLines.join("\n") + "\n",
      hiddenMap,
      stdin: meta.stdin,
      argv: meta.argv,
      stdout: meta.stdout,
      stderr: meta.stderr,
      ret: meta.ret,
      diag_ids: meta.diag_ids,
      diag_spans: meta.diag_spans,
    });

    i = j;
  }

  return doctests;
}

function parseNmdText(text) {
  const lines = text.replace(/\r\n/g, "\n").split("\n");
  const doctests = scanForDoctests(lines, {
    lineTransform: (raw) => raw,
    isHiddenLine: (raw) => raw.startsWith("|"),
  });
  return { doctests };
}

function parseNeplText(text) {
  const lines = text.replace(/\r\n/g, "\n").split("\n");

  const doctests = scanForDoctests(lines, {
    lineTransform: (raw) => {
      const m = raw.match(/^\s*\/\/:(\|)?\s?(.*)$/);
      if (!m) return "";
      return (m[1] ? "|" : "") + m[2];
    },
    isHiddenLine: (raw) => /^\s*\/\/:\|/.test(raw),
  });

  // ドキュメント本文抽出: //: ブロックを拾いつつ、直後のコードから kind を判定して見出しに付与する
  const docLines = [];
  let inDocBlock = false;
  let currentBlock = [];
  let isFirstBlock = true;

  for (let i = 0; i < lines.length; i++) {
    const raw = lines[i];
    const m = raw.match(/^\s*\/\/:(\|)?\s?(.*)$/);

    if (m) {
      inDocBlock = true;
      currentBlock.push((m[1] ? "|" : "") + m[2]);
    } else {
      if (inDocBlock) {
        // ブロック終了。次の有効なコード行を探して kind を判定する
        let kind = null;
        let typeInfo = null;
        for (let j = i; j < lines.length; j++) {
          const codeLine = lines[j].trim();
          if (codeLine === "" || codeLine.startsWith("//")) continue;

          if (codeLine.startsWith("fn ")) {
            kind = "fn";
            const typeMatch = codeLine.match(/<.*>/);
            if (typeMatch) {
              typeInfo = typeMatch[0]; // "<...>"
            }
          } else if (codeLine.startsWith("struct ")) kind = "struct";
          else if (codeLine.startsWith("enum ")) kind = "enum";
          else if (codeLine.startsWith("trait ")) kind = "trait";
          else if (codeLine.startsWith("impl ")) kind = "impl";
          else if (codeLine.startsWith("let ")) kind = "let";
          break;
        }

        if (isFirstBlock && !kind) {
          // モジュールのトップレベルドキュメントとみなす
          let hasH1 = currentBlock.some((l) => /^#\s/.test(l));
          if (hasH1) {
            kind = "module";
          }
        }

        if (kind) {
          // ブロック内に見出しがない場合、最初の行を legacy 形式の見出しとして扱う
          let hasHeading = currentBlock.some((l) => /^#{1,6}\s/.test(l));
          if (!hasHeading && currentBlock.length > 0) {
            const firstLine = currentBlock[0];
            const metaMatch = firstLine.match(/^\|?([a-zA-Z0-9_.]+):\s?(.*)$/);
            if (metaMatch) {
              const name = metaMatch[1];
              const desc = metaMatch[2];
              let attr = `{kind=${kind}`;
              if (typeInfo) attr += ` type=${typeInfo}`;
              attr += `}`;
              currentBlock[0] = `## ${name} ${attr}`;
              if (desc.trim()) {
                currentBlock.splice(1, 0, desc);
              }
            } else if (isFirstBlock) {
              // モジュールトップレベルで、かつ見出しがない場合
              currentBlock[0] = `# ${firstLine} {kind=module}`;
              kind = "module";
            } else {
              // 強制的に見出しを入れる
              let attr = `{kind=${kind}`;
              if (typeInfo) attr += ` type=${typeInfo}`;
              attr += `}`;
              currentBlock.unshift(`## [unknown] ${attr}`);
            }
          } else if (kind) {
            // ブロック内の最初の見出し行に {kind=...} を付与する
            for (let k = 0; k < currentBlock.length; k++) {
              if (/^#{1,6}\s/.test(currentBlock[k])) {
                let attr = `{kind=${kind}`;
                if (typeInfo) attr += ` type=${typeInfo}`;
                attr += `}`;
                currentBlock[k] += ` ${attr}`;
                break;
              }
            }
          }
        }

        docLines.push(...currentBlock);
        currentBlock = [];
        isFirstBlock = false;
        inDocBlock = false;
      }
    }
  }
  // 末尾でブロックが残っていた場合
  if (inDocBlock) {
    let kind = null;
    if (isFirstBlock) kind = "module";
    if (kind) {
      let hasHeading = currentBlock.some((l) => /^#{1,6}\s/.test(l));
      if (!hasHeading && currentBlock.length > 0) {
        currentBlock[0] = `# ${currentBlock[0]} {kind=${kind}}`;
      } else {
        for (let k = 0; k < currentBlock.length; k++) {
          if (/^#{1,6}\s/.test(currentBlock[k])) {
            currentBlock[k] += ` {kind=${kind}}`;
            break;
          }
        }
      }
    }
    docLines.push(...currentBlock);
  }

  return {
    doctests,
    docText: docLines.join("\n") + "\n",
  };
}

function parseFile(filePath) {
  const text = fs.readFileSync(filePath, "utf-8");
  if (filePath.endsWith(".n.md") || filePath.endsWith(".md"))
    return { kind: "nmd", ...parseNmdText(text), rawText: text };
  if (filePath.endsWith(".nepl"))
    return { kind: "nepl", ...parseNeplText(text), rawText: text };
  return { kind: "unknown", doctests: [], rawText: text };
}

// -----------------------------------------
// .n.md -> AST（HTML 生成の品質を回復）
// -----------------------------------------

function isHeadingLine(line) {
  const m = /^(#{1,6})\s+(.*?)(?:\s+\{(.*)\})?\s*$/.exec(line);
  if (!m) return null;
  const level = m[1].length;
  const text = m[2];
  const attrStr = m[3] || "";
  let kind = null;
  let type = null;

  if (attrStr) {
    const kindMatch = attrStr.match(/kind=([^\s]+)/);
    if (kindMatch) kind = kindMatch[1];
    const typeMatch = attrStr.match(/type=(<.*>)/);
    if (typeMatch) type = typeMatch[1];
  }

  return { level, text, kind, type };
}

function isFenceStart(line) {
  const m = /^```([^\s`]*)\s*$/.exec(line);
  if (!m) return null;
  return { lang: m[1] || "" };
}

function isFenceEnd(line) {
  return /^```\s*$/.test(line);
}

function isBreakClose1(line) {
  return line === ";;;";
}

function isBreakHrClose1(line) {
  return line === "---";
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
    if (prev && prev.type === "text") prev.text += t;
    else out.push({ type: "text", text: t });
  };

  const readUntil = (needle, start) => {
    const idx = s.indexOf(needle, start);
    if (idx === -1) return null;
    return { end: idx, content: s.slice(start, idx) };
  };

  const splitGlossParts = (inner) => {
    // [] の中では / を区切りとして扱わない
    const parts = [];
    let buf = "";
    let bracket = 0;
    for (let k = 0; k < inner.length; k++) {
      const ch = inner[k];
      if (ch === "[") bracket++;
      if (ch === "]") bracket = Math.max(0, bracket - 1);
      if (ch === "/" && bracket === 0) {
        parts.push(buf);
        buf = "";
        continue;
      }
      buf += ch;
    }
    parts.push(buf);
    return parts;
  };

  while (i < s.length) {
    // $$...$$
    if (s.startsWith("$$", i)) {
      const r = readUntil("$$", i + 2);
      if (r) {
        out.push({ type: "math", display: true, text: r.content });
        i = r.end + 2;
        continue;
      }
    }
    // $...$
    if (s[i] === "$") {
      const r = readUntil("$", i + 1);
      if (r) {
        out.push({ type: "math", display: false, text: r.content });
        i = r.end + 1;
        continue;
      }
    }
    // `...`
    if (s[i] === "`") {
      const r = readUntil("`", i + 1);
      if (r) {
        out.push({ type: "code_inline", text: r.content });
        i = r.end + 1;
        continue;
      }
    }
    // Gloss { ... }
    if (s[i] === "{") {
      const r = readUntil("}", i + 1);
      if (r) {
        const parts = splitGlossParts(r.content).map((p) => p.trim());
        if (parts.length >= 2) {
          out.push({
            type: "gloss",
            base: parseInlines(parts[0]),
            notes: parts.slice(1).map((p) => parseInlines(p)),
          });
          i = r.end + 1;
          continue;
        }
      }
    }
    // Link or Ruby
    if (s[i] === "[") {
      const r = readUntil("]", i + 1);
      if (r) {
        const after = s[r.end + 1] || "";
        if (after === "(") {
          const r2 = readUntil(")", r.end + 2);
          if (r2) {
            const textInner = r.content;
            const href = r2.content;
            out.push({
              type: "link",
              text: parseInlines(textInner),
              href: href,
            });
            i = r2.end + 1;
            continue;
          }
        }
        const inner = r.content;
        const slash = inner.indexOf("/");
        if (slash !== -1) {
          const base = inner.slice(0, slash);
          const ruby = inner.slice(slash + 1);
          out.push({
            type: "ruby",
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
  return { type: "document", children: [] };
}

function newSection(level, headingInlines, kind, type) {
  const sec = { type: "section", level, heading: headingInlines, children: [] };
  if (kind) sec.kind = kind;
  if (type) sec.typeInfo = type;
  return sec;
}

function newParagraph(lines) {
  // 段落内の改行は保持（HTML 側で <br/> 化）
  return { type: "paragraph", inlines: parseInlines(lines.join("\n")) };
}

function newHr() {
  return { type: "hr" };
}

function newCodeBlock(lang, codeText) {
  return { type: "code", lang, text: codeText };
}

function parseNmdAst(source) {
  const lines = source.replace(/\r\n/g, "\n").replace(/\r/g, "\n").split("\n");

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
      curContainer().children.push(
        newCodeBlock(fs0.lang, codeLines.join("\n")),
      );
      i = j;
      continue;
    }

    // Heading
    const h = isHeadingLine(line);
    if (h) {
      closeToLevel(h.level);
      const sec = newSection(h.level, parseInlines(h.text), h.kind, h.type);
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
    if (line.trim() === "") {
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
      curContainer().children.push({ type: "list", items });
      i = j;
      continue;
    }

    // Paragraph
    const para = [];
    let j = i;
    while (j < lines.length) {
      const ln = lines[j];
      if (ln.trim() === "") break;
      if (
        isFenceStart(ln) ||
        isHeadingLine(ln) ||
        isBreakHrClose1(ln) ||
        isBreakClose1(ln) ||
        isHrOnly(ln) ||
        /^-\s+/.test(ln)
      )
        break;
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
