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
const layoutPath = path.join(
    repoRoot,
    "stdlib",
    "alloc",
    "collections",
    "adjacency_matrix",
    "layout.nepl",
);
const source = fs.readFileSync(createPath, "utf8");
const layoutSource = fs.readFileSync(layoutPath, "utf8");

function docBlockForFunction(sourceText, fnName) {
    const fnMarker = `pub fn ${fnName}`;
    const fnIndex = sourceText.indexOf(fnMarker);
    assert.notEqual(fnIndex, -1, `${fnName} must exist`);
    const docMarker = `//: ${fnName}:`;
    const docIndex = sourceText.lastIndexOf(docMarker, fnIndex);
    assert.notEqual(docIndex, -1, `${fnName} must have a doc-comment header`);
    return sourceText.slice(docIndex, fnIndex);
}

function assertLayoutDocSections(fnName) {
    const block = docBlockForFunction(layoutSource, fnName);
    assert.match(block, /\/\/:\s+### \[目的\/もくてき\]/, `${fnName} doc must state its purpose`);
    assert.match(block, /\/\/:\s+### \[契約\/けいやく\]/, `${fnName} doc must state its stable contract`);
    assert.match(
        block,
        /\/\/:\s+### \[現状実装\/げんじょうじっそう\]/,
        `${fnName} doc must separate current implementation details from contract`,
    );
    assert.match(block, /\/\/:\s+### \[計算量\/けいさんりょう\]/, `${fnName} doc must state complexity`);
    assert.match(block, /\/\/:\s+neplg2:test\[stdio,\s*normalize_newlines\]/, `${fnName} doc must include a report doctest`);
    assert.match(block, /test_report_new\s+"/, `${fnName} doctest must emit a named TestReport`);
}

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

[
    "adjacency_matrix_bit_index",
    "adjacency_matrix_byte_index",
    "adjacency_matrix_mask",
    "adjacency_matrix_valid_vertex",
    "adjacency_matrix_valid_edge",
    "adjacency_matrix_byte_len",
].forEach(assertLayoutDocSections);

console.log("adjacency matrix doc report contract passed");
