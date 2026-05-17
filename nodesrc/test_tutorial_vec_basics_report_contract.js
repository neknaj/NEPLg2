#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const path = require("node:path");

const { parseFile } = require("./parser.js");

const repoRoot = path.resolve(__dirname, "..");
const relPath = "tutorials/getting_started/13_vec_basics.n.md";
const parsed = parseFile(path.join(repoRoot, relPath));

assert.equal(parsed.doctests.length, 1, "Vec basics tutorial must keep exactly one runnable doctest");

const doctest = parsed.doctests[0];

assert.deepEqual(
    doctest.tags,
    ["stdio", "normalize_newlines"],
    "Vec basics doctest must opt into stdout execution and normalized output",
);
assert.equal(doctest.ret, null, "Vec basics doctest must not use ret: as a process exit-code substitute");
assert.equal(doctest.exit_code, 0, "Vec basics doctest must pin exit_code: 0");
assert.ok(doctest.stdout && doctest.stdout.length > 0, "Vec basics doctest must pin a stdout report");
assert.match(
    doctest.stdout,
    /^Checked \[ok,ok,ok\]\n\[0\] ok\n\[1\] ok\n\[2\] ok\n$/,
    "Vec basics doctest must keep the deterministic std/test stdout report",
);
assert.match(
    doctest.code,
    /checks_print_report[\s\S]*checks_exit_code/,
    "Vec basics doctest must print the check report before deriving the exit code",
);
assert.equal(
    [...doctest.code.matchAll(/free<i32>\s+vec_push_error_vec<i32>\s+e/g)].length,
    2,
    "Vec basics doctest must recover and free the Vec owner from both push error paths",
);

console.log("tutorial vec basics report contract regression passed");
