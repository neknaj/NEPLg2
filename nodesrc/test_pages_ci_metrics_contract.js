#!/usr/bin/env node
"use strict";

const assert = require("node:assert");
const fs = require("node:fs");
const path = require("node:path");

const repoRoot = path.resolve(__dirname, "..");

function read(rel) {
    return fs.readFileSync(path.join(repoRoot, rel), "utf8").replace(/\r\n/g, "\n");
}

function assertIncludes(text, needle, message) {
    assert(text.includes(needle), `${message}\nexpected to find:\n${needle}`);
}

function assertNotIncludes(text, needle, message) {
    assert(!text.includes(needle), `${message}\nunexpected text:\n${needle}`);
}

const workflow = read(".github/workflows/ci.yml");
const index = read("web/index.html");
const tests = read("web/tests.html");
const testinfo = read("web/testinfo.html");
const metrics = read("web/metrics.html");
const runner = read("nodesrc/run_repo_metrics.js");

assertIncludes(index, '<link data-trunk rel="copy-file" href="metrics.html" />', "Trunk must publish metrics.html at the Pages root");
assertIncludes(index, '<link data-trunk rel="copy-file" href="tests.html" />', "Trunk must keep publishing tests.html");
assertIncludes(index, '<link data-trunk rel="copy-file" href="testinfo.html" />', "Trunk must keep publishing testinfo.html");
assertIncludes(index, 'href="./metrics.html"', "Playground header should link to the metrics page");

assertIncludes(workflow, "Build repository metrics into dist", "CI must generate repo metrics for Pages");
assertIncludes(workflow, "cp -a web/dist/. dist/", "CI must copy Trunk output into the Pages artifact root");
assertIncludes(workflow, "node nodesrc/run_repo_metrics.js --root . --json dist/metrics/repo_metrics.json --csv dist/metrics/repo_metrics.csv", "CI must run repo_metrics.ts through the wrapper");
assertIncludes(workflow, '"run_url": "${{ github.server_url }}/${{ github.repository }}/actions/runs/${{ github.run_id }}"', "status.json should expose the Actions run URL");
assertIncludes(workflow, '"head_sha": "${{ github.sha }}"', "status.json should expose the source revision");

assertIncludes(runner, "repo_metrics.ts", "wrapper must keep repo_metrics.ts as the metrics authority");
assertIncludes(runner, "--experimental-strip-types", "wrapper should use native TS execution when available");
assertIncludes(runner, "node_modules", "wrapper should fall back to the repo TypeScript toolchain");

assertIncludes(metrics, "./metrics/repo_metrics.json", "metrics.html must load the generated metrics JSON by default");
assertIncludes(metrics, "./metrics/repo_metrics.csv", "metrics.html must link the generated metrics CSV");

for (const html of [tests, testinfo]) {
    assertIncludes(html, "./tests/status.json", "CI result viewers must load the workflow status summary");
    assertIncludes(html, "./tests/tests-current.json", "CI result viewers must include wasi doctest JSON");
    assertIncludes(html, "./tests/nmd-tests.json", "CI result viewers must include nmd doctest JSON");
    assertIncludes(html, "./tests/tutorials-tests.json", "CI result viewers must include tutorials doctest JSON");
    assertIncludes(html, "./tests/examples-tests.json", "CI result viewers must include examples doctest JSON");
    assertIncludes(html, "./tests/stdlib-tests.json", "CI result viewers must include stdlib doctest JSON");
    assertIncludes(html, "./tests/tests-llvm.json", "CI result viewers must include LLVM doctest JSON");
    assertIncludes(html, "./tests/tests-dual-tests.json", "CI result viewers must include dual tests JSON");
    assertIncludes(html, "./tests/tests-dual-stdlib.json", "CI result viewers must include dual stdlib JSON");
    assertNotIncludes(html, 'value="./tests.json"', "CI result viewers must not default to stale tests.json");
    assertNotIncludes(html, 'value="./tests/cargo-test.log"', "CI result viewers must not depend on an unpublished cargo log");
}

console.log("Pages CI metrics contract regression passed");
