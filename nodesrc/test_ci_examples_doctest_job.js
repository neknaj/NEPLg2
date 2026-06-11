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

const examplesJob = jobBlock("examples-test");
assertContains(examplesJob, "        needs: build", "examples-test must reuse bootstrap build artifacts");
assertContains(
    examplesJob,
    "            - name: Checkout repository\n              uses: actions/checkout@v4",
    "examples-test must checkout repository sources before running nodesrc/tests.js",
);
assertContains(
    examplesJob,
    '                  NEPL_TEST_CASE_TIMEOUT_MS: "60000"',
    "examples-test must keep enough per-case timeout headroom for current nm base compile",
);
assertContains(
    examplesJob,
    '        timeout-minutes: 15',
    "examples-test job timeout must leave room for the 10 minute command timeout wrapper to report",
);
assertContains(
    examplesJob,
    '              run: node nodesrc/ci_timeout.js --minutes 10 --label "examples doctests" --timeout-nonfatal -- node nodesrc/tests.js -i examples -o examples-tests.json -j 2 --timeout-nonfatal',
    "examples-test must detect timeouts without failing the Action on timeout-only doctest errors",
);
assertContains(examplesJob, "                  name: bootstrap-build", "examples-test must download bootstrap-build");
assertContains(examplesJob, "                  name: examples-tests", "examples-test must upload examples-tests artifact");
assertContains(examplesJob, "                  path: examples-tests.json", "examples-test artifact must contain the result JSON");

const pagesFinalBundle = jobBlock("pages-final-bundle");
assertContains(pagesFinalBundle, "            - examples-test", "pages-final-bundle must wait for examples-test");
assertContains(pagesFinalBundle, "                  name: examples-tests", "pages-final-bundle must download examples-tests artifact");
assertContains(pagesFinalBundle, "                  path: artifacts/examples", "examples artifact must be downloaded to a dedicated directory");
assertContains(
    pagesFinalBundle,
    "                  cp -f artifacts/examples/examples-tests.json dist/tests/examples-tests.json || true",
    "examples test JSON must be published into dist/tests",
);
assertContains(
    pagesFinalBundle,
    '                    "examples_test": "${{ needs[\'examples-test\'].result }}",',
    "final status summary must include examples-test result",
);

console.log("CI examples doctest job regression passed");
