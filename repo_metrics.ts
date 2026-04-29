#!/usr/bin/env node
/// <reference types="node" />

import { readFileSync, readdirSync, statSync, writeFileSync } from "node:fs";
import { resolve, relative, extname } from "node:path";
import { spawnSync } from "node:child_process";

type Mode = "auto" | "git" | "walk";
type SuffixMode = "last" | "all";
type BinaryMode = "skip" | "bytes";
type AreaKind = "top_level_docs_tests" | "source_tree" | "other";
type LineKind = "blank" | "source" | "doc_comment" | "document" | "test" | "comment" | "other";

type Args = {
    root: string;
    mode: Mode;
    suffixMode: SuffixMode;
    maxBytes: number;
    binary: BinaryMode;
    csv: string | null;
    json: string | null;
};

type FileStats = {
    lines: number;
    chars: number;
    bytes: number;
    blank: number;
    source: number;
    doc_comment: number;
    document: number;
    test: number;
    comment: number;
    other: number;
    testCases: number;
    kindChars: Record<LineKind, number>;
    kindBytes: Record<LineKind, number>;
};

type BucketStats = FileStats & {
    files: number;
};

type SimpleStats = {
    files: number;
    lines: number;
    chars: number;
    bytes: number;
    testCases: number;
};

type TextLine = {
    text: string;
    rawBytes: number;
};

const TOP_LEVEL_DOC_TEST_DIRS = new Set(["tests", "tutorials", "doc", "examples"]);
const CONTENT_KINDS: readonly LineKind[] = ["blank", "source", "doc_comment", "document", "test", "comment", "other"];
const SOURCE_EXTS = new Set([
    ".c", ".cpp", ".css", ".h", ".hpp", ".html", ".java", ".js", ".jsx",
    ".mjs", ".mts", ".nepl", ".py", ".rb", ".rs", ".sh", ".sql", ".ts",
    ".tsx", ".wat", ".wast", ".wasm", ".yaml", ".yml",
]);
const MARKDOWN_EXTS = new Set([".md", ".n.md"]);
const DOCTEST_META_RE = /^\s*(stdin|argv|stdout|stderr|ret|diag_code|diag_codes|diag_span|diag_spans)\s*:\s*(.*?)\s*$/;
const DOCTEST_RE = /^\s*neplg2:test(?:\[[^\]]+\])?\s*$/;
const DOCTEST_FENCE_OPEN_RE = /^\s*```neplg2\s*$/;
const DOCTEST_FENCE_CLOSE_RE = /^\s*```\s*$/;
const NEPL_DOC_RE = /^\s*\/\/:(\|)?\s?(.*)$/;
const RUST_DOC_RE = /^\s*(\/\/\/|\/\/!)/;
const RUST_COMMENT_RE = /^\s*\/\//;
const RUST_CFG_TEST_RE = /^\s*#\[\s*cfg\s*\(\s*test\s*\)\s*\]/;
const RUST_TEST_ATTR_RE = /^\s*#\[(?:test|tokio::test|wasm_bindgen_test)\b/;
const RUST_FN_RE = /^\s*(?:pub\s+)?(?:async\s+)?fn\b/;

function emptyFileStats(): FileStats {
    return {
        lines: 0,
        chars: 0,
        bytes: 0,
        blank: 0,
        source: 0,
        doc_comment: 0,
        document: 0,
        test: 0,
        comment: 0,
        other: 0,
        testCases: 0,
        kindChars: {
            blank: 0,
            source: 0,
            doc_comment: 0,
            document: 0,
            test: 0,
            comment: 0,
            other: 0,
        },
        kindBytes: {
            blank: 0,
            source: 0,
            doc_comment: 0,
            document: 0,
            test: 0,
            comment: 0,
            other: 0,
        },
    };
}

function emptyBucketStats(): BucketStats {
    return {
        files: 0,
        ...emptyFileStats(),
    };
}

function emptySimpleStats(): SimpleStats {
    return {
        files: 0,
        lines: 0,
        chars: 0,
        bytes: 0,
        testCases: 0,
    };
}

