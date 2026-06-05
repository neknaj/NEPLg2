#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");

const repoRoot = path.resolve(__dirname, "..");
const scanRoots = [
    "nodesrc",
    "nodesrc/source_policy",
];
const extraScanFiles = [
    "nodesrc/run_source_policy_regressions.js",
    "nodesrc/selfhost_zenn_review_packet.js",
    "nodesrc/selfhost_zenn_review_response_check.js",
    "doc/neplg2/self_host_zenn_review_checklist.md",
    "doc/neplg2/self_host_zenn_review_prompt.md",
];

function read(relPath) {
    return fs.readFileSync(path.join(repoRoot, relPath), "utf8").replace(/\r\n/g, "\n");
}

function walk(dir) {
    const files = [];
    for (const entry of fs.readdirSync(path.join(repoRoot, dir), { withFileTypes: true })) {
        const relPath = path.posix.join(dir.replace(/\\/g, "/"), entry.name);
        if (entry.isDirectory()) {
            files.push(...walk(relPath));
        } else if (entry.isFile() && entry.name.endsWith(".js")) {
            files.push(relPath);
        }
    }
    return files;
}

const scanned = new Set();
for (const root of scanRoots) {
    for (const relPath of walk(root)) {
        if (!relPath.startsWith("nodesrc/test_") && !relPath.startsWith("nodesrc/source_policy/")) {
            continue;
        }
        scanned.add(relPath);
    }
}
for (const relPath of extraScanFiles) {
    if (fs.existsSync(path.join(repoRoot, relPath))) {
        scanned.add(relPath);
    }
}

