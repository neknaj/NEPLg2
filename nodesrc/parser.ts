// nodesrc/parser.ts
// 目的:
// - .n.md（拡張 Markdown）から doctest と本文を抽出する
// - .nepl 内のドキュメントコメント（//:）から doctest と本文を抽出する
// - HTML 生成向けに、.n.md を AST（見出しネスト / ルビ / Gloss / リンク / コードフェンス /
//   table / strong / em / strike / blockquote / ordered list 等）へ変換する
//
// 方針:
// - doctest は「neplg2:test」行の直後の ```neplg2 コードブロックを 1 テストケースとして扱う
// - コードブロック内の行頭 "|" は「本文では隠したい前置き」だが、テスト実行には含める
// - 外部ライブラリを使用しない

import * as fs from "node:fs";

// ===== Inline Node Types =====

export type InlineNode =
  | { type: "text"; text: string }
  | { type: "code_inline"; text: string }
  | { type: "math"; display: boolean; text: string }
  | { type: "strong"; children: InlineNode[] }
  | { type: "em"; children: InlineNode[] }
  | { type: "strike"; children: InlineNode[] }
  | { type: "ruby"; base: InlineNode[]; ruby: InlineNode[] }
  | { type: "gloss"; base: InlineNode[]; notes: InlineNode[][] }
  | { type: "link"; text: InlineNode[]; href: string }
  | { type: "image"; alt: string; src: string };

export type TableAlign = null | "left" | "center" | "right";

// ===== Block Node Types =====

export type SectionNode = {
  type: "section";
  level: number;
  heading: InlineNode[];
  children: BlockNode[];
  kind?: string;
  typeInfo?: string;
};

export type BlockNode =
  | { type: "document"; children: BlockNode[] }
  | SectionNode
  | { type: "paragraph"; inlines: InlineNode[] }
  | { type: "hr" }
  | { type: "list"; ordered: boolean; items: InlineNode[][] }
  | { type: "code"; lang: string; text: string }
  | { type: "table"; align: TableAlign[]; head: InlineNode[][]; rows: InlineNode[][][] }
  | { type: "blockquote"; children: BlockNode[] };

// ===== Doctest Types =====

interface DiagSpan {
  file: string | null;
  line: number;
  col: number;
}

interface Doctest {
  tags: string[];
  code: string;
  hiddenMap: boolean[];
  stdin: string | null;
  argv: unknown;
  stdout: string | null;
  stderr: string | null;
  ret: unknown;
  diag_ids: number[];
  diag_spans: DiagSpan[];
}

// ===== Doctest Utilities =====

function parseTags(s: string): string[] {
  const m = s.match(/neplg2:test(?:\[([^\]]+)\])?/);
  if (!m) return [];
  if (!m[1]) return [];
  return m[1].split(",").map(x => x.trim()).filter(Boolean);
}

function stripHiddenPrefix(line: string): string {
  if (line.startsWith("|")) {
    const t = line.slice(1);
    return t.startsWith(" ") ? t.slice(1) : t;
  }
  return line;
}

