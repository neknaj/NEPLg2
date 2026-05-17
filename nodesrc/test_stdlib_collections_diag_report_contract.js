#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const path = require("node:path");
const { parseFile } = require("./parser");

const repoRoot = path.resolve(__dirname, "..");
const file = path.join(repoRoot, "tests", "stdlib", "collections_diag.n.md");
const parsed = parseFile(file);

assert.equal(parsed.doctests.length, 4, "collections_diag doctest count changed");

for (const [index, doctest] of parsed.doctests.entries()) {
    const name = `collections_diag doctest#${index + 1}`;
    assert.equal(doctest.ret, null, `${name} must not use ret: as an exit-code substitute`);
    assert.equal(doctest.exit_code, 0, `${name} must fix the process exit code`);
    assert.deepEqual(
        doctest.tags,
        ["stdio", "normalize_newlines"],
        `${name} must run as a stdout-normalized stdio fixture`,
    );
    assert.equal(
        doctest.stdout,
        "Checked [ok]\n[0] ok\n",
        `${name} must fix the std/test report stdout`,
    );
    assert.match(
        doctest.code,
        /checks_print_report\s+checks[\s\S]*checks_exit_code\s+shown/,
        `${name} must print the assertion report before returning its exit code`,
    );
}

console.log("stdlib collections diag report contract passed");
