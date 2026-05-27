#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");
const { stripNeplComments } = require("./source_policy/nepl_source_view");

const repoRoot = path.resolve(__dirname, "..");

const targets = [
    "stdlib/std/streamio/scanner.nepl",
    "stdlib/std/streamio/scanner/state.nepl",
];

const forbidden = /\bvec::(?:get|replace|filled|free)<i32>\b/;
const violations = [];

for (const relPath of targets) {
    const src = fs.readFileSync(path.join(repoRoot, relPath), "utf8");
    const code = stripNeplComments(src);
    const lines = code.split(/\r?\n/);
    lines.forEach((line, index) => {
        if (forbidden.test(line)) {
            violations.push(`${relPath}:${index + 1}: ${line.trim()}`);
        }
    });
}

assert.deepEqual(
    violations,
    [],
    `StreamScanner cursor Vec helper calls must stay postfix-free in NEPLg2.1 source:\n${violations.join("\n")}`,
);

console.log("NEPLg2.1 streamio scanner postfix cleanup regression passed");
