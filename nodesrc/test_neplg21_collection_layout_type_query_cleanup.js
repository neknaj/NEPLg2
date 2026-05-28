#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");

const repoRoot = path.resolve(__dirname, "..");
const relPaths = [
    ...listNeplFiles(path.join(repoRoot, "stdlib/alloc/collections/vec")),
    "tests/compiler/move_effect.n.md",
];
const oldTypeOnlyLayoutCall = /\b(?:size_of|align_of)<[^>\r\n]+>/;

const violations = [];

for (const relPath of relPaths) {
    const lines = fs.readFileSync(path.join(repoRoot, relPath), "utf8").split(/\r?\n/);
    if (relPath.endsWith(".n.md")) {
        collectMarkdownFenceViolations(relPath, lines);
    } else {
        collectNeplSourceViolations(relPath, lines);
    }
}

assert.deepEqual(
    violations,
    [],
    `NEPLg2.1 collection layout queries must use the postfix-free type marker form, for example size_of %.T:\n${violations.join("\n")}`,
);

console.log("NEPLg2.1 collection layout type query cleanup regression passed");

function collectMarkdownFenceViolations(relPath, lines) {
    let inNeplFence = false;
    lines.forEach((line, index) => {
        const trimmed = line.trim();
        if (trimmed === "```neplg2") {
            inNeplFence = true;
            return;
        }
        if (trimmed === "```") {
            inNeplFence = false;
            return;
        }
        if (inNeplFence) {
            collectLineViolation(relPath, index, line);
        }
    });
}

function collectNeplSourceViolations(relPath, lines) {
    let inDocFence = false;
    lines.forEach((line, index) => {
        const trimmed = line.trim();
        if (/^\/\/:\s*```neplg2\b/.test(line)) {
            inDocFence = true;
            return;
        }
        if (/^\/\/:\s*```/.test(line)) {
            inDocFence = false;
            return;
        }

        if (inDocFence) {
            collectLineViolation(relPath, index, stripNeplDocPrefix(line));
            return;
        }

        if (trimmed && !trimmed.startsWith("//")) {
            collectLineViolation(relPath, index, line);
        }
    });
}

function collectLineViolation(relPath, index, line) {
    if (oldTypeOnlyLayoutCall.test(line)) {
        violations.push(`${relPath}:${index + 1}: ${line.trim()}`);
    }
}

function stripNeplDocPrefix(line) {
    return line.replace(/^\/\/:\|?\s?/, "");
}

function listNeplFiles(rootDir) {
    const results = [];
    const entries = fs.readdirSync(rootDir, { withFileTypes: true });
    for (const entry of entries) {
        const fullPath = path.join(rootDir, entry.name);
        if (entry.isDirectory()) {
            results.push(...listNeplFiles(fullPath));
            continue;
        }
        if (entry.isFile() && entry.name.endsWith(".nepl")) {
            results.push(path.relative(repoRoot, fullPath).replace(/\\/g, "/"));
        }
    }
    return results.sort();
}
