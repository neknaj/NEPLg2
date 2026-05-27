#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const path = require("node:path");
const { parseFile } = require("./parser");

const repoRoot = path.resolve(__dirname, "..");
const file = path.join(repoRoot, "stdlib", "alloc", "collections", "vec", "mutation", "pop.nepl");
const parsed = parseFile(file);

assert.equal(parsed.doctests.length, 5, "vec/mutation/pop.nepl doctest count changed");

const doctest = parsed.doctests.find((case_) => case_.code.includes('test_report_new "vec_drop_last_keeps_owner"'));
assert.ok(doctest, "drop_last doctest must stay present");

const expectedStdout = [
    'test_report name="vec_drop_last_keeps_owner" count=1 failed=0',
    'assertion index=0 status=ok kind=eq_i32 label="drop_last length" expected="1" actual="1" message=""',
    "",
].join("\n");

assert.equal(doctest.ret, null, "drop_last doctest must not use ret-only success reporting");
assert.equal(doctest.exit_code, 0, "drop_last doctest must pin exit_code: 0");
assert.deepEqual(doctest.tags, ["stdio", "normalize_newlines"], "drop_last doctest must be a stdout-normalized stdio fixture");
assert.equal(doctest.stdout, expectedStdout, "drop_last doctest must pin the canonical stdout report exactly");
assert.match(
    doctest.code,
    /test_report_new "vec_drop_last_keeps_owner"/,
    "drop_last doctest must construct a named TestReport",
);
assert.match(doctest.code, /test_report_print_stdout\b/, "drop_last doctest must print the report");
assert.match(doctest.code, /test_report_exit_code\b/, "drop_last doctest must derive exit code from the shown report");
assert.match(
    doctest.code,
    /free\s+v[\s\S]*let shown test_report_print_stdout report/,
    "drop_last doctest must keep Vec owner cleanup before printing the report",
);
assert.doesNotMatch(doctest.code, /\bchecks_exit_code\b/, "drop_last doctest must not hide report details behind checks_exit_code");
assert.doesNotMatch(doctest.code, /\bchecks_print_report\b/, "drop_last doctest must not use legacy Checks report output");
assert.doesNotMatch(doctest.code, /\bchecks_new\b/, "drop_last doctest must not use legacy Checks construction");

console.log("stdlib vec pop.nepl report contract passed");
