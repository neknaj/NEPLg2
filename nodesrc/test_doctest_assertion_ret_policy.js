#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");
const { parseFile } = require("./parser");

const repoRoot = path.resolve(__dirname, "..");
const roots = ["tests", "tutorials", "stdlib", "examples"];

function walkFiles(dir, out = []) {
    if (!fs.existsSync(dir)) {
        return out;
    }
    for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
        const full = path.join(dir, entry.name);
        if (entry.isDirectory()) {
            if (entry.name === "node_modules" || entry.name === "target" || entry.name === "tmp") {
                continue;
            }
            walkFiles(full, out);
        } else if (entry.isFile() && (entry.name.endsWith(".n.md") || entry.name.endsWith(".nepl"))) {
            out.push(full);
        }
    }
    return out;
}

function stripLineComments(code) {
    return String(code || "")
        .split(/\r?\n/)
        .map((line) => {
            const index = line.indexOf("//");
            return index >= 0 ? line.slice(0, index) : line;
        })
        .join("\n");
}

function importsStdTest(code) {
    return /#import\s+"std\/test"(?:\s+as\b|\s*$)/m.test(code);
}

function usesStdTestReportApi(code) {
    return /\b(?:checks_|test_report_)\w*\b/.test(code);
}

function isCoreOnlyAssertionExample(code) {
    return /#target\s+core\b/.test(code) && /#import\s+"core\/test"(?:\s+as\b|\s*$)/m.test(code);
}

function assertionRetViolations() {
    const violations = [];
    let retCount = 0;
    let coreOnlyAssertionRetCount = 0;

    for (const root of roots) {
        for (const file of walkFiles(path.join(repoRoot, root))) {
            const rel = path.relative(repoRoot, file);
            const parsed = parseFile(file);
            parsed.doctests.forEach((doctest, index) => {
                if (doctest.tags.includes("compile_fail") || doctest.tags.includes("skip")) {
                    return;
                }
                if (doctest.ret === null || doctest.ret === undefined) {
                    return;
                }

                retCount += 1;
                const code = stripLineComments(doctest.code || "");
                if (isCoreOnlyAssertionExample(code)) {
                    coreOnlyAssertionRetCount += 1;
                    return;
                }
                if (importsStdTest(code) || usesStdTestReportApi(code)) {
                    violations.push(
                        `${rel}::doctest#${index + 1}: std/test assertion doctest must use stdout + exit_code, not ret`,
                    );
                }
            });
        }
    }

    return { violations, retCount, coreOnlyAssertionRetCount };
}

const { violations, retCount, coreOnlyAssertionRetCount } = assertionRetViolations();

assert(retCount > 0, "assertion ret policy must scan return-value doctests");
assert(
    coreOnlyAssertionRetCount > 0,
    "assertion ret policy must keep an explicit core/test ret-only allowance visible",
);
assert.deepEqual(
    violations,
    [],
    `std/test assertion doctests must not use ret-only metadata:\n${violations.join("\n")}`,
);

console.log("doctest assertion ret policy passed");
