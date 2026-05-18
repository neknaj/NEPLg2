#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const path = require("node:path");
const { parseFile } = require("./parser");

const repoRoot = path.resolve(__dirname, "..");
const file = path.join(repoRoot, "tests", "stdlib", "traits_hash.n.md");
const parsed = parseFile(file);

assert.equal(parsed.doctests.length, 6, "traits_hash doctest count changed");

const expectedStdoutByIndex = new Map([
    [0, "Checked [ok,ok,ok]\n[0] ok\n[1] ok\n[2] ok\n"],
    [4, "Checked [ok,ok,ok]\n[0] ok\n[1] ok\n[2] ok\n"],
    [5, "Checked [ok,ok]\n[0] ok\n[1] ok\n"],
]);

for (const [index, expectedStdout] of expectedStdoutByIndex) {
    const doctest = parsed.doctests[index];
    assert.equal(
        doctest.ret,
        null,
        `traits_hash doctest#${index + 1} must not use ret: as an exit-code substitute`,
    );
    assert.equal(
        doctest.exit_code,
        0,
        `traits_hash doctest#${index + 1} must pin exit_code: 0`,
    );
    assert.deepEqual(
        doctest.tags,
        ["stdio", "normalize_newlines"],
        `traits_hash doctest#${index + 1} must be a stdout-normalized stdio fixture`,
    );
    assert.equal(
        doctest.stdout,
        expectedStdout,
        `traits_hash doctest#${index + 1} must pin the assertion report stdout`,
    );
    assert.match(
        doctest.code,
        /checks_print_report[\s\S]*checks_exit_code/,
        `traits_hash doctest#${index + 1} must print the report before returning its exit code`,
    );
}

console.log("stdlib traits_hash report contract passed");
