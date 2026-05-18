#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const path = require("node:path");
const { parseFile } = require("./parser");

const repoRoot = path.resolve(__dirname, "..");
const file = path.join(repoRoot, "tests", "stdlib", "traits_serde.n.md");
const parsed = parseFile(file);

assert.equal(parsed.doctests.length, 2, "traits_serde doctest count changed");

const expectedStdouts = [
    "Checked [ok,ok,ok,ok]\n[0] ok\n[1] ok\n[2] ok\n[3] ok\n",
    "Checked [ok,ok,ok]\n[0] ok\n[1] ok\n[2] ok\n",
];

for (const [index, expectedStdout] of expectedStdouts.entries()) {
    const doctest = parsed.doctests[index];
    assert.equal(
        doctest.ret,
        null,
        `traits_serde doctest#${index + 1} must not use ret: as an exit-code substitute`,
    );
    assert.equal(
        doctest.exit_code,
        0,
        `traits_serde doctest#${index + 1} must pin exit_code: 0`,
    );
    assert.deepEqual(
        doctest.tags,
        ["stdio", "normalize_newlines"],
        `traits_serde doctest#${index + 1} must be a stdout-normalized stdio fixture`,
    );
    assert.equal(
        doctest.stdout,
        expectedStdout,
        `traits_serde doctest#${index + 1} must pin the assertion report stdout`,
    );
    assert.match(
        doctest.code,
        /checks_print_report[\s\S]*checks_exit_code/,
        `traits_serde doctest#${index + 1} must print the report before returning its exit code`,
    );
}

assert.match(
    parsed.doctests[0].code,
    /#import\s+"core\/cast"\s+as\s+\*/,
    "serialize doctest must import core/cast before using cast",
);

console.log("stdlib traits_serde report contract passed");