function parseMetaValue(raw: unknown): unknown {
  const s = String(raw ?? "").trim();
  if (!s) return "";
  if (s.startsWith('"') || s.startsWith("'") || s.startsWith("[") || s.startsWith("{")) {
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

function parseRetValue(raw: unknown): unknown {
  const parsed = parseMetaValue(raw);
  if (typeof parsed === "number") return parsed;
  if (typeof parsed !== "string") return parsed;
  const s = parsed.trim();
  if (/^-?\d+$/.test(s)) return parseInt(s, 10);
  if (/^-?(?:\d+\.\d*|\d*\.\d+)$/.test(s)) return parseFloat(s);
  return parsed;
}

function parseDiagSpanEntry(raw: unknown): DiagSpan | null {
  if (raw && typeof raw === "object" && !Array.isArray(raw)) {
    const r = raw as Record<string, unknown>;
    const line = Number(r.line);
    const col = Number(r.col);
    if (!Number.isFinite(line) || !Number.isFinite(col)) return null;
    const fileRaw = r.file;
    return {
      file: fileRaw === undefined || fileRaw === null ? null : String(fileRaw).trim(),
      line,
      col,
    };
  }
  const s = String(raw ?? "").trim();
  if (!s) return null;
  let m = s.match(/^(\d+)\s*:\s*(\d+)$/);
  if (m) return { file: null, line: parseInt(m[1], 10), col: parseInt(m[2], 10) };
  m = s.match(/^(.+):(\d+):(\d+)$/);
  if (m) return { file: String(m[1]).trim(), line: parseInt(m[2], 10), col: parseInt(m[3], 10) };
  return null;
}

function parseDiagSpanList(raw: unknown): DiagSpan[] {
  const parsed = parseMetaValue(raw);
  const vals: unknown[] = Array.isArray(parsed)
    ? parsed
    : String(parsed).split(",").map(x => x.trim()).filter(Boolean);
  return vals.map(v => parseDiagSpanEntry(v)).filter((x): x is DiagSpan => x !== null);
}

function parseMlstrMeta(lines: string[], start: number, opts: ScanOptions): { value: string; nextIndex: number } {
  const parts: string[] = [];
  let j = start;
  while (j < lines.length) {
    const line = opts.lineTransform(lines[j]);
    const m = line.match(/^\s*##:\s?(.*)$/);
    if (!m) break;
    parts.push(m[1]);
    j++;
  }
  return {
    value: parts.length > 0 ? parts.join("\n") + "\n" : "",
    nextIndex: j,
  };
}

// ===== Doctest Scanner =====

interface ScanOptions {
  lineTransform: (raw: string) => string;
  isHiddenLine: (raw: string) => boolean;
  isDocLine?: (raw: string) => boolean;
  fallbackCode?: string | null;
}

function scanForDoctests(lines: string[], opts: ScanOptions): Doctest[] {
  const doctests: Doctest[] = [];
  const fallbackCode = typeof opts.fallbackCode === "string"
    ? (opts.fallbackCode.endsWith("\n") ? opts.fallbackCode : opts.fallbackCode + "\n")
    : null;
  const fallbackHiddenMap = fallbackCode
    ? fallbackCode.replace(/\r\n/g, "\n").split("\n").slice(0, -1).map(() => false)
    : [];

  for (let i = 0; i < lines.length; i++) {
    const raw = lines[i];
    const line = opts.lineTransform(raw);

    const m = line.match(/^\s*neplg2:test(?:\[[^\]]+\])?\s*$/);
    if (!m) continue;

    const tags = parseTags(line);
    const meta: {
      stdin: string | null;
      argv: string | null;
      stdout: string | null;
      stderr: string | null;
      ret: unknown;
      diag_ids: number[];
      diag_spans: DiagSpan[];
    } = {
      stdin: null,
      argv: null,
      stdout: null,
      stderr: null,
      ret: null,
      diag_ids: [],
      diag_spans: [],
    };

    let j = i + 1;
    while (j < lines.length) {
      const raw2 = lines[j];
      const l2 = opts.lineTransform(raw2);
      if (/^\s*neplg2:test(?:\[[^\]]+\])?\s*$/.test(l2)) break;
      const mm = l2.match(
        /^\s*(stdin|argv|stdout|stderr|ret|diag_id|diag_ids|diag_span|diag_spans)\s*:\s*(.*?)\s*$/,
      );
      if (mm) {
        const k = mm[1];
        const rawValue = mm[2] ?? "";
        if ((k === "stdin" || k === "stdout" || k === "stderr") && String(rawValue).trim() === "mlstr:") {
          const parsed = parseMlstrMeta(lines, j + 1, opts);
          meta[k] = parsed.value;
          j = parsed.nextIndex - 1;
        } else if (k === "diag_id") {
          const v = parseInt(String(parseMetaValue(rawValue)).trim(), 10);
          if (Number.isFinite(v)) meta.diag_ids.push(v);
        } else if (k === "diag_ids") {
          const rawVals = parseMetaValue(rawValue);
          const vals: unknown[] = Array.isArray(rawVals)
            ? rawVals
            : String(rawVals).split(",").map(x => x.trim()).filter(Boolean);
          for (const x of vals) {
            const v = parseInt(String(x).trim(), 10);
            if (Number.isFinite(v)) meta.diag_ids.push(v);
          }
        } else if (k === "diag_span") {
          const one = parseDiagSpanEntry(parseMetaValue(rawValue));
          if (one) meta.diag_spans.push(one);
        } else if (k === "diag_spans") {
          meta.diag_spans.push(...parseDiagSpanList(rawValue));
        } else {
          const mk = k as "stdin" | "argv" | "stdout" | "stderr" | "ret";
          if (mk === "ret") {
            meta.ret = parseRetValue(rawValue);
          } else {
            meta[mk] = parseMetaValue(rawValue) as string | null;
          }
        }
      }
      if (/^\s*```\s*neplg2\s*$/.test(l2)) break;
      if (opts.isDocLine && !opts.isDocLine(raw2)) break;
      j++;
    }
    const hasFence = j < lines.length && /^\s*```\s*neplg2\s*$/.test(opts.lineTransform(lines[j]));
    if (!hasFence) {
      if (fallbackCode !== null) {
        doctests.push({
          tags,
          code: fallbackCode,
          hiddenMap: fallbackHiddenMap.slice(),
          stdin: meta.stdin,
          argv: meta.argv,
          stdout: meta.stdout,
          stderr: meta.stderr,
          ret: meta.ret,
          diag_ids: meta.diag_ids,
          diag_spans: meta.diag_spans,
        });
        i = j - 1;
      }
      continue;
    }

    j++;
    const codeLines: string[] = [];
    const hiddenMap: boolean[] = [];
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

// ===== .n.md / .nepl Parsers =====

function parseNmdText(text: string): { doctests: Doctest[] } {
  const lines = text.replace(/\r\n/g, "\n").split("\n");
  const doctests = scanForDoctests(lines, {
    lineTransform: (raw) => raw,
    isHiddenLine: (raw) => raw.startsWith("|"),
    isDocLine: () => true,
  });
  return { doctests };
}

function parseNeplText(text: string): { doctests: Doctest[]; docText: string } {
  const lines = text.replace(/\r\n/g, "\n").split("\n");

  const doctests = scanForDoctests(lines, {
    lineTransform: (raw) => {
      const m = raw.match(/^\s*\/\/:(\|)?\s?(.*)$/);
      if (!m) return "";
      return (m[1] ? "|" : "") + m[2];
    },
    isHiddenLine: (raw) => /^\s*\/\/:\|/.test(raw),
    isDocLine: (raw) => /^\s*\/\/:/.test(raw),
    fallbackCode: text,
  });

  const docLines: string[] = [];
  let inDocBlock = false;
  let currentBlock: string[] = [];
  let isFirstBlock = true;

  for (let i = 0; i < lines.length; i++) {
    const raw = lines[i];
    const m = raw.match(/^\s*\/\/:(\|)?\s?(.*)$/);

    if (m) {
      inDocBlock = true;
      currentBlock.push((m[1] ? "|" : "") + m[2]);
    } else {
      if (inDocBlock) {
        let kind: string | null = null;
        let typeInfo: string | null = null;
        for (let j = i; j < lines.length; j++) {
          const codeLine = lines[j].trim();
          if (codeLine === "" || codeLine.startsWith("//")) continue;
          if (codeLine.startsWith("fn ")) {
            kind = "fn";
            const typeMatch = codeLine.match(/<.*>/);
            if (typeMatch) typeInfo = typeMatch[0];
          } else if (codeLine.startsWith("struct ")) kind = "struct";
          else if (codeLine.startsWith("enum ")) kind = "enum";
          else if (codeLine.startsWith("trait ")) kind = "trait";
          else if (codeLine.startsWith("impl ")) kind = "impl";
          else if (codeLine.startsWith("let ")) kind = "let";
          break;
        }

        if (isFirstBlock && !kind) {
          const hasH1 = currentBlock.some(l => /^#\s/.test(l));
          if (hasH1) kind = "module";
        }

        if (kind) {
          const hasHeading = currentBlock.some(l => /^#{1,6}\s/.test(l));
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
              if (desc.trim()) currentBlock.splice(1, 0, desc);
            } else if (isFirstBlock) {
              currentBlock[0] = `# ${firstLine} {kind=module}`;
              kind = "module";
            } else {
              let attr = `{kind=${kind}`;
              if (typeInfo) attr += ` type=${typeInfo}`;
              attr += `}`;
              currentBlock.unshift(`## [unknown] ${attr}`);
            }
          } else {
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

  if (inDocBlock) {
    let kind: string | null = null;
    if (isFirstBlock) kind = "module";
    if (kind) {
      const hasHeading = currentBlock.some(l => /^#{1,6}\s/.test(l));
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

  return { doctests, docText: docLines.join("\n") + "\n" };
}

// ===== Inline Parser =====

function normalizeNewlinesLiteral(s: string): string {
  return s.replace(/\\n/g, "\n");
}

function splitGlossParts(inner: string): string[] {
  const parts: string[] = [];
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
}

export function parseInlines(text: string): InlineNode[] {
  const s = normalizeNewlinesLiteral(text);
  const out: InlineNode[] = [];
  let i = 0;

  const pushText = (t: string): void => {
    if (!t) return;
    const prev = out[out.length - 1];
    if (prev && prev.type === "text") prev.text += t;
    else out.push({ type: "text", text: t });
  };

  // Find closing delimiter starting from `from`.
  // If skipDouble is given, skip that longer sequence at matches of `close`.
  const findClose = (from: number, close: string, skipDouble?: string): number => {
    let j = from;
    while (j <= s.length - close.length) {
      if (s.startsWith(close, j)) {
        if (skipDouble && s.startsWith(skipDouble, j)) {
          j += skipDouble.length;
          continue;
        }
        return j;
      }
      j++;
    }
    return -1;
  };

  while (i < s.length) {
    // $$...$$ (must check before $)
    if (s.startsWith("$$", i)) {
      const end = findClose(i + 2, "$$");
      if (end >= 0) {
        out.push({ type: "math", display: true, text: s.slice(i + 2, end) });
        i = end + 2;
        continue;
      }
    }

    // $...$
    if (s[i] === "$") {
      const end = findClose(i + 1, "$");
      if (end >= 0) {
        out.push({ type: "math", display: false, text: s.slice(i + 1, end) });
        i = end + 1;
        continue;
      }
    }

    // `...`
    if (s[i] === "`") {
      const end = findClose(i + 1, "`");
      if (end >= 0) {
        out.push({ type: "code_inline", text: s.slice(i + 1, end) });
        i = end + 1;
        continue;
      }
    }

    // ~~...~~
    if (s.startsWith("~~", i)) {
      const end = findClose(i + 2, "~~");
      if (end >= 0) {
        out.push({ type: "strike", children: parseInlines(s.slice(i + 2, end)) });
        i = end + 2;
        continue;
      }
    }

    // **...** (must check before *)
    if (s.startsWith("**", i)) {
      const end = findClose(i + 2, "**");
      if (end >= 0) {
        out.push({ type: "strong", children: parseInlines(s.slice(i + 2, end)) });
        i = end + 2;
        continue;
      }
    }

    // *...* (not **)
    if (s[i] === "*" && !s.startsWith("**", i)) {
      const end = findClose(i + 1, "*", "**");
      if (end >= 0) {
        out.push({ type: "em", children: parseInlines(s.slice(i + 1, end)) });
        i = end + 1;
        continue;
      }
    }

    // {.../...} gloss
    if (s[i] === "{") {
      const closeIdx = findClose(i + 1, "}");
      if (closeIdx >= 0) {
        const inner = s.slice(i + 1, closeIdx);
        const parts = splitGlossParts(inner).map(p => p.trim());
        if (parts.length >= 2) {
          out.push({
            type: "gloss",
            base: parseInlines(parts[0]),
            notes: parts.slice(1).map(p => parseInlines(p)),
          });
          i = closeIdx + 1;
          continue;
        }
      }
    }

    // ![alt](src) image (must check before link)
    if (s.startsWith("![", i)) {
      const closeAlt = s.indexOf("]", i + 2);
      if (closeAlt >= 0 && s[closeAlt + 1] === "(") {
        const closeSrc = s.indexOf(")", closeAlt + 2);
        if (closeSrc >= 0) {
          out.push({ type: "image", alt: s.slice(i + 2, closeAlt), src: s.slice(closeAlt + 2, closeSrc) });
          i = closeSrc + 1;
          continue;
        }
      }
    }

    // [text](url) link or [text/ruby] ruby
    if (s[i] === "[") {
      const closeBracket = s.indexOf("]", i + 1);
      if (closeBracket >= 0) {
        const after = s[closeBracket + 1] ?? "";
        if (after === "(") {
          const closeParen = s.indexOf(")", closeBracket + 2);
          if (closeParen >= 0) {
            out.push({
              type: "link",
              text: parseInlines(s.slice(i + 1, closeBracket)),
              href: s.slice(closeBracket + 2, closeParen),
            });
            i = closeParen + 1;
            continue;
          }
        }
        const inner = s.slice(i + 1, closeBracket);
        const slash = inner.indexOf("/");
        if (slash !== -1) {
          out.push({
            type: "ruby",
            base: parseInlines(inner.slice(0, slash)),
            ruby: parseInlines(inner.slice(slash + 1)),
          });
          i = closeBracket + 1;
          continue;
        }
      }
    }

    pushText(s[i]);
    i++;
  }

  return out;
}

// ===== Block Parse Helpers =====

function isHeadingLine(line: string): { level: number; text: string; kind: string | null; type: string | null } | null {
  const m = /^(#{1,6})\s+(.*?)(?:\s+\{(.*)\})?\s*$/.exec(line);
  if (!m) return null;
  const level = m[1].length;
  const text = m[2];
  const attrStr = m[3] ?? "";
  let kind: string | null = null;
  let type: string | null = null;
  if (attrStr) {
    const kindMatch = attrStr.match(/kind=([^\s]+)/);
    if (kindMatch) kind = kindMatch[1];
    const typeMatch = attrStr.match(/type=(<.*>)/);
    if (typeMatch) type = typeMatch[1];
  }
  return { level, text, kind, type };
}

function isFenceStart(line: string): { lang: string } | null {
  const m = /^```([^\s`]*)\s*$/.exec(line);
  if (!m) return null;
  return { lang: m[1] ?? "" };
}

function isFenceEnd(line: string): boolean {
  return /^```\s*$/.test(line);
}

function isBreakHrClose1(line: string): boolean {
  return line === "---";
}

function isBreakClose1(line: string): boolean {
  return line === ";;;";
}

function isHrOnly(line: string): boolean {
  return /^-{4,}\s*$/.test(line);
}

// ===== Table Parsing =====

function parseTableCells(line: string): string[] {
  const trimmed = line.trim();
  const inner = trimmed.startsWith("|") ? trimmed.slice(1) : trimmed;
  const withoutTrail = inner.endsWith("|") ? inner.slice(0, -1) : inner;
  return withoutTrail.split("|").map(c => c.trim());
}

function isTableLine(line: string): boolean {
  return line.trim().startsWith("|");
}

function isTableSeparatorLine(line: string): boolean {
  const cells = parseTableCells(line);
  return cells.length > 0 && cells.every(c => /^:?-+:?$/.test(c.trim()));
}

function parseTableAlign(sepLine: string): TableAlign[] {
  return parseTableCells(sepLine).map(cell => {
    if (/^:[-]+:$/.test(cell)) return "center";
    if (/^:[-]+$/.test(cell)) return "left";
    if (/^[-]+:$/.test(cell)) return "right";
    return null;
  });
}

// ===== Block AST Builder =====

type ContainerNode = { children: BlockNode[] };

function parseNmdAstFromLines(lines: string[]): BlockNode[] {
  const rootChildren: BlockNode[] = [];

  type StackEntry = { level: number; node: ContainerNode };
  const stack: StackEntry[] = [{ level: 0, node: { children: rootChildren } }];

  const curContainer = (): ContainerNode => stack[stack.length - 1].node;

  const closeToLevel = (level: number): void => {
    while (stack.length > 1 && stack[stack.length - 1].level >= level) {
      stack.pop();
    }
  };

  const closeOne = (): void => {
    if (stack.length > 1) stack.pop();
  };

  // Test if a line breaks out of a paragraph
  const isBlockStarter = (ln: string, nextLn?: string): boolean => {
    if (ln.trim() === "") return true;
    if (isFenceStart(ln)) return true;
    if (isHeadingLine(ln)) return true;
    if (isBreakHrClose1(ln)) return true;
    if (isBreakClose1(ln)) return true;
    if (isHrOnly(ln)) return true;
    if (ln.startsWith(">")) return true;
    if (/^[-*]\s/.test(ln)) return true;
    if (/^\d+\.\s/.test(ln)) return true;
    if (isTableLine(ln) && nextLn !== undefined && isTableSeparatorLine(nextLn)) return true;
    return false;
  };

  let i = 0;
  while (i < lines.length) {
    const line = lines[i];

    // Code fence
    const fs0 = isFenceStart(line);
    if (fs0) {
      let j = i + 1;
      const codeLines: string[] = [];
      while (j < lines.length && !isFenceEnd(lines[j])) {
        codeLines.push(lines[j]);
        j++;
      }
      if (j < lines.length && isFenceEnd(lines[j])) j++;
      curContainer().children.push({ type: "code", lang: fs0.lang, text: codeLines.join("\n") });
      i = j;
      continue;
    }

    // Heading
    const h = isHeadingLine(line);
    if (h) {
      closeToLevel(h.level);
      const sec: BlockNode = {
        type: "section",
        level: h.level,
        heading: parseInlines(h.text),
        children: [],
        ...(h.kind ? { kind: h.kind } : {}),
        ...(h.type ? { typeInfo: h.type } : {}),
      };
      curContainer().children.push(sec);
      stack.push({ level: h.level, node: sec as unknown as ContainerNode });
      i++;
      continue;
    }

    // Breaks / Hr
    if (isBreakHrClose1(line)) {
      curContainer().children.push({ type: "hr" });
      closeOne();
      i++;
      continue;
    }
    if (isBreakClose1(line)) {
      closeOne();
      i++;
      continue;
    }
    if (isHrOnly(line)) {
      curContainer().children.push({ type: "hr" });
      i++;
      continue;
    }

    // Blank
    if (line.trim() === "") {
      i++;
      continue;
    }

    // Blockquote
    if (line.startsWith(">")) {
      const bqLines: string[] = [];
      let j = i;
      while (j < lines.length) {
        const ln = lines[j];
        if (ln.startsWith(">")) {
          bqLines.push(ln.replace(/^>\s?/, ""));
          j++;
        } else if (ln.trim() === "" && j > i && j + 1 < lines.length && lines[j + 1].startsWith(">")) {
          // blank line between blockquote paragraphs
          bqLines.push("");
          j++;
        } else {
          break;
        }
      }
      // Remove trailing blank lines
      while (bqLines.length > 0 && bqLines[bqLines.length - 1].trim() === "") bqLines.pop();
      const bqChildren = parseNmdAstFromLines(bqLines);
      curContainer().children.push({ type: "blockquote", children: bqChildren });
      i = j;
      continue;
    }

    // Table
    if (isTableLine(line) && i + 1 < lines.length && isTableSeparatorLine(lines[i + 1])) {
      const headCells = parseTableCells(line).map(c => parseInlines(c));
      const align = parseTableAlign(lines[i + 1]);
      const rows: InlineNode[][][] = [];
      let j = i + 2;
      while (j < lines.length && isTableLine(lines[j]) && !isTableSeparatorLine(lines[j])) {
        rows.push(parseTableCells(lines[j]).map(c => parseInlines(c)));
        j++;
      }
      curContainer().children.push({ type: "table", align, head: headCells, rows });
      i = j;
      continue;
    }

    // Ordered list
    if (/^\d+\.\s/.test(line)) {
      const items: InlineNode[][] = [];
      let j = i;
      while (j < lines.length && /^\d+\.\s/.test(lines[j])) {
        const m = lines[j].match(/^\d+\.\s+(.*)/);
        if (m) items.push(parseInlines(m[1]));
        j++;
      }
      curContainer().children.push({ type: "list", ordered: true, items });
      i = j;
      continue;
    }

    // Unordered list
    if (/^[-*]\s/.test(line)) {
      const items: InlineNode[][] = [];
      let j = i;
      while (j < lines.length && /^[-*]\s/.test(lines[j])) {
        const m = lines[j].match(/^[-*]\s+(.*)/);
        if (m) items.push(parseInlines(m[1]));
        j++;
      }
      curContainer().children.push({ type: "list", ordered: false, items });
      i = j;
      continue;
    }

    // Paragraph: collect consecutive non-block lines
    const para: string[] = [];
    let j = i;
    while (j < lines.length) {
      const ln = lines[j];
      const nextLn = lines[j + 1];
      if (isBlockStarter(ln, nextLn)) break;
      para.push(ln);
      j++;
    }
    if (para.length > 0) {
      curContainer().children.push({ type: "paragraph", inlines: parseInlines(para.join("\n")) });
    }
    i = j;
  }

  return rootChildren;
}

export function parseNmdAst(source: string): BlockNode {
  const lines = source.replace(/\r\n/g, "\n").replace(/\r/g, "\n").split("\n");
  const children = parseNmdAstFromLines(lines);
  return { type: "document", children };
}

// ===== File Parser =====

function parseFile(filePath: string): { kind: "nmd" | "nepl" | "unknown"; doctests: Doctest[]; rawText: string; docText?: string } {
  const text = fs.readFileSync(filePath, "utf-8");
  if (filePath.endsWith(".n.md") || filePath.endsWith(".md"))
    return { kind: "nmd", ...parseNmdText(text), rawText: text };
  if (filePath.endsWith(".nepl"))
    return { kind: "nepl", ...parseNeplText(text), rawText: text };
  return { kind: "unknown", doctests: [], rawText: text };
}

module.exports = {
  parseFile,
  parseNmdText,
  parseNeplText,
  parseNmdAst,
  parseInlines,
};
