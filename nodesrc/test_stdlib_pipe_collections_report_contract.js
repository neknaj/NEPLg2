#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const path = require("node:path");
const { parseFile } = require("./parser");

const repoRoot = path.resolve(__dirname, "..");
const file = path.join(repoRoot, "tests", "stdlib", "pipe_collections.n.md");
const parsed = parseFile(file);

assert.equal(parsed.doctests.length, 8, "pipe_collections doctest count changed");

const expectedCheckCounts = [2, 2, 3, 3, 3, 2, 2, 2];

function expectedStdout(count) {
    const labels = Array.from({ length: count }, () => "ok").join(",");
    const lines = [`Checked [${labels}]`];
    for (let i = 0; i < count; i += 1) {
        lines.push(`[${i}] ok`);
    }
    return `${lines.join("\n")}\n`;
}

for (const [index, expectedCount] of expectedCheckCounts.entries()) {
    const doctest = parsed.doctests[index];
    assert.equal(
        doctest.ret,
        null,
        `pipe_collections doctest#${index + 1} must not use ret: as an exit-code substitute`,
    );
    assert.equal(
        doctest.exit_code,
        0,
        `pipe_collections doctest#${index + 1} must pin exit_code: 0`,
    );
    assert.deepEqual(
        doctest.tags,
        ["stdio", "normalize_newlines"],
        `pipe_collections doctest#${index + 1} must be a stdout-normalized stdio fixture`,
    );
    assert.equal(
        doctest.stdout,
        expectedStdout(expectedCount),
        `pipe_collections doctest#${index + 1} must pin the assertion report stdout`,
    );
    assert.match(
        doctest.code,
        /checks_print_report[\s\S]*checks_exit_code/,
        `pipe_collections doctest#${index + 1} must print the report before returning its exit code`,
    );
}

console.log("stdlib pipe_collections report contract passed");