function run(cmd: string[], cwd: string): Buffer {
    const proc = spawnSync(cmd[0], cmd.slice(1), {
        cwd,
        encoding: "buffer",
        stdio: ["ignore", "pipe", "pipe"],
    });
    if (proc.status !== 0) {
        const err = (proc.stderr ?? Buffer.alloc(0)).toString("utf8").trim();
        throw new Error(err || `Command failed: ${cmd.join(" ")}`);
    }
    return proc.stdout ?? Buffer.alloc(0);
}

function isGitRepo(path: string): boolean {
    try {
        run(["git", "rev-parse", "--is-inside-work-tree"], path);
        return true;
    } catch {
        return false;
    }
}

function gitRoot(path: string): string {
    return run(["git", "rev-parse", "--show-toplevel"], path).toString("utf8").trim();
}

function listGitTrackedAndUnignored(root: string): string[] {
    const out = run(["git", "ls-files", "-z", "--cached", "--others", "--exclude-standard"], root);
    return out
        .toString("utf8")
        .split("\0")
        .map((v) => v.trim())
        .filter((v) => v.length > 0);
}

function listFilesWalk(root: string): string[] {
    const out: string[] = [];
    const stack = [root];
    while (stack.length > 0) {
        const current = stack.pop()!;
        const entries = readdirSync(current, { withFileTypes: true });
        for (const entry of entries) {
            const abs = `${current}/${entry.name}`;
            if (entry.isDirectory()) {
                if (entry.name === ".git") continue;
                stack.push(abs);
                continue;
            }
            if (entry.isFile()) {
                out.push(relative(root, abs).replace(/\\/g, "/"));
            }
        }
    }
    out.sort();
    return out;
}

function isProbablyBinary(path: string, sampleSize = 8192): boolean {
    try {
        const buf = readFileSync(path);
        const head = buf.subarray(0, sampleSize);
        return head.includes(0);
    } catch {
        return true;
    }
}

function extKey(relPath: string, suffixMode: SuffixMode): string {
    const parts = relPath.split("/").pop() ?? relPath;
    if (suffixMode === "all") {
        const idx = parts.indexOf(".");
        return idx >= 0 ? parts.slice(idx).toLowerCase() : "(no_ext)";
    }
    const ext = extname(parts).toLowerCase();
    return ext || "(no_ext)";
}

function classifyArea(relPath: string): AreaKind {
    const parts = relPath.split("/").filter(Boolean);
    if (parts.length === 0) return "other";
    if (TOP_LEVEL_DOC_TEST_DIRS.has(parts[0])) return "top_level_docs_tests";
    if (parts[0] === "stdlib" || parts.includes("src")) return "source_tree";
    return "other";
}

function isTestPath(relPath: string): boolean {
    return relPath.split("/").includes("tests");
}

function addLine(stats: FileStats, kind: LineKind, line: TextLine): void {
    stats.lines += 1;
    stats.chars += line.text.length;
    stats.bytes += line.rawBytes;
    if (line.text.trim() === "") {
        stats.blank += 1;
        stats.kindChars.blank += line.text.length;
        stats.kindBytes.blank += line.rawBytes;
        return;
    }
    stats[kind] += 1;
    stats.kindChars[kind] += line.text.length;
    stats.kindBytes[kind] += line.rawBytes;
}

function readTextLines(path: string, maxBytes: number | null): TextLine[] {
    const raw = readFileSync(path);
    if (maxBytes !== null && maxBytes > 0 && raw.length > maxBytes) {
        throw new Error(`file too large (${raw.length} bytes) > maxBytes`);
    }
    const text = raw.toString("utf8");
    const rawLines = raw.toString("binary").match(/[^\r\n]*(?:\r\n|\r|\n|$)/g) ?? [];
    const textLines = text.match(/[^\r\n]*(?:\r\n|\r|\n|$)/g) ?? [];
    const out: TextLine[] = [];
    const count = Math.max(rawLines.length, textLines.length);
    for (let i = 0; i < count; i++) {
        const rawLine = rawLines[i] ?? "";
        const textLine = textLines[i] ?? "";
        if (i === count - 1 && rawLine === "" && textLine === "") continue;
        out.push({ text: textLine, rawBytes: Buffer.byteLength(rawLine, "binary") });
    }
    return out;
}