const forbidden = [
    /\bimplementationLineCount\b/,
    /\bassertLineLimit\b/,
    /\blineLimits\b/,
    /\bmaxLines\b/,
    /\bmaxBytes\b/,
    /\bmaxFileSize\b/,
    /\bmaxCommentLines\b/,
    /\bmaxDocCommentLines\b/,
    /\bmaxCommentCount\b/,
    /\bmaxDocCommentCount\b/,
    /\bmaxCommentLength\b/,
    /\bmaxDocCommentLength\b/,
    /\blineCount\b[\s\S]{0,80}(?:<=|>=|<|>)\s*\d+/,
    /\b(?:commentCount|docCommentCount|commentLength|docCommentLength)\b[\s\S]{0,80}(?:<=|>=|<|>)\s*\d+/,
    /\b(?:lines|implementationLines)\.length\b\s*(?:<=|>=|<|>)\s*\d+/,
    /\b(?:source|text|content)\.length\b\s*(?:<=|>=|<|>)\s*\d+/,
    /\b(?:commentLines|docLines|docCommentLines)\.length\b\s*(?:<=|>=|<|>)\s*\d+/,
    /\b[A-Za-z_][A-Za-z0-9_]*(?:Lines|LineCount)\.length\b\s*(?:<=|>=|<|>)\s*\d+/,
    /\b[A-Za-z_][A-Za-z0-9_]*(?:CommentCount|DocCommentCount|CommentLength|DocCommentLength)\b[\s\S]{0,80}(?:<=|>=|<|>)\s*\d+/,
    /\b[A-Za-z_][A-Za-z0-9_]*(?:Bytes|ByteLength|FileSize)\b[\s\S]{0,80}(?:<=|>=|<|>)\s*\d+/,
    /\d+\s*(?:<=|>=|<|>)\s*\b(?:lineCount|lines\.length|implementationLines\.length)\b/,
    /\d+\s*(?:<=|>=|<|>)\s*\b(?:commentCount|docCommentCount|commentLength|docCommentLength)\b/,
    /\d+\s*(?:<=|>=|<|>)\s*\b(?:source\.length|text\.length|content\.length)\b/,
    /\d+\s*(?:<=|>=|<|>)\s*\b(?:commentLines\.length|docLines\.length|docCommentLines\.length)\b/,
    /\bBuffer\.byteLength\s*\([\s\S]{0,80}\)\s*(?:<=|>=|<|>)\s*\d+/,
    /\bfs\.statSync\s*\([\s\S]{0,80}\)\.size\s*(?:<=|>=|<|>)\s*\d+/,
    /\.split\(["']\\n["']\)\.length\s*<=/,
    /\d+\s*行以内/,
    /\d+\s*行以下/,
    /最大\s*\d+\s*行/,
    /(?:コメント|ドキュメントコメント|doc comment)[^。\n]{0,40}\d+\s*行まで/i,
    /(?:コメント|ドキュメントコメント|doc comment)[^。\n]{0,40}\d+\s*行以下/i,
    /(?:コメント|ドキュメントコメント|doc comment)[^。\n]{0,40}最大\s*\d+\s*行/i,
    /(?:コメント|ドキュメントコメント|doc comment)[^。\n]{0,40}\d+\s*(?:文字|byte|bytes|バイト)まで/i,
    /(?:コメント|ドキュメントコメント|doc comment)[^。\n]{0,40}\d+\s*(?:文字|byte|bytes|バイト)以下/i,
    /(?:ファイルサイズ|ファイル容量|file size)[^。\n]{0,40}\d+\s*(?:byte|bytes|バイト|KB|MB)まで/i,
    /(?:ファイルサイズ|ファイル容量|file size)[^。\n]{0,40}\d+\s*(?:byte|bytes|バイト|KB|MB)以下/i,
    /(?:ファイルサイズ|ファイル容量|file size)[^。\n]{0,40}最大\s*\d+\s*(?:byte|bytes|バイト|KB|MB)/i,
    /(?:within|under|below|less than|fewer than)\s+\d+\s+lines/i,
    /(?:within|under|below|less than|fewer than)\s+\d+\s+bytes/i,
    /line budget/i,
    /line limit/i,
    /comment budget/i,
    /comment limit/i,
    /doc comment budget/i,
    /doc comment limit/i,
    /file size limit/i,
    /size budget/i,
    /split threshold/i,
    /split review limit/i,
    /implementation lines/i,
    /facade must stay small/i,
    /should stay narrowly scoped/i,
    /responsibility split limit/i,
    /responsibility freeze limit/i,
];

const forbiddenSelfCheckSamples = [
    "assert.ok(Buffer.byteLength(source) <= 20000)",
    "assert.ok(fs.statSync(file).size <= 50000)",
    "assert.ok(source.length <= 30000)",
    "assert.ok(commentLines.length <= 120)",
    "assert.ok(docLines.length <= 80)",
    "assert.ok(commentCount <= 120)",
    "assert.ok(docCommentLength <= 6000)",
    "const maxFileSize = 12000",
    "const maxDocCommentLines = 40",
    "const maxCommentCount = 120",
    "const maxDocCommentLength = 6000",
    "doc comment budget",
    "file size limit",
    "500行以下",
    "最大500行",
    "コメントは100行まで",
    "doc commentは80行まで",
    "ドキュメントコメントは最大40行",
    "コメントは600文字以下",
    "ファイルサイズは64KB以下",
    "ファイルサイズは最大64KB",
];

for (const sample of forbiddenSelfCheckSamples) {
    assert.ok(
        forbidden.some((pattern) => pattern.test(sample)),
        `line-count limit guard must also detect size/comment-volume gate sample: ${sample}`,
    );
}

for (const relPath of scanned) {
    if (relPath === "nodesrc/test_source_policy_no_line_count_limits.js") {
        continue;
    }
    const source = read(relPath);
    for (const pattern of forbidden) {
        assert.doesNotMatch(
            source,
            pattern,
            `${relPath} must not enforce line-count limits; use structural responsibility checks and allow detailed documentation comments`,
        );
    }
}

console.log("source policy line-count limit guard passed");
