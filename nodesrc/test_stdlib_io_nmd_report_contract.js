#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const path = require("node:path");
const { parseFile } = require("./parser");

const repoRoot = path.resolve(__dirname, "..");
const file = path.join(repoRoot, "tests", "stdlib", "io.n.md");
const parsed = parseFile(file);

assert.equal(parsed.doctests.length, 6, "io.n.md doctest count changed");

const doctest = parsed.doctests[2];
assert.equal(doctest.ret, null, "missing-file doctest must not use ret: as an exit-code substitute");
assert.equal(doctest.exit_code, 0, "missing-file doctest must pin exit_code: 0");
assert.deepEqual(
    doctest.tags,
    ["stdio", "normalize_newlines"],
    "missing-file doctest must be a stdout-normalized stdio fixture",
);
assert.equal(
    doctest.stdout,
    "Checked [ok]\n[0] ok\n",
    "missing-file doctest must pin the IoError assertion report stdout",
);
assert.match(
    doctest.code,
    /checks_print_report[\s\S]*checks_exit_code/,
    "missing-file doctest must print the report before returning its exit code",
);

console.log("stdlib io.n.md report contract passed");