function classifyMarkdownLines(lines: TextLine[]): FileStats {
    const stats = emptyFileStats();
    let state: "document" | "await_fence" | "in_fence" = "document";

    for (const line of lines) {
        const stripped = line.text.replace(/[\r\n]+$/, "");
        if (state === "document") {
            if (DOCTEST_RE.test(stripped)) {
                addLine(stats, "test", line);
                stats.testCases += 1;
                state = "await_fence";
            } else {
                addLine(stats, "document", line);
            }
            continue;
        }

        if (state === "await_fence") {
            if (DOCTEST_META_RE.test(stripped)) {
                addLine(stats, "test", line);
            } else if (DOCTEST_FENCE_OPEN_RE.test(stripped)) {
                addLine(stats, "test", line);
                state = "in_fence";
            } else {
                addLine(stats, "document", line);
                state = "document";
            }
            continue;
        }

        addLine(stats, "test", line);
        if (DOCTEST_FENCE_CLOSE_RE.test(stripped)) {
            state = "document";
        }
    }

    return stats;
}

function classifyNeplLines(relPath: string, lines: TextLine[]): FileStats {
    const stats = emptyFileStats();
    const testFile = isTestPath(relPath);
    let state: "document_comment" | "await_fence" | "in_fence" = "document_comment";

    for (const line of lines) {
        const stripped = line.text.replace(/[\r\n]+$/, "");
        const m = stripped.match(NEPL_DOC_RE);
        if (m) {
            const docText = `${m[1] ? "|" : ""}${m[2] ?? ""}`;
            if (state === "document_comment") {
                if (DOCTEST_RE.test(docText)) {
                    addLine(stats, "test", line);
                    stats.testCases += 1;
                    state = "await_fence";
                } else {
                    addLine(stats, "doc_comment", line);
                }
            } else if (state === "await_fence") {
                if (DOCTEST_META_RE.test(docText)) {
                    addLine(stats, "test", line);
                } else if (DOCTEST_FENCE_OPEN_RE.test(docText)) {
                    addLine(stats, "test", line);
                    state = "in_fence";
                } else {
                    addLine(stats, "doc_comment", line);
                    state = "document_comment";
                }
            } else {
                addLine(stats, "test", line);
                if (DOCTEST_FENCE_CLOSE_RE.test(docText)) {
                    state = "document_comment";
                }
            }
            continue;
        }

        if (line.text.trim() === "") {
            addLine(stats, "other", line);
        } else if (testFile) {
            addLine(stats, "test", line);
        } else if (stripped.trimStart().startsWith("//")) {
            addLine(stats, "comment", line);
        } else {
            addLine(stats, "source", line);
        }
    }

    return stats;
}

