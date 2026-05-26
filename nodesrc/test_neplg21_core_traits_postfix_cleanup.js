#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");

const repoRoot = path.resolve(__dirname, "..");

const checks = [
    {
        relPath: "stdlib/core/traits/deserialize.nepl",
        forbiddenPatterns: [
            ["deserialize primitive postfix", /\bdeserialize<i32>/],
            ["parse_err_to_std primitive postfix", /\bparse_err_to_std<(?:bool|i32|i64|i128|f32|f64)>/],
        ],
    },
    {
        relPath: "stdlib/core/traits/hash.nepl",
        forbiddenPatterns: [
            ["old Hasher prose type notation", /Hasher<\.K>/, "nonDeclaration"],
            ["old custom Hasher prose type notation", /Hasher<MyKey>/],
            ["old Hash prose type notation", /Hash<(?:i32|bool|u8|i64|str)>/],
        ],
    },
    {
        relPath: "stdlib/core/traits/serialize.nepl",
        forbiddenPatterns: [
            ["old Serialize prose type notation", /Serialize<T,\s*F>/],
        ],
    },
];

const violations = [];

for (const check of checks) {
    const filePath = path.join(repoRoot, check.relPath);
    const text = fs.readFileSync(filePath, "utf8");
    const lines = text.split(/\r?\n/);
    lines.forEach((line, index) => {
        const trimmed = line.trim();
        for (const [label, pattern, mode] of check.forbiddenPatterns) {
            if (mode === "nonDeclaration" && /^(pub\s+)?(fn|impl|trait|struct|enum)\b/.test(trimmed)) {
                continue;
            }
            if (pattern.test(line)) {
                violations.push(`${check.relPath}:${index + 1}: ${label}: ${trimmed}`);
            }
        }
    });
}

assert.deepEqual(
    violations,
    [],
    `NEPLg2.1 core traits docs and implementations must not reintroduce selected generic postfixes:\n${violations.join("\n")}`,
);

console.log("NEPLg2.1 core traits postfix cleanup regression passed");
