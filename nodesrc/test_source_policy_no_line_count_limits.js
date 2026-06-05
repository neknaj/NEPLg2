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
    /\blineCount\b[\s\S]{0,80}(?:<=|>=|<|>)\s*\d+/,
    /\b(?:lines|implementationLines)\.length\b\s*(?:<=|>=|<|>)\s*\d+/,
    /\b[A-Za-z_][A-Za-z0-9_]*(?:Lines|LineCount)\.length\b\s*(?:<=|>=|<|>)\s*\d+/,
    /\d+\s*(?:<=|>=|<|>)\s*\b(?:lineCount|lines\.length|implementationLines\.length)\b/,
    /\.split\(["']\\n["']\)\.length\s*<=/,
    /\d+\s*行以内/,
    /(?:within|under|below|less than|fewer than)\s+\d+\s+lines/i,
    /line budget/i,
    /line limit/i,
    /split threshold/i,
    /split review limit/i,
    /implementation lines/i,
    /facade must stay small/i,
    /should stay narrowly scoped/i,
    /responsibility split limit/i,
    /responsibility freeze limit/i,
];

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