function classifyRustLines(relPath: string, lines: TextLine[]): FileStats {
    const stats = emptyFileStats();
    const testFile = isTestPath(relPath);
    let braceDepth = 0;
    const testRegionEnds: number[] = [];
    let pendingCfgTest = false;
    let pendingTestAttr = false;

    for (const line of lines) {
        const stripped = line.text.replace(/[\r\n]+$/, "");
        const logical = stripped.trim();
        const inTestRegion = testFile || testRegionEnds.length > 0;
        const isCfgTest = RUST_CFG_TEST_RE.test(stripped);
        const isTestAttr = RUST_TEST_ATTR_RE.test(stripped);
        const isDoc = RUST_DOC_RE.test(stripped);

        if (logical === "") {
            addLine(stats, "other", line);
        } else if (isCfgTest || isTestAttr) {
            addLine(stats, "test", line);
            if (isCfgTest) pendingCfgTest = true;
            if (isTestAttr) {
                pendingTestAttr = true;
                stats.testCases += 1;
            }
        } else if (isDoc) {
            addLine(stats, "doc_comment", line);
        } else if (pendingCfgTest || pendingTestAttr || inTestRegion) {
            addLine(stats, "test", line);
        } else if (RUST_COMMENT_RE.test(stripped)) {
            addLine(stats, "comment", line);
        } else {
            addLine(stats, "source", line);
        }

        const depthBefore = braceDepth;
        const opens = (stripped.match(/{/g) ?? []).length;
        const closes = (stripped.match(/}/g) ?? []).length;

        if (pendingCfgTest && logical !== "" && !isCfgTest) {
            if (stripped.includes("{")) {
                testRegionEnds.push(depthBefore);
                pendingCfgTest = false;
            } else if (stripped.endsWith(";")) {
                pendingCfgTest = false;
            }
        }

        if (pendingTestAttr && logical !== "" && !isTestAttr) {
            if (RUST_FN_RE.test(stripped) && stripped.includes("{")) {
                testRegionEnds.push(depthBefore);
                pendingTestAttr = false;
            } else if (!stripped.startsWith("#[") && stripped.includes("{")) {
                testRegionEnds.push(depthBefore);
                pendingTestAttr = false;
            } else if (stripped.endsWith(";")) {
                pendingTestAttr = false;
            }
        }

        braceDepth += opens - closes;
        while (testRegionEnds.length > 0 && braceDepth <= testRegionEnds[testRegionEnds.length - 1]) {
            testRegionEnds.pop();
        }
    }

    return stats;
}

function classifyGenericLines(relPath: string, lines: TextLine[]): FileStats {
    const stats = emptyFileStats();
    const key = extKey(relPath, "all");
    const testFile = isTestPath(relPath);
    const isMarkdown = MARKDOWN_EXTS.has(key) || extname(relPath).toLowerCase() === ".md";
    const isSource = SOURCE_EXTS.has(extname(relPath).toLowerCase());

    for (const line of lines) {
        if (line.text.trim() === "") {
            addLine(stats, "other", line);
        } else if (isMarkdown) {
            addLine(stats, "document", line);
        } else if (testFile) {
            addLine(stats, "test", line);
        } else if (isSource) {
            addLine(stats, "source", line);
        } else {
            addLine(stats, "other", line);
        }
    }

    return stats;
}

function measureTextFile(relPath: string, absPath: string, maxBytes: number | null): FileStats {
    const lines = readTextLines(absPath, maxBytes);
    const key = extKey(relPath, "all");
    const suffix = extname(relPath).toLowerCase();
    if (key === ".n.md" || suffix === ".md") return classifyMarkdownLines(lines);
    if (suffix === ".nepl") return classifyNeplLines(relPath, lines);
    if (suffix === ".rs") return classifyRustLines(relPath, lines);
    return classifyGenericLines(relPath, lines);
}

function accumulateBucket(dest: BucketStats, src: FileStats): void {
    dest.lines += src.lines;
    dest.chars += src.chars;
    dest.bytes += src.bytes;
    dest.blank += src.blank;
    dest.source += src.source;
    dest.doc_comment += src.doc_comment;
    dest.document += src.document;
    dest.test += src.test;
    dest.comment += src.comment;
    dest.other += src.other;
    dest.testCases += src.testCases;
}

function accumulateSimple(dest: SimpleStats, files: number, lines: number, chars: number, bytes: number, testCases: number): void {
    dest.files += files;
    dest.lines += lines;
    dest.chars += chars;
    dest.bytes += bytes;
    dest.testCases += testCases;
}

function sortBuckets<T extends { bytes: number; lines: number; files: number }>(entries: Array<[string, T]>): Array<[string, T]> {
    return [...entries].sort((a, b) => {
        const sa = a[1];
        const sb = b[1];
        if (sb.bytes !== sa.bytes) return sb.bytes - sa.bytes;
        if (sb.lines !== sa.lines) return sb.lines - sa.lines;
        if (sb.files !== sa.files) return sb.files - sa.files;
        return a[0].localeCompare(b[0]);
    });
}

