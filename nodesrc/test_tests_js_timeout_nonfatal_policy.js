#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");
const { spawnSync } = require("node:child_process");

const repoRoot = path.resolve(__dirname, "..");
const testsJs = path.join(repoRoot, "nodesrc", "tests.js");
const tmpDir = path.join(repoRoot, "tmp", `tests-js-timeout-nonfatal-${process.pid}`);
fs.mkdirSync(tmpDir, { recursive: true });

function runTests(args, env) {
    return spawnSync(process.execPath, [testsJs, ...args], {
        cwd: repoRoot,
        encoding: "utf8",
        env: {
            ...process.env,
            ...env,
        },
    });
}

const timeoutStrictOut = path.join(tmpDir, "timeout-strict.json");
const timeoutStrict = runTests([
    "-i",
    "tests/stdlib/io.n.md",
    "--no-tree",
    "-o",
    timeoutStrictOut,
    "-j",
    "1",
], {
    NEPL_TEST_CASE_TIMEOUT_MS: "1",
});
assert.notEqual(timeoutStrict.status, 0, "timeout-only doctest errors must fail without --timeout-nonfatal");
const timeoutStrictJson = JSON.parse(fs.readFileSync(timeoutStrictOut, "utf8"));
assert.equal(timeoutStrictJson.summary.failed, 0);
assert.ok(timeoutStrictJson.summary.errored > 0, "strict timeout run must record errored cases");
assert.ok(timeoutStrictJson.summary.timed_out > 0, "strict timeout run must count timed-out cases");
assert.equal(timeoutStrictJson.summary.non_timeout_failed, 0);
assert.equal(timeoutStrictJson.summary.non_timeout_errored, 0);
assert.ok(timeoutStrictJson.results.some((r) => r.timeout), "strict timeout run must preserve timeout diagnostics");

const timeoutNonfatalOut = path.join(tmpDir, "timeout-nonfatal.json");
const timeoutNonfatal = runTests([
    "-i",
    "tests/stdlib/io.n.md",
    "--no-tree",
    "-o",
    timeoutNonfatalOut,
    "-j",
    "1",
    "--timeout-nonfatal",
], {
    NEPL_TEST_CASE_TIMEOUT_MS: "1",
});
assert.equal(
    timeoutNonfatal.status,
    0,
    `timeout-only doctest errors must not fail with --timeout-nonfatal\nstdout:\n${timeoutNonfatal.stdout}\nstderr:\n${timeoutNonfatal.stderr}`,
);
const timeoutNonfatalJson = JSON.parse(fs.readFileSync(timeoutNonfatalOut, "utf8"));
assert.equal(timeoutNonfatalJson.summary.failed, 0);
assert.ok(timeoutNonfatalJson.summary.errored > 0, "nonfatal timeout run must still record errored cases");
assert.ok(timeoutNonfatalJson.summary.timed_out > 0, "nonfatal timeout run must still count timed-out cases");
assert.equal(timeoutNonfatalJson.summary.non_timeout_failed, 0);
assert.equal(timeoutNonfatalJson.summary.non_timeout_errored, 0);
assert.ok(timeoutNonfatalJson.top_issues?.length > 0 || timeoutNonfatalJson.results.some((r) => r.timeout));

const badCasePath = path.join(tmpDir, "bad-case.n.md");
fs.writeFileSync(badCasePath, [
    "# bad case",
    "",
    "neplg2:test",
    "```neplg2",
    "#entry main",
    "#indent 4",
    "fn main %fn void i32 \\void:",
    "    missing_name",
    "```",
    "",
].join("\n"));

const badCaseOut = path.join(tmpDir, "bad-case.json");
const badCase = runTests([
    "-i",
    badCasePath,
    "--no-tree",
    "-o",
    badCaseOut,
    "-j",
    "1",
    "--timeout-nonfatal",
], {
    NEPL_TEST_CASE_TIMEOUT_MS: "60000",
});
assert.notEqual(badCase.status, 0, "non-timeout compile errors must still fail with --timeout-nonfatal");
const badCaseJson = JSON.parse(fs.readFileSync(badCaseOut, "utf8"));
assert.equal(badCaseJson.summary.timed_out, 0);
assert.ok(
    badCaseJson.summary.non_timeout_failed > 0 || badCaseJson.summary.non_timeout_errored > 0,
    "non-timeout compile error must be counted separately from timeout diagnostics",
);
assert.ok(
    badCaseJson.summary.failed > 0 || badCaseJson.summary.errored > 0,
    "non-timeout compile error must remain visible as failed or errored",
);

console.log("nodesrc tests.js timeout nonfatal policy passed");
