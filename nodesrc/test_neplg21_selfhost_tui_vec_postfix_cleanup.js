#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");

const repoRoot = path.resolve(__dirname, "..");

const checks = [
    {
        relPath: "stdlib/platforms/wasix/tui/text/wrap.nepl",
        forbiddenPatterns: [
            ["old Vec str prose notation", /Vec<str>/],
            ["old TUI Vec str helper postfix", /\bv::(?:vec_empty|push|vec_push_error_vec|new)<str>/],
        ],
    },
    {
        relPath: "stdlib/platforms/wasix/tui/text.nepl",
        forbiddenPatterns: [
            ["old Vec str prose notation", /Vec<str>/],
        ],
    },
    {
        relPath: "stdlib/neplg2/core/infra/text.nepl",
        forbiddenPatterns: [
            ["old Vec i32 prose notation", /Vec<i32>/],
            ["old source text Vec i32 helper postfix", /\bv::(?:push|filled)<i32>/],
        ],
    },
    {
        relPath: "stdlib/neplg2/core/mono/mono.nepl",
        forbiddenPatterns: [
            ["old Option mono instance prose notation", /Option<SelfhostMonoInstanceId>/],
            ["old mono record Vec helper postfix", /\bv::(?:new|push|vec_push_error_kind)<SelfhostMonoInstanceRecord>/],
        ],
    },
    {
        relPath: "stdlib/neplg2/core/module/vfs.nepl",
        forbiddenPatterns: [
            ["old VFS Vec prose notation", /Vec<SelfhostVirtualFile>/],
            ["old VFS Vec helper postfix", /\bv::(?:new|push)<SelfhostVirtualFile>/],
        ],
    },
];

const violations = [];

for (const check of checks) {
    const filePath = path.join(repoRoot, check.relPath);
    const lines = fs.readFileSync(filePath, "utf8").split(/\r?\n/);
    lines.forEach((line, index) => {
        for (const [label, pattern] of check.forbiddenPatterns) {
            if (pattern.test(line)) {
                violations.push(`${check.relPath}:${index + 1}: ${label}: ${line.trim()}`);
            }
        }
    });
}

assert.deepEqual(
    violations,
    [],
    `NEPLg2.1 selfhost/TUI Vec cleanup must not reintroduce selected old generic postfixes:\n${violations.join("\n")}`,
);

console.log("NEPLg2.1 selfhost/TUI Vec postfix cleanup regression passed");
