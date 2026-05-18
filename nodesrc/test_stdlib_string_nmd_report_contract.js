#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const path = require("node:path");
const { parseFile } = require("./parser");

const repoRoot = path.resolve(__dirname, "..");
const file = path.join(repoRoot, "tests", "stdlib", "string.n.md");
const parsed = parseFile(file);

assert.equal(parsed.doctests.length, 17, "tests/stdlib/string.n.md doctest count changed");

const doctest = parsed.doctests[16];

assert.equal(doctest.ret, null, "test_string_builder_linear_build must not use ret: as an exit-code substitute");
assert.equal(doctest.exit_code, 0, "test_string_builder_linear_build must pin exit_code: 0");
assert.deepEqual(
    doctest.tags,
    ["stdio", "normalize_newlines"],
    "test_string_builder_linear_build must be a stdout-normalized stdio fixture",
);
assert.equal(
    doctest.stdout,
    "Checked [ok]\n[0] ok\n",
    "test_string_builder_linear_build must pin the std/test report stdout",
);
assert.match(
    doctest.code,
    /checks_print_report[\s\S]*checks_exit_code/,
    "test_string_builder_linear_build must print the report before returning its exit code",
);

console.log("tests/stdlib string report contract passed");
