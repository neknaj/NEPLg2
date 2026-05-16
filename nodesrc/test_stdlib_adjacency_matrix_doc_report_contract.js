#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");

const repoRoot = path.resolve(__dirname, "..");
const createPath = path.join(
    repoRoot,
    "stdlib",
    "alloc",
    "collections",
    "adjacency_matrix",
    "api",
    "create.nepl",
);
const source = fs.readFileSync(createPath, "utf8");

assert.match(
    source,
    /neplg2:test\[stdio,\s*normalize_newlines\]/,
    "AdjacencyMatrix.new doc-comment doctest must use stdout report tags",
);
assert.match(
    source,
    /\/\/:\s*exit_code:\s*0/,
    "AdjacencyMatrix.new doc-comment doctest must assert exit_code metadata",
);
assert.match(
    source,
    /test_report_new\s+"adjacency_matrix_new"/,
    "AdjacencyMatrix.new doc-comment doctest must emit a named TestReport",
);
assert.match(
    source,
    /test_report_push\s+assert_eq_i32\s+"matrix len"\s+5\s+size/,
    "AdjacencyMatrix.new doc-comment doctest must report the observed matrix length",
);
assert.match(
    source,
    /test_report_print_stdout\s+report[\s\S]*test_report_exit_code\s+shown/,
    "AdjacencyMatrix.new doc-comment doctest must separate stdout report from exit code",
);
assert.doesNotMatch(
    source,
    /\blet\s+ok\s+<bool>\s+eq\s+len\s+&g\s+5\b/,
    "AdjacencyMatrix.new doc-comment doctest must not return to stale eq assertion style",
);

console.log("adjacency matrix doc report contract passed");
