#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");
const { parseFile } = require("./parser");

const repoRoot = path.resolve(__dirname, "..");

const targets = [
    {
        rel: "stdlib/neplg2/cli/args/parse.nepl",
        reports: [
            { name: "selfhost_cli_parse_args_doc", count: 5 },
            { name: "selfhost_cli_parse_argv_doc", count: 3 },
        ],
    },
    {
        rel: "stdlib/neplg2/cli/args/options.nepl",
        reports: [
            { name: "selfhost_cli_default_options_doc", count: 1 },
            { name: "selfhost_cli_options_to_compile_options_doc", count: 4 },
        ],
    },
];

for (const target of targets) {
    const fullPath = path.join(repoRoot, ...target.rel.split("/"));
    const source = fs.readFileSync(fullPath, "utf8");
    const parsed = parseFile(fullPath);

    assert.equal(
        parsed.doctests.length,
        target.reports.length,
        `${target.rel} must keep the documented CLI args doctest count`,
    );
    assert.doesNotMatch(
        source,
        /\/\/:\s*ret:/,
        `${target.rel} doc-comment doctests must not use ret as test status`,
    );

    for (let index = 0; index < target.reports.length; index += 1) {
        const doctest = parsed.doctests[index];
        const expected = target.reports[index];
        assert.deepEqual(
            doctest.tags,
            ["stdio", "normalize_newlines"],
            `${expected.name} must normalize stdout report output`,
        );
        assert.equal(doctest.exit_code, 0, `${expected.name} must use exit_code metadata`);
        assert.equal(doctest.ret, null, `${expected.name} must not use ret metadata`);
        assert.match(
            doctest.stdout,
            new RegExp(`^test_report name="${expected.name}" count=${expected.count} failed=0\\n`),
            `${expected.name} must publish a canonical std/test report`,
        );
        assert.match(
            doctest.code,
            /\btest_report_new\b[\s\S]*\btest_report_print_stdout\b[\s\S]*\btest_report_exit_code\b/,
            `${expected.name} must separate report construction, stdout, and exit code`,
        );

        const assertionLines = doctest.stdout
            .split(/\r?\n/)
            .filter((line) => line.startsWith("assertion index="));
        assert.equal(
            assertionLines.length,
            expected.count,
            `${expected.name} stdout must fixture every assertion line`,
        );
    }
}

console.log("selfhost cli args doc report contract ok");
