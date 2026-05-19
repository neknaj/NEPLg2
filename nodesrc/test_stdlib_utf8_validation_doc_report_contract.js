#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");
const { parseFile } = require("./parser");

const repoRoot = path.resolve(__dirname, "..");

const contracts = [
    {
        rel: ["stdlib", "std", "text", "validate.nepl"],
        count: 4,
        reports: [
            ["std_text_validate_module_doc", "valid UTF-8 bytes"],
            ["std_text_validate_lead_kind_doc", "ASCII lead kind"],
            ["std_text_validate_byte_at_doc", "byte at index 1"],
            ["std_text_validate_mem_doc", "valid memory span"],
        ],
    },
    {
        rel: ["stdlib", "alloc", "string", "utf8.nepl"],
        count: 2,
        reports: [
            ["alloc_string_utf8_lead_kind_doc", "ASCII lead kind"],
            ["alloc_string_utf8_validate_mem_doc", "valid memory span"],
        ],
    },
];

for (const contract of contracts) {
    const file = path.join(repoRoot, ...contract.rel);
    const source = fs.readFileSync(file, "utf8");
    const parsed = parseFile(file);
    const relText = contract.rel.join("/");

    assert.equal(parsed.doctests.length, contract.count, `${relText} doctest count changed`);
    assert.doesNotMatch(source, /\/\/:\s*ret:/, `${relText} must not use ret-only doc-comment metadata`);

    for (const [name, label] of contract.reports) {
        const doctest = parsed.doctests.find((case_) => case_.code.includes(`test_report_new "${name}"`));
        assert.ok(doctest, `${name} doctest must stay present`);
        assert.equal(doctest.ret, null, `${name} must not use ret as test-success metadata`);
        assert.equal(doctest.exit_code, 0, `${name} must pin exit_code: 0`);
        assert.deepEqual(doctest.tags, ["stdio", "normalize_newlines"], `${name} must normalize stdout as stdio`);
        assert.equal(
            doctest.stdout,
            [
                `test_report name="${name}" count=1 failed=0`,
                `assertion index=0 status=ok kind=bool label="${label}" expected="true" actual="true" message=""`,
                "",
            ].join("\n"),
            `${name} must pin the canonical stdout report exactly`,
        );
        assert.match(doctest.code, /test_report_print_stdout\b/, `${name} must print the report`);
        assert.match(doctest.code, /test_report_exit_code\b/, `${name} must derive exit code from the shown report`);
        assert.doesNotMatch(doctest.code, /\bchecks_print_report\b/, `${name} must not use legacy Checks report output`);
        assert.doesNotMatch(doctest.code, /\bchecks_exit_code\b/, `${name} must not hide report details`);
    }
}

console.log("stdlib UTF-8 validation doc report contract passed");
