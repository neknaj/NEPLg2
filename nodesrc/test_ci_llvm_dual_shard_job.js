#!/usr/bin/env node
"use strict";

const assert = require("assert");
const fs = require("fs");
const path = require("path");

const repoRoot = path.resolve(__dirname, "..");
const workflowPath = path.join(repoRoot, ".github", "workflows", "ci.yml");
const workflow = fs.readFileSync(workflowPath, "utf8").replace(/\r\n/g, "\n");

function jobBlock(name) {
    const startMarker = `\n    ${name}:\n`;
    const start = workflow.indexOf(startMarker);
    assert(start >= 0, `ci workflow must define ${name} job`);
    const bodyStart = start + startMarker.length;
    const nextJob = workflow.slice(bodyStart).search(/\n    [A-Za-z0-9_-]+:\n/);
    const end = nextJob >= 0 ? bodyStart + nextJob : workflow.length;
    return workflow.slice(start, end);
}

function assertContains(haystack, needle, message) {
    assert(
        haystack.includes(needle),
        `${message}\nexpected to find:\n${needle}`,
    );
}

const llvmDualJob = jobBlock("llvm-dual-test");
for (const suite of ["tests", "stdlib"]) {
    for (const shard of [1, 2, 3, 4, 5, 6, 7, 8]) {
        const id = `${suite}-shard-${shard}`;
        assertContains(llvmDualJob, `                    - id: ${id}`, `llvm-dual-test must include ${id}`);
        assertContains(llvmDualJob, `                      shard: "${shard}/8"`, `${id} must pass a stable shard spec`);
        assertContains(llvmDualJob, `                      output: tests-dual-${suite}-shard-${shard}.json`, `${id} must write a unique JSON output`);
    }
}
assertContains(
    llvmDualJob,
    "              run: timeout --signal=KILL 18m node nodesrc/tests.js ${{ matrix.inputs }} -o ${{ matrix.output }} --runner all --llvm-all --assert-io --strict-dual --shard ${{ matrix.shard }} ${{ matrix.tree_flag }} -j 2",
    "llvm-dual-test must run each shard through nodesrc/tests.js",
);
assertContains(llvmDualJob, "                  name: llvm-dual-${{ matrix.id }}", "llvm-dual artifacts must be unique per shard");

const pagesFinalBundle = jobBlock("pages-final-bundle");
assertContains(
    pagesFinalBundle,
    "            - name: Checkout repository\n              uses: actions/checkout@v4",
    "pages-final-bundle must checkout repository sources before running merge scripts",
);
assertContains(
    pagesFinalBundle,
    "node nodesrc/merge_doctest_json.js -o dist/tests/tests-dual-tests.json artifacts/llvm-dual/tests-dual-tests-shard-*.json",
    "pages-final-bundle must merge tests shards back to the canonical dual JSON",
);
assertContains(
    pagesFinalBundle,
    "node nodesrc/merge_doctest_json.js -o dist/tests/tests-dual-stdlib.json artifacts/llvm-dual/tests-dual-stdlib-shard-*.json",
    "pages-final-bundle must merge stdlib shards back to the canonical dual JSON",
);

console.log("CI LLVM dual shard job regression passed");
