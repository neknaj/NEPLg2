#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const path = require("node:path");
const { parseFile } = require("./parser");

const repoRoot = path.resolve(__dirname, "..");
const file = path.join(repoRoot, "tests", "stdlib", "selfhost_req.n.md");
const parsed = parseFile(file);

const expected = [
    ["selfhost_req_file_io", "bool", "missing file returns err", "true", "true"],
    ["selfhost_req_byte_manipulation", "eq_i32", "first byte as i32", "222", "222"],
    ["selfhost_req_string_utils", "eq_i32", "trim slice result code", "0", "0"],
    ["selfhost_req_string_map", "eq_i32", "hash map string key value", "10", "10"],
    ["selfhost_req_string_builder", "eq_i32", "builder length", "20", "20"],
    ["selfhost_req_trait_extensions", "eq_i32", "trait key value length", "5", "5"],
];

assert.equal(parsed.doctests.length, expected.length, "selfhost_req doctest count changed");

function expectedStdout([name, kind, label, expectedValue, actualValue]) {
    return `test_report name="${name}" count=1 failed=0\nassertion index=0 status=ok kind=${kind} label="${label}" expected="${expectedValue}" actual="${actualValue}" message=""\n`;
}

parsed.doctests.forEach((doctest, index) => {
    const name = `${expected[index][0]} doctest#${index + 1}`;

    assert.equal(doctest.ret, null, `${name} must not use ret: as an exit-code substitute`);
    assert.equal(doctest.exit_code, 0, `${name} must pin exit_code: 0`);
    assert.deepEqual(
        doctest.tags,
        ["stdio", "normalize_newlines"],
        `${name} must be a stdout-normalized stdio fixture`,
    );
    assert.equal(doctest.stdout, expectedStdout(expected[index]), `${name} must pin TestReport stdout`);
    assert.match(
        doctest.code,
        /#import\s+"std\/test"\s+as\s+\*[\s\S]*test_report_print_stdout[\s\S]*test_report_exit_code/,
        `${name} must print a TestReport before returning its exit code`,
    );
});

console.log("selfhost_req report contract passed");
