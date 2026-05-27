#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");

const repoRoot = path.resolve(__dirname, "..");

const collectionDirs = [
    "stdlib/alloc/collections/bloom_filter",
    "stdlib/alloc/collections/counting_bloom_filter",
    "stdlib/alloc/collections/stack",
];

const violations = [];

for (const relDir of collectionDirs) {
    const absDir = path.join(repoRoot, relDir);
    const publicHelpers = collectPublicFunctionNames(absDir);
    const helperCallPattern = new RegExp(`\\b(?:${publicHelpers.map(escapeRegExp).join("|")})<`);

    for (const relPath of listNeplFiles(absDir)) {
        const absPath = path.join(repoRoot, relPath);
        const lines = fs.readFileSync(absPath, "utf8").split(/\r?\n/);
        let pendingCompileFailFence = false;
        let inDocFence = false;
        let skipCurrentFence = false;

        lines.forEach((line, index) => {
            if (!line.startsWith("//:")) {
                return;
            }

            if (/^\/\/:\s*neplg2:test\[.*\bcompile_fail\b/.test(line)) {
                pendingCompileFailFence = true;
            }

            if (/^\/\/:\s*```/.test(line)) {
                if (!inDocFence) {
                    inDocFence = true;
                    skipCurrentFence = pendingCompileFailFence;
                    pendingCompileFailFence = false;
                    return;
                }

                inDocFence = false;
                skipCurrentFence = false;
                return;
            }

            if (skipCurrentFence) {
                return;
            }

            if (helperCallPattern.test(line)) {
                violations.push(`${relPath}:${index + 1}: ${line.trim()}`);
            }
        });
    }
}

assert.deepEqual(
    violations,
    [],
    `NEPLg2.1 filter/stack positive doctests must not use old public helper postfix type arguments:\n${violations.join("\n")}`,
);

console.log("NEPLg2.1 filter/stack positive doctest postfix cleanup regression passed");

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

function collectPublicFunctionNames(rootDir) {
    const names = new Set();

    for (const relPath of listNeplFiles(rootDir)) {
        const absPath = path.join(repoRoot, relPath);
        const lines = fs.readFileSync(absPath, "utf8").split(/\r?\n/);

        for (const line of lines) {
            const match = /^\s*pub\s+fn\s+([A-Za-z_][A-Za-z0-9_]*)\b/.exec(line);
            if (match) {
                names.add(match[1]);
            }
        }
    }

    assert.notEqual(names.size, 0, `${path.relative(repoRoot, rootDir)} public function list must not be empty`);
    return [...names].sort((a, b) => b.length - a.length || a.localeCompare(b));
}

function escapeRegExp(value) {
    return value.replace(/[\\^$.*+?()[\]{}|]/g, "\\$&");
}
