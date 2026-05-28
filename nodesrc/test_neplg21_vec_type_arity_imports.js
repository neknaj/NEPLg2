#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");

const repoRoot = path.resolve(__dirname, "..");
const stdlibRoot = path.join(repoRoot, "stdlib");

function collectNeplFiles(dir, out = []) {
    for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
        const full = path.join(dir, entry.name);
        if (entry.isDirectory()) {
            collectNeplFiles(full, out);
        } else if (entry.isFile() && entry.name.endsWith(".nepl")) {
            out.push(full);
        }
    }
    return out;
}

function repoPath(filePath) {
    return path.relative(repoRoot, filePath).replace(/\\/g, "/");
}

function stripsDocComments(source) {
    return source
        .replace(/\r\n/g, "\n")
        .split("\n")
        .filter((line) => !/^\s*\/\//.test(line))
        .join("\n");
}

function isVecImplementationFile(filePath) {
    const rel = repoPath(filePath);
    return rel === "stdlib/alloc/collections/vec.nepl" ||
        rel.startsWith("stdlib/alloc/collections/vec/");
}

const violations = [];
for (const filePath of collectNeplFiles(stdlibRoot)) {
    if (isVecImplementationFile(filePath)) {
        continue;
    }
    const source = fs.readFileSync(filePath, "utf8");
    const executableSource = stripsDocComments(source);
    const usesVecPrefixType = /(^|[\s%&])Vec\s+[A-Za-z_.]/m.test(executableSource);
    const importsVecTypeArity =
        /#import\s+"alloc\/collections\/vec(?:\/types)?"/.test(executableSource);
    if (usesVecPrefixType && !importsVecTypeArity) {
        violations.push(repoPath(filePath));
    }
}

assert.deepEqual(
    violations,
    [],
    "NEPLg2.1 prefix type parsing needs each executable module that writes Vec T types to import alloc/collections/vec/types or alloc/collections/vec directly:\n" +
        violations.join("\n"),
);
