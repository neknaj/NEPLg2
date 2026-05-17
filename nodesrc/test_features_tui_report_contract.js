#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const path = require("node:path");
const { parseFile } = require("./parser");

const repoRoot = path.resolve(__dirname, "..");
const file = path.join(repoRoot, "tests", "stdlib", "features_tui.n.md");
const parsed = parseFile(file);

assert.equal(parsed.doctests.length, 5, "features_tui doctest count changed");

const boxHelpers = parsed.doctests[3];
const expectedStdout = [
    "Checked [ok,ok,ok,ok,ok,ok,ok,ok,ok,ok,ok,ok,ok,ok,ok]",
    "[0] ok",
    "[1] ok",
    "[2] ok",
    "[3] ok",
    "[4] ok",
    "[5] ok",
    "[6] ok",
    "[7] ok",
    "[8] ok",
    "[9] ok",
    "[10] ok",
    "[11] ok",
    "[12] ok",
    "[13] ok",
    "[14] ok",
    "",
].join("\n");

assert.equal(
    boxHelpers.ret,
    null,
    "features_tui box helper doctest must not use ret: as an exit-code substitute",
);
assert.equal(boxHelpers.exit_code, 0, "features_tui box helper doctest must fix exit_code");
assert.deepEqual(
    boxHelpers.tags,
    ["stdio", "normalize_newlines"],
    "features_tui box helper doctest must be a stdout-normalized stdio fixture",
);
assert.equal(
    boxHelpers.stdout,
    expectedStdout,
    "features_tui box helper doctest must fix the std/test report stdout",
);
assert.match(
    boxHelpers.code,
    /checks_print_report\s+checks[\s\S]*checks_exit_code\s+shown/,
    "features_tui box helper doctest must print the report before returning its exit code",
);

console.log("features_tui report contract passed");
