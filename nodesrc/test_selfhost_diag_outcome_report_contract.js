#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const path = require("node:path");
const { parseFile } = require("./parser");

const repoRoot = path.resolve(__dirname, "..");
const file = path.join(repoRoot, "tests", "stdlib", "neplg2_diag_outcome.n.md");
const parsed = parseFile(file);

const expectedCheckCounts = [8, 8, null];

assert.equal(parsed.doctests.length, expectedCheckCounts.length, "selfhost diag outcome doctest count changed");

function expectedStdout(count) {
    const statuses = Array.from({ length: count }, () => "ok").join(",");
    const rows = Array.from({ length: count }, (_value, index) => `[${index}] ok`);
    return [`Checked [${statuses}]`, ...rows, ""].join("\n");
}

for (const [index, count] of expectedCheckCounts.entries()) {
    const doctest = parsed.doctests[index];
    const name = `selfhost diag outcome doctest#${index + 1}`;

    assert.equal(doctest.ret, null, `${name} must not use ret: as an exit-code substitute`);
    assert.equal(doctest.exit_code, 0, `${name} must pin exit_code: 0`);

    if (count === null) {
        assert.equal(doctest.stdout, "okerr", `${name} must keep its direct stdout fixture`);
        continue;
    }

    assert.deepEqual(
        doctest.tags,
        ["stdio", "normalize_newlines"],
        `${name} must be a stdout-normalized stdio fixture`,
    );
    assert.equal(
        doctest.stdout,
        expectedStdout(count),
        `${name} must pin the std/test report stdout`,
    );
    assert.match(
        doctest.code,
        /checks_print_report[\s\S]*checks_exit_code/,
        `${name} must print the report before returning its exit code`,
    );
}

console.log("selfhost diag outcome report contract passed");
