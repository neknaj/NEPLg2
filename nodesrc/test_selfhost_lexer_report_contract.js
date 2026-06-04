#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const path = require("node:path");
const { parseFile } = require("./parser");

const repoRoot = path.resolve(__dirname, "..");
const file = path.join(repoRoot, "tests", "stdlib", "neplg2_lexer.n.md");
const parsed = parseFile(file);

const expectedCheckCounts = [19, 17, 2, 11, 3, 3, 3, 3, 4, 3, 43, 9, 19, 17];

assert.equal(parsed.doctests.length, expectedCheckCounts.length, "selfhost lexer doctest count changed");

function expectedStdout(count) {
    const statuses = Array.from({ length: count }, () => "ok").join(",");
    const rows = Array.from({ length: count }, (_value, index) => `[${index}] ok`);
    return [`Checked [${statuses}]`, ...rows, ""].join("\n");
}

parsed.doctests.forEach((doctest, index) => {
    const name = `selfhost lexer doctest#${index + 1}`;

    assert.equal(doctest.ret, null, `${name} must not use ret: as an exit-code substitute`);
    assert.equal(doctest.exit_code, 0, `${name} must pin exit_code: 0`);
    assert.deepEqual(
        doctest.tags,
        ["stdio", "normalize_newlines"],
        `${name} must be a stdout-normalized stdio fixture`,
    );
    assert.equal(
        doctest.stdout,
        expectedStdout(expectedCheckCounts[index]),
        `${name} must pin the std/test report stdout`,
    );
    assert.match(
        doctest.code,
        /checks_print_report[\s\S]*checks_exit_code/,
        `${name} must print the report before returning its exit code`,
    );
});

console.log("selfhost lexer report contract passed");
