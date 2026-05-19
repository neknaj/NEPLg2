#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const path = require("node:path");
const { parseFile } = require("./parser");

const repoRoot = path.resolve(__dirname, "..");
const file = path.join(repoRoot, "stdlib", "tests", "json.n.md");
const parsed = parseFile(file);

assert.equal(parsed.doctests.length, 1, "stdlib/tests/json.n.md doctest count changed");

const doctest = parsed.doctests[0];
const expectedStdout = [
    "Checked [ok,ok,ok,ok,ok,ok,ok,ok,ok,ok,ok,ok,ok]",
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
    "",
].join("\n");

assert.equal(doctest.ret, null, "json_main must not use ret-only success reporting");
assert.equal(doctest.exit_code, 0, "json_main must pin exit_code: 0");
assert.deepEqual(doctest.tags, ["stdio", "normalize_newlines"], "json_main must be a stdout-normalized stdio fixture");
assert.equal(doctest.stdout, expectedStdout, "json_main must pin the std/test report stdout");
assert.match(doctest.code, /checks_print_report[\s\S]*checks_exit_code/, "json_main must print the report before returning its exit code");
assert.match(doctest.code, /\bjson_is_null\b/, "json_main must keep null constructor assertions");
assert.match(doctest.code, /\bjson_as_bool\b/, "json_main must keep bool accessor assertions");
assert.match(doctest.code, /\bjson_as_number\b/, "json_main must keep number accessor assertions");
assert.match(doctest.code, /\bjson_as_string\b/, "json_main must keep string accessor assertions");
assert.match(doctest.code, /\bjson_array_new\b/, "json_main must keep array constructor assertions");
assert.match(doctest.code, /\bjson_object_new\b/, "json_main must keep object constructor assertions");

console.log("stdlib json.n.md report contract passed");
