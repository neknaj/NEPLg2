#!/usr/bin/env node
"use strict";

const assert = require("assert");
const fs = require("fs");
const path = require("path");
const { spawnSync } = require("child_process");

const repoRoot = path.resolve(__dirname, "..");
const tmpDir = path.join(repoRoot, "tmp", `merge-doctest-json-${process.pid}`);
fs.mkdirSync(tmpDir, { recursive: true });

function writeReport(name, report) {
    const p = path.join(tmpDir, name);
    fs.writeFileSync(p, JSON.stringify(report, null, 2));
    return p;
}

const shard1 = writeReport("shard-1.json", {
    schema: "neplg2-doctest/v1",
    partial: false,
    resolved_dist_dirs: ["dist-a"],
    scan: { shard: { spec: "1/2", index: 1, total: 2, cases_before: 2, cases_after: 1 } },
    summary: { total: 1, passed: 1, failed: 0, errored: 0 },
    results: [
        { id: "b::doctest#2", file: "b", index: 2, status: "pass" },
    ],
});
const shard2 = writeReport("shard-2.json", {
    schema: "neplg2-doctest/v1",
    partial: true,
    resolved_dist_dirs: ["dist-b", "dist-a"],
    scan: { shard: { spec: "2/2", index: 2, total: 2, cases_before: 2, cases_after: 1 } },
    summary: { total: 2, passed: 0, failed: 1, errored: 1, timed_out: 1 },
    results: [
        { id: "a::doctest#1", file: "a", index: 1, status: "fail" },
        { id: "a::doctest#2", file: "a", index: 2, status: "error", timeout: { after_ms: 1 } },
    ],
});
const outPath = path.join(tmpDir, "merged.json");
const result = spawnSync(process.execPath, [
    path.join(repoRoot, "nodesrc", "merge_doctest_json.js"),
    "-o",
    outPath,
    shard1,
    shard2,
], {
    cwd: repoRoot,
    encoding: "utf8",
});
assert.equal(result.status, 0, `merge_doctest_json.js failed\nstdout:\n${result.stdout}\nstderr:\n${result.stderr}`);
const merged = JSON.parse(fs.readFileSync(outPath, "utf8"));
assert.equal(merged.merged, true);
assert.equal(merged.partial, true);
assert.deepEqual(merged.summary, {
    total: 3,
    passed: 1,
    failed: 1,
    errored: 1,
    timed_out: 1,
    timeout_failed: 0,
    timeout_errored: 1,
    non_timeout_failed: 1,
    non_timeout_errored: 0,
});
assert.deepEqual(merged.resolved_dist_dirs, ["dist-a", "dist-b"]);
assert.deepEqual(merged.results.map((r) => r.id), ["a::doctest#1", "a::doctest#2", "b::doctest#2"]);
assert.equal(merged.shards.length, 2);

console.log("merge doctest JSON regression passed");
