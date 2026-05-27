#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");

const repoRoot = path.resolve(__dirname, "..");
const targetDir = path.join(repoRoot, "doc", "examples");
const oldImpureFunctionSyntax = /%fn\*|\bfn\*/;

const violations = [];

for (const entry of fs.readdirSync(targetDir, { withFileTypes: true })) {
    if (!entry.isFile() || !entry.name.endsWith(".nepl")) {
        continue;
    }
    const relPath = path.join("doc", "examples", entry.name).replaceAll(path.sep, "/");
    const lines = fs.readFileSync(path.join(targetDir, entry.name), "utf8").split(/\r?\n/);
    lines.forEach((line, index) => {
        if (oldImpureFunctionSyntax.test(line)) {
            violations.push(`${relPath}:${index + 1}: ${line.trim()}`);
        }
    });
}

assert.deepEqual(
    violations,
    [],
    `NEPLg2.1 doc examples must use impure fn instead of the old fn* draft spelling:\n${violations.join("\n")}`,
);

console.log("NEPLg2.1 doc examples impure fn cleanup regression passed");
