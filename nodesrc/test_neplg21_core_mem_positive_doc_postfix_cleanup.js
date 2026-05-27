#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");

const repoRoot = path.resolve(__dirname, "..");

const coreMemSources = [
    "stdlib/core/mem.nepl",
    ...listNeplFiles(path.join(repoRoot, "stdlib/core/mem")),
];

const migratedHelpers = [
    "alloc_region",
    "alloc_region_bytes",
    "dealloc_region",
    "mem_ptr_add",
    "mem_ptr_wrap",
    "realloc_region_bytes_keep",
    "region_new",
    "region_ptr_at",
    "region_realloc_error_region",
];

const helperCallPattern = new RegExp(`\\b(?:${migratedHelpers.map(escapeRegExp).join("|")})<`);
const violations = [];

for (const relPath of coreMemSources) {
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

        if (!inDocFence || skipCurrentFence) {
            return;
        }

        if (helperCallPattern.test(line)) {
            violations.push(`${relPath}:${index + 1}: ${line.trim()}`);
        }
    });
}

assert.deepEqual(
    violations,
    [],
    `NEPLg2.1 core/mem positive doctests must not use old postfix type arguments for value-evidenced helpers:\n${violations.join("\n")}`,
);

console.log("NEPLg2.1 core/mem positive doctest postfix cleanup regression passed");

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

function escapeRegExp(value) {
    return value.replace(/[\\^$.*+?()[\]{}|]/g, "\\$&");
}
