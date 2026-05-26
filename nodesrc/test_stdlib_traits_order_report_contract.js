#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const path = require("node:path");
const { parseFile } = require("./parser");

const repoRoot = path.resolve(__dirname, "..");
const file = path.join(repoRoot, "tests", "stdlib", "traits_order.n.md");
const parsed = parseFile(file);

assert.equal(parsed.doctests.length, 3, "traits_order doctest count changed");

const doctest = parsed.doctests[2];

assert.equal(doctest.ret, null, "vec sort doctest must not use ret: as an exit-code substitute");
assert.equal(doctest.exit_code, 0, "vec sort doctest must pin exit_code: 0");
assert.deepEqual(
    doctest.tags,
    ["stdio", "normalize_newlines"],
    "vec sort doctest must be a stdout-normalized stdio fixture",
);
assert.equal(
    doctest.stdout,
    "Checked [ok,ok,ok,ok]\n[0] ok\n[1] ok\n[2] ok\n[3] ok\n",
    "vec sort doctest must pin each sorted-position assertion in stdout",
);
assert.match(
    doctest.code,
    /checks_print_report[\s\S]*checks_exit_code/,
    "vec sort doctest must print the report before returning its exit code",
);
assert.doesNotMatch(
    doctest.code,
    /\b(?:new|push)<i32>/,
    "vec sort doctest must rely on NEPLg2.1 Vec expected type or receiver evidence instead of explicit producer or mutator postfixes",
);

console.log("stdlib traits_order report contract passed");
