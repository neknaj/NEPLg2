#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const path = require("node:path");
const { parseFile } = require("./parser");

const repoRoot = path.resolve(__dirname, "..");
const file = path.join(repoRoot, "tests", "stdlib", "selfhost_cli_driver.n.md");
const parsed = parseFile(file);

assert.equal(parsed.doctests.length, 3, "selfhost_cli_driver doctest count changed");

const expectedReport = ["Checked [ok,ok]", "[0] ok", "[1] ok", ""].join("\n");

for (const index of [0, 2]) {
    const doctest = parsed.doctests[index];
    const name = `selfhost_cli_driver doctest#${index + 1}`;

    assert.equal(doctest.ret, null, `${name} must not use ret: as an exit-code substitute`);
    assert.equal(doctest.exit_code, 0, `${name} must fix exit_code`);
    assert.deepEqual(
        doctest.tags,
        ["stdio", "normalize_newlines"],
        `${name} must be a stdout-normalized stdio fixture`,
    );
    assert.equal(doctest.stdout, expectedReport, `${name} must fix the std/test report stdout`);
    assert.match(
        doctest.code,
        /checks_print_report\s+checks[\s\S]*checks_exit_code\s+shown/,
        `${name} must print the report before returning its exit code`,
    );
    assert.doesNotMatch(
        doctest.code,
        /\bv::(?:new|push)<str>/,
        `${name} must rely on Vec str expected type or receiver evidence instead of explicit producer or mutator postfixes`,
    );
}

console.log("selfhost CLI driver report contract passed");