function printBucketTable(title: string, keyName: string, stats: Map<string, BucketStats>): void {
    const rows = sortBuckets(Array.from(stats.entries()));
    const headers = [
        keyName, "files", "lines", "chars", "bytes", "blank", "source",
        "doc_comment", "document", "test", "comment", "other", "test_cases",
    ];
    const data = rows.map(([key, s]) => [
        key,
        formatNum(s.files),
        formatNum(s.lines),
        formatNum(s.chars),
        formatNum(s.bytes),
        formatNum(s.blank),
        formatNum(s.source),
        formatNum(s.doc_comment),
        formatNum(s.document),
        formatNum(s.test),
        formatNum(s.comment),
        formatNum(s.other),
        formatNum(s.testCases),
    ]);
    const widths = calcWidths(headers, data);
    console.log(title);
    console.log(formatRow(headers, widths));
    console.log(formatRow(headers.map((h) => "-".repeat(h.length)), widths));
    for (const row of data) {
        console.log(formatRow(row, widths));
    }
    const total = emptyBucketStats();
    for (const [, s] of stats) {
        total.files += s.files;
        accumulateBucket(total, s);
    }
    console.log("");
    console.log(formatRow([
        "TOTAL",
        formatNum(total.files),
        formatNum(total.lines),
        formatNum(total.chars),
        formatNum(total.bytes),
        formatNum(total.blank),
        formatNum(total.source),
        formatNum(total.doc_comment),
        formatNum(total.document),
        formatNum(total.test),
        formatNum(total.comment),
        formatNum(total.other),
        formatNum(total.testCases),
    ], widths));
}

function printSimpleTable(title: string, keyName: string, stats: Map<string, SimpleStats>): void {
    const rows = sortBuckets(Array.from(stats.entries()));
    const headers = [keyName, "files", "lines", "chars", "bytes", "test_cases"];
    const data = rows.map(([key, s]) => [
        key,
        formatNum(s.files),
        formatNum(s.lines),
        formatNum(s.chars),
        formatNum(s.bytes),
        formatNum(s.testCases),
    ]);
    const widths = calcWidths(headers, data);
    console.log(title);
    console.log(formatRow(headers, widths));
    console.log(formatRow(headers.map((h) => "-".repeat(h.length)), widths));
    for (const row of data) {
        console.log(formatRow(row, widths));
    }
    const total = emptySimpleStats();
    for (const [, s] of stats) {
        accumulateSimple(total, s.files, s.lines, s.chars, s.bytes, s.testCases);
    }
    console.log("");
    console.log(formatRow([
        "TOTAL",
        formatNum(total.files),
        formatNum(total.lines),
        formatNum(total.chars),
        formatNum(total.bytes),
        formatNum(total.testCases),
    ], widths));
}

function calcWidths(headers: string[], data: string[][]): number[] {
    const widths = headers.map((h) => h.length);
    for (const row of data) {
        for (let i = 0; i < row.length; i++) {
            widths[i] = Math.max(widths[i], row[i].length);
        }
    }
    return widths;
}

function formatRow(row: string[], widths: number[]): string {
    return row.map((cell, i) => (i === 0 ? cell.padEnd(widths[i]) : cell.padStart(widths[i]))).join("  ");
}

function formatNum(value: number): string {
    return value.toLocaleString("en-US");
}

function writeCsv(path: string, extStats: Map<string, BucketStats>, areaStats: Map<string, BucketStats>, kindStats: Map<string, SimpleStats>): void {
    const lines: string[] = [];
    lines.push([
        "section", "name", "files", "lines", "chars", "bytes",
        "blank", "source", "doc_comment", "document", "test", "comment", "other", "test_cases",
    ].join(","));
    for (const [section, table] of [["extension", extStats], ["area", areaStats]] as const) {
        for (const [name, s] of Array.from(table.entries()).sort()) {
            lines.push([
                section, csvEsc(name), String(s.files), String(s.lines), String(s.chars), String(s.bytes),
                String(s.blank), String(s.source), String(s.doc_comment), String(s.document),
                String(s.test), String(s.comment), String(s.other), String(s.testCases),
            ].join(","));
        }
    }
    for (const [name, s] of Array.from(kindStats.entries()).sort()) {
        lines.push([
            "content_kind",
            csvEsc(name),
            String(s.files),
            String(s.lines),
            String(s.chars),
            String(s.bytes),
            "",
            "",
            "",
            "",
            "",
            "",
            "",
            String(s.testCases),
        ].join(","));
    }
    writeFileSync(path, `${lines.join("\n")}\n`, "utf8");
}

