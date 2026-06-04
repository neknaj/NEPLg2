#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");
const { parseFile } = require("./parser");

const repoRoot = path.resolve(__dirname, "..");
const relPath = "tests/stdlib/neplg2_type_arena.n.md";
const file = path.join(repoRoot, relPath);
const parsed = parseFile(file);
const source = fs.readFileSync(file, "utf8");

const expectedCheckCounts = [11, 6, 4, 2, 6, 4];

assert.equal(parsed.doctests.length, expectedCheckCounts.length, "selfhost type arena doctest count changed");
assert.doesNotMatch(
    source,
    /\balloc\d+\.(?:arena|type_id)\b/,
    "selfhost type arena doctests must use public arena allocation accessors, not owner-backed fields",
);

function expectedStdout(count) {
    const statuses = Array.from({ length: count }, () => "ok").join(",");
    const rows = Array.from({ length: count }, (_value, index) => `[${index}] ok`);
    return [`Checked [${statuses}]`, ...rows, ""].join("\n");
}

parsed.doctests.forEach((doctest, index) => {
    const name = `selfhost type arena doctest#${index + 1}`;

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

console.log("selfhost type arena report contract passed");
