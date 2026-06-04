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
    "test_report name=\"features_tui_box_helpers_clamp_narrow_widths\" count=15 failed=0",
    "assertion index=0 status=ok kind=str_eq label=\"assert_str_eq\" expected=\"\" actual=\"\" message=\"\"",
    "assertion index=1 status=ok kind=str_eq label=\"assert_str_eq\" expected=\"\" actual=\"\" message=\"\"",
    "assertion index=2 status=ok kind=str_eq label=\"assert_str_eq\" expected=\"\" actual=\"\" message=\"\"",
    "assertion index=3 status=ok kind=str_eq label=\"assert_str_eq\" expected=\"\" actual=\"\" message=\"\"",
    "assertion index=4 status=ok kind=str_eq label=\"assert_str_eq\" expected=\"\" actual=\"\" message=\"\"",
    "assertion index=5 status=ok kind=str_eq label=\"assert_str_eq\" expected=\"\" actual=\"\" message=\"\"",
    "assertion index=6 status=ok kind=str_eq label=\"assert_str_eq\" expected=\"\" actual=\"\" message=\"\"",
    "assertion index=7 status=ok kind=str_eq label=\"assert_str_eq\" expected=\"\" actual=\"\" message=\"\"",
    "assertion index=8 status=ok kind=str_eq label=\"assert_str_eq\" expected=\"\" actual=\"\" message=\"\"",
    "assertion index=9 status=ok kind=str_eq label=\"assert_str_eq\" expected=\"\" actual=\"\" message=\"\"",
    "assertion index=10 status=ok kind=str_eq label=\"assert_str_eq\" expected=\"\" actual=\"\" message=\"\"",
    "assertion index=11 status=ok kind=str_eq label=\"assert_str_eq\" expected=\"a\" actual=\"a\" message=\"\"",
    "assertion index=12 status=ok kind=str_eq label=\"assert_str_eq\" expected=\"ab\" actual=\"ab\" message=\"\"",
    "assertion index=13 status=ok kind=str_eq label=\"assert_str_eq\" expected=\"\" actual=\"\" message=\"\"",
    "assertion index=14 status=ok kind=str_eq label=\"assert_str_eq\" expected=\"\x1b[31m\x1b[44ma\x1b[0m\" actual=\"\x1b[31m\x1b[44ma\x1b[0m\" message=\"\"",
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
    /test_report_print_stdout\s+checks[\s\S]*test_report_exit_code\s+shown/,
    "features_tui box helper doctest must print the report before returning its exit code",
);
assert.doesNotMatch(
    boxHelpers.code,
    /\bchecks_(?:new|push|print_report|exit_code)\b/,
    "features_tui box helper doctest must not use the legacy checks_* report API",
);

console.log("features_tui report contract passed");
