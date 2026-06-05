#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");
const { parseFile } = require("./parser");

const repoRoot = path.resolve(__dirname, "..");

function source(relPath) {
    return fs.readFileSync(path.join(repoRoot, relPath), "utf8").replace(/\r\n/g, "\n");
}

function assertIncludes(code, needle, message) {
    assert.ok(code.includes(needle), message);
}

function assertReport(code, reportName) {
    assertIncludes(
        code,
        "neplg2:test[stdio, normalize_newlines]",
        `${reportName} must use stdio report doctest metadata`,
    );
    assertIncludes(code, `test_report_new "${reportName}"`, `${reportName} report doctest is missing`);
}

const btreeMapSearch = source("stdlib/alloc/collections/btreemap/search.nepl");
const btreeSetSearch = source("stdlib/alloc/collections/btreeset/search.nepl");

for (const reportName of [
    "btreemap_key_eq_doc",
    "btreemap_lower_bound_doc",
    "btreemap_is_at_doc",
    "btreemap_lower_bound_storage_doc",
    "btreemap_is_at_storage_doc",
]) {
    assertReport(btreeMapSearch, reportName);
}

for (const snippet of [
    "low <= result <= high",
    "Option::None",
    "storage invariant failure",
    "idx >= len0",
    "O(log n)",
    "`storage` owner は[消費/しょうひ]しません",
]) {
    assertIncludes(btreeMapSearch, snippet, `BTreeMap search docs must keep contract detail: ${snippet}`);
}

const btreeSetFile = path.join(repoRoot, "stdlib", "alloc", "collections", "btreeset", "search.nepl");
const parsedSet = parseFile(btreeSetFile);
assert.equal(parsedSet.doctests.length, 1, "btreeset/search.nepl must keep the current key equality doctest until its slice is updated");

const setDoctest = parsedSet.doctests[0];
assert.equal(setDoctest.ret, null, "btreeset_key_eq_doc must not use ret-only success reporting");
assert.equal(setDoctest.exit_code, 0, "btreeset_key_eq_doc must pin exit_code: 0");
assert.match(
    setDoctest.stdout,
    /^test_report name="btreeset_key_eq_doc" count=2 failed=0\n/,
    "btreeset_key_eq_doc must pin canonical stdout report",
);
assert.match(setDoctest.code, /test_report_new "btreeset_key_eq_doc"/, "btreeset_key_eq_doc must construct a named TestReport");
assert.match(setDoctest.code, /btreeset_key_eq 7 7/, "btreeset_key_eq_doc must exercise equal keys");
assert.match(setDoctest.code, /btreeset_key_eq 7 9/, "btreeset_key_eq_doc must exercise unequal keys");
assert.doesNotMatch(setDoctest.code, /btreeset_key_eq<[^>]+>/, "btreeset_key_eq_doc must rely on argument evidence");
assert.match(setDoctest.code, /test_report_print_stdout report/, "btreeset_key_eq_doc must print the report");
assert.match(setDoctest.code, /test_report_exit_code shown/, "btreeset_key_eq_doc must derive exit code from the shown report");
assert.doesNotMatch(btreeSetSearch, /\bchecks_exit_code\b/, "btreeset/search.nepl must not hide report details behind checks_exit_code");
assert.doesNotMatch(btreeSetSearch, /\bchecks_print_report\b/, "btreeset/search.nepl must use canonical TestReport output");
assert.doesNotMatch(btreeSetSearch, /\bchecks_new\b/, "btreeset/search.nepl must not reintroduce legacy Checks construction");

console.log("stdlib btree search doc report contract passed");
