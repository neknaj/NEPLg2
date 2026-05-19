#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");
const { parseFile } = require("./parser");

const repoRoot = path.resolve(__dirname, "..");
const roots = ["tests", "tutorials", "stdlib", "examples"];

function walk(dir, out = []) {
    if (!fs.existsSync(dir)) {
        return out;
    }
    for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
        const full = path.join(dir, entry.name);
        if (entry.isDirectory()) {
            if (entry.name === "node_modules" || entry.name === "target" || entry.name === "tmp") {
                continue;
            }
            walk(full, out);
        } else if (entry.isFile() && entry.name.endsWith(".n.md")) {
            out.push(full);
        }
    }
    return out;
}

function stripLineComments(code) {
    return code
        .split(/\r?\n/)
        .map((line) => {
            const index = line.indexOf("//");
            return index >= 0 ? line.slice(0, index) : line;
        })
        .join("\n");
}

function hasReportPrint(code) {
    return /\b(?:checks_print_report|test_report_print_stdout)\b/.test(code);
}

function hasReportExit(code) {
    return /\b(?:checks_exit_code|test_report_exit_code)\b/.test(code);
}

function reportMetadataViolations(rel, doctest, index) {
    if (doctest.tags.includes("compile_fail")) {
        return [];
    }

    const code = stripLineComments(doctest.code || "");
    const printsReport = hasReportPrint(code);
    const exitsFromReport = hasReportExit(code);
    if (!printsReport && !exitsFromReport) {
        return [];
    }

    const prefix = `${rel}::doctest#${index + 1}`;
    const violations = [];
    if (!printsReport) {
        violations.push(`${prefix}: report exit helper is used without printing the report`);
    }
    if (!exitsFromReport) {
        violations.push(`${prefix}: report print helper is used without deriving exit_code from the shown report`);
    }
    if (!doctest.tags.includes("stdio")) {
        violations.push(`${prefix}: report doctest must opt into stdio`);
    }
    if (!doctest.tags.includes("normalize_newlines")) {
        violations.push(`${prefix}: report doctest must normalize stdout newlines`);
    }
    if (doctest.stdout === null || doctest.stdout === "") {
        violations.push(`${prefix}: report doctest must pin stdout`);
    }
    if (doctest.exit_code === null || doctest.exit_code === undefined) {
        violations.push(`${prefix}: report doctest must pin exit_code`);
    }
    if (doctest.ret !== null && doctest.ret !== undefined) {
        violations.push(`${prefix}: report doctest must use exit_code, not ret`);
    }
    return violations;
}

{
    const good = {
        tags: ["stdio", "normalize_newlines"],
        code: "let shown test_report_print_stdout report\n    test_report_exit_code shown\n",
        stdout: "test_report name=\"sample\" count=0 failed=0\n",
        exit_code: 0,
        ret: null,
    };
    assert.deepEqual(reportMetadataViolations("sample.n.md", good, 0), []);

    const bad = {
        tags: [],
        code: "checks_exit_code checks\n",
        stdout: null,
        exit_code: null,
        ret: 0,
    };
    assert.deepEqual(reportMetadataViolations("sample.n.md", bad, 0), [
        "sample.n.md::doctest#1: report exit helper is used without printing the report",
        "sample.n.md::doctest#1: report doctest must opt into stdio",
        "sample.n.md::doctest#1: report doctest must normalize stdout newlines",
        "sample.n.md::doctest#1: report doctest must pin stdout",
        "sample.n.md::doctest#1: report doctest must pin exit_code",
        "sample.n.md::doctest#1: report doctest must use exit_code, not ret",
    ]);
}

const violations = [];
let reportDoctestCount = 0;
for (const root of roots) {
    for (const file of walk(path.join(repoRoot, root))) {
        const rel = path.relative(repoRoot, file);
        const parsed = parseFile(file);
        parsed.doctests.forEach((doctest, index) => {
            const hitCount = reportMetadataViolations(rel, doctest, index);
            if (hitCount.length > 0) {
                violations.push(...hitCount);
            }

            const code = stripLineComments(doctest.code || "");
            if (!doctest.tags.includes("compile_fail") && (hasReportPrint(code) || hasReportExit(code))) {
                reportDoctestCount += 1;
            }
        });
    }
}

assert(reportDoctestCount > 0, "n.md report metadata policy must scan active report doctests");
assert.deepEqual(
    violations,
    [],
    `.n.md report doctests must pin stdout, exit_code, and stdio metadata:\n${violations.join("\n")}`,
);

console.log("n.md report metadata policy passed");
