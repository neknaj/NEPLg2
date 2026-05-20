#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const path = require("node:path");
const { parseFile } = require("./parser");

const repoRoot = path.resolve(__dirname, "..");
const file = path.join(repoRoot, "tests", "stdlib", "neplg2_checker.n.md");
const parsed = parseFile(file);

const expectedDoctests = [
    { title: "summarizes_module_items_with_typed_kind_match", checkCount: 11 },
    { title: "rejects_duplicate_singleton_directives", checkCount: 2 },
    { title: "rejects_raw_text_without_matching_raw_block", checkCount: 1 },
    { title: "rejects_declaration_items_without_parser_header_evidence", checkCount: 1 },
];

assert.equal(parsed.doctests.length, expectedDoctests.length, "selfhost checker doctest count changed");

function expectedStdout(count) {
    const statuses = Array.from({ length: count }, () => "ok").join(",");
    const rows = Array.from({ length: count }, (_value, index) => `[${index}] ok`);
    return [`Checked [${statuses}]`, ...rows, ""].join("\n");
}

parsed.doctests.forEach((doctest, index) => {
    const expected = expectedDoctests[index];
    const name = `selfhost checker ${expected.title}`;

    assert.equal(doctest.ret, null, `${name} must not use ret: as an exit-code substitute`);
    assert.equal(doctest.exit_code, 0, `${name} must pin exit_code: 0`);
    assert.deepEqual(
        doctest.tags,
        ["stdio", "normalize_newlines"],
        `${name} must be a stdout-normalized stdio fixture`,
    );
    assert.equal(
        doctest.stdout,
        expectedStdout(expected.checkCount),
        `${name} must pin the std/test report stdout`,
    );
    assert.match(
        doctest.code,
        /checks_print_report[\s\S]*checks_exit_code/,
        `${name} must print the report before returning its exit code`,
    );
});

console.log("selfhost checker report contract passed");
