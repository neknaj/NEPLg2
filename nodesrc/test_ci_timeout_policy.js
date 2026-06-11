#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");
const { spawnSync } = require("node:child_process");

const repoRoot = path.resolve(__dirname, "..");
const tmpDir = path.join(repoRoot, "tmp", `ci-timeout-policy-${process.pid}`);
const workflow = fs.readFileSync(path.join(repoRoot, ".github", "workflows", "ci.yml"), "utf8").replace(/\r\n/g, "\n");
const wrapper = path.join(repoRoot, "nodesrc", "ci_timeout.js");
fs.mkdirSync(tmpDir, { recursive: true });

function jobBlock(name) {
    const startMarker = `\n    ${name}:\n`;
    const start = workflow.indexOf(startMarker);
    assert(start >= 0, `ci workflow must define ${name} job`);
    const bodyStart = start + startMarker.length;
    const nextJob = workflow.slice(bodyStart).search(/\n    [A-Za-z0-9_-]+:\n/);
    const end = nextJob >= 0 ? bodyStart + nextJob : workflow.length;
    return workflow.slice(start, end);
}

function runWrapper(args, env = {}) {
    return spawnSync(process.execPath, [wrapper, ...args], {
        cwd: repoRoot,
        encoding: "utf8",
        env: {
            ...process.env,
            ...env,
        },
    });
}

assert.doesNotMatch(workflow, /timeout --signal=KILL/, "CI must not use raw GNU timeout because it cannot report timeout as nonfatal");

for (const jobName of [
    "compile-test",
    "rust-test",
    "nm-compile",
    "wasi-test",
    "nmd-doctest",
    "tutorials-test",
    "examples-test",
    "stdlib-test",
    "llvm-test",
]) {
    const job = jobBlock(jobName);
    assert.match(job, /timeout-minutes:\s+15/, `${jobName} must give the 10 minute wrapper room to report a timeout`);
}

for (const line of workflow.split("\n").filter((l) => l.includes("node nodesrc/tests.js"))) {
    assert.match(line, /node nodesrc\/ci_timeout\.js --minutes \d+/, "doctest runner must be guarded by ci_timeout.js");
    assert.match(line, /--timeout-nonfatal/, "doctest runner must keep structured timeout-only errors nonfatal in CI");
}

const nonTimeoutFailure = runWrapper([
    "--timeout-ms",
    "10000",
    "--label",
    "non-timeout failure",
    "--",
    process.execPath,
    "-e",
    "process.exit(7)",
]);
assert.equal(nonTimeoutFailure.status, 7, "ci_timeout.js must preserve non-timeout command failures");

const timeoutSuccess = runWrapper([
    "--timeout-ms",
    "1",
    "--label",
    "timeout strict",
    "--",
    process.execPath,
    "-e",
    "setTimeout(() => {}, 10000)",
]);
assert.equal(timeoutSuccess.status, 124, "ci_timeout.js must fail on command timeouts by default");
assert.match(
    `${timeoutSuccess.stdout}\n${timeoutSuccess.stderr}`,
    /timeout strict timed out/,
    "ci_timeout.js must report detected command timeouts",
);

const timeoutNonfatal = runWrapper([
    "--timeout-ms",
    "1",
    "--label",
    "timeout nonfatal",
    "--timeout-marker",
    path.join(tmpDir, "timeout-success.json"),
    "--timeout-nonfatal",
    "--",
    process.execPath,
    "-e",
    "setTimeout(() => {}, 10000)",
]);
assert.equal(timeoutNonfatal.status, 0, "ci_timeout.js must make detected command timeouts nonfatal only when explicitly requested");
assert.match(
    `${timeoutNonfatal.stdout}\n${timeoutNonfatal.stderr}`,
    /timeout nonfatal timed out/,
    "ci_timeout.js must report detected command timeouts",
);
const timeoutMarker = JSON.parse(fs.readFileSync(path.join(tmpDir, "timeout-success.json"), "utf8"));
assert.equal(timeoutMarker.timed_out, true, "ci_timeout.js must write an explicit timeout marker when requested");
assert.equal(timeoutMarker.label, "timeout nonfatal");

console.log("CI timeout policy regression passed");