function csvEsc(value: string): string {
    if (!/[",\n]/.test(value)) return value;
    return `"${value.replace(/"/g, "\"\"")}"`;
}

function writeJson(
    path: string,
    extStats: Map<string, BucketStats>,
    areaStats: Map<string, BucketStats>,
    kindStats: Map<string, SimpleStats>,
    skipped: Array<{ path: string; reason: string }>,
): void {
    const payload = {
        byExtension: Array.from(extStats.entries()).sort().map(([name, stats]) => ({
            name,
            files: stats.files,
            lines: stats.lines,
            chars: stats.chars,
            bytes: stats.bytes,
            blank: stats.blank,
            source: stats.source,
            doc_comment: stats.doc_comment,
            document: stats.document,
            test: stats.test,
            comment: stats.comment,
            other: stats.other,
            testCases: stats.testCases,
        })),
        byArea: Array.from(areaStats.entries()).sort().map(([name, stats]) => ({
            name,
            files: stats.files,
            lines: stats.lines,
            chars: stats.chars,
            bytes: stats.bytes,
            blank: stats.blank,
            source: stats.source,
            doc_comment: stats.doc_comment,
            document: stats.document,
            test: stats.test,
            comment: stats.comment,
            other: stats.other,
            testCases: stats.testCases,
        })),
        byContentKind: Array.from(kindStats.entries()).sort().map(([name, stats]) => ({ name, ...stats })),
        skipped,
    };
    writeFileSync(path, `${JSON.stringify(payload, null, 2)}\n`, "utf8");
}

function parseArgs(argv: string[]): Args {
    let root = ".";
    let mode: Mode = "auto";
    let suffixMode: SuffixMode = "all";
    let maxBytes = 5_000_000;
    let binary: BinaryMode = "skip";
    let csv: string | null = null;
    let json: string | null = null;

    for (let i = 0; i < argv.length; i++) {
        const arg = argv[i];
        if (arg === "--root" && i + 1 < argv.length) {
            root = argv[++i];
            continue;
        }
        if (arg === "--mode" && i + 1 < argv.length) {
            mode = argv[++i] as Mode;
            continue;
        }
        if (arg === "--suffix-mode" && i + 1 < argv.length) {
            suffixMode = argv[++i] as SuffixMode;
            continue;
        }
        if (arg === "--max-bytes" && i + 1 < argv.length) {
            maxBytes = Number(argv[++i]);
            continue;
        }
        if (arg === "--binary" && i + 1 < argv.length) {
            binary = argv[++i] as BinaryMode;
            continue;
        }
        if (arg === "--csv" && i + 1 < argv.length) {
            csv = argv[++i];
            continue;
        }
        if (arg === "--json" && i + 1 < argv.length) {
            json = argv[++i];
            continue;
        }
        if (arg === "-h" || arg === "--help") {
            printUsage();
            process.exit(0);
        }
        throw new Error(`unknown argument: ${arg}`);
    }

    if (!["auto", "git", "walk"].includes(mode)) {
        throw new Error(`--mode must be auto|git|walk, got: ${mode}`);
    }
    if (!["last", "all"].includes(suffixMode)) {
        throw new Error(`--suffix-mode must be last|all, got: ${suffixMode}`);
    }
    if (!["skip", "bytes"].includes(binary)) {
        throw new Error(`--binary must be skip|bytes, got: ${binary}`);
    }

    return {
        root,
        mode,
        suffixMode,
        maxBytes,
        binary,
        csv,
        json,
    };
}

function printUsage(): void {
    console.log("Usage: node --experimental-strip-types repo_metrics.ts [options]");
    console.log("");
    console.log("Options:");
    console.log("  --root <path>         Repo root or subdir (default: .)");
    console.log("  --mode <auto|git|walk>");
    console.log("  --suffix-mode <all|last>   Extension grouping mode (default: all)");
    console.log("  --max-bytes <n>       Skip text counting above this size (default: 5000000, 0 disables)");
    console.log("  --binary <skip|bytes> Skip binaries or count only file size (default: skip)");
    console.log("  --csv <path>          Write flattened CSV");
    console.log("  --json <path>         Write structured JSON");
}

function main(argv: string[]): number {
    const args = parseArgs(argv);
    let root = resolve(args.root);
    let useGit = false;

    if (args.mode === "auto" || args.mode === "git") {
        useGit = isGitRepo(root);
        if (args.mode === "git" && !useGit) {
            console.error("ERROR: --mode git but not inside a Git repository.");
            return 2;
        }
    }

    const relPaths = useGit ? listGitTrackedAndUnignored(gitRoot(root)) : listFilesWalk(root);
    if (useGit) root = gitRoot(root);

    const extStats = new Map<string, BucketStats>();
    const areaStats = new Map<string, BucketStats>();
    const kindStats = new Map<string, SimpleStats>();
    const skipped: Array<{ path: string; reason: string }> = [];
    const maxBytes = args.maxBytes === 0 ? null : args.maxBytes;

    for (const relPath of relPaths) {
        const absPath = resolve(root, relPath);
        let st;
        try {
            st = statSync(absPath);
        } catch {
            skipped.push({ path: relPath, reason: "unreadable" });
            continue;
        }
        if (!st.isFile()) continue;

        const ext = extKey(relPath, args.suffixMode);
        const area = classifyArea(relPath);

        if (isProbablyBinary(absPath)) {
            if (args.binary === "skip") {
                skipped.push({ path: relPath, reason: "binary" });
                continue;
            }
            if (!extStats.has(ext)) extStats.set(ext, emptyBucketStats());
            if (!areaStats.has(area)) areaStats.set(area, emptyBucketStats());
            extStats.get(ext)!.files += 1;
            extStats.get(ext)!.bytes += st.size;
            areaStats.get(area)!.files += 1;
            areaStats.get(area)!.bytes += st.size;
            continue;
        }

        let measured: FileStats;
        try {
            measured = measureTextFile(relPath, absPath, maxBytes);
        } catch (error) {
            const msg = String(error instanceof Error ? error.message : error);
            skipped.push({ path: relPath, reason: msg.includes("too large") ? "too_large" : "unreadable" });
            continue;
        }

        if (!extStats.has(ext)) extStats.set(ext, emptyBucketStats());
        if (!areaStats.has(area)) areaStats.set(area, emptyBucketStats());
        extStats.get(ext)!.files += 1;
        areaStats.get(area)!.files += 1;
        accumulateBucket(extStats.get(ext)!, measured);
        accumulateBucket(areaStats.get(area)!, measured);

        for (const kind of CONTENT_KINDS) {
            const lines = measured[kind];
            if (lines <= 0) continue;
            if (!kindStats.has(kind)) kindStats.set(kind, emptySimpleStats());
            const bucket = kindStats.get(kind)!;
            bucket.files += 1;
            bucket.lines += lines;
            bucket.chars += measured.kindChars[kind];
            bucket.bytes += measured.kindBytes[kind];
            bucket.testCases += kind === "test" ? measured.testCases : 0;
        }
    }

    printBucketTable("By Extension", "ext", extStats);
    console.log("");
    printBucketTable("By Area", "area", areaStats);
    console.log("");
    printSimpleTable("By Content Kind", "kind", kindStats);

    if (skipped.length > 0) {
        console.log("");
        console.log(`Skipped files: ${skipped.length} (showing up to 20)`);
        for (const item of skipped.slice(0, 20)) {
            console.log(`  - ${item.path} [${item.reason}]`);
        }
        if (skipped.length > 20) {
            console.log("  ...");
        }
    }

    if (args.csv) writeCsv(args.csv, extStats, areaStats, kindStats);
    if (args.json) writeJson(args.json, extStats, areaStats, kindStats, skipped);
    return 0;
}

try {
    process.exitCode = main(process.argv.slice(2));
} catch (error) {
    console.error(String(error instanceof Error ? error.message : error));
    process.exitCode = 1;
}
