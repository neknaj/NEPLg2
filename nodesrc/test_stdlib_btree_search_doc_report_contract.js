#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");

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

const btreeMapSearchReports = [
    "btreemap_key_eq_doc",
    "btreemap_lower_bound_doc",
    "btreemap_is_at_doc",
    "btreemap_lower_bound_storage_doc",
    "btreemap_is_at_storage_doc",
];
const btreeSetSearchReports = [
    "btreeset_key_eq_doc",
    "btreeset_lower_bound_doc",
    "btreeset_is_at_doc",
    "btreeset_lower_bound_storage_doc",
    "btreeset_is_at_storage_doc",
];
const sharedSearchSnippets = [
    "low <= result <= high",
    "Option::None",
    "storage invariant failure",
    "idx >= len0",
    "O(log n)",
    "`storage` owner は[消費/しょうひ]しません",
];

for (const reportName of btreeMapSearchReports) {
    assertReport(btreeMapSearch, reportName);
}

for (const reportName of btreeSetSearchReports) {
    assertReport(btreeSetSearch, reportName);
}

for (const snippet of sharedSearchSnippets) {
    assertIncludes(btreeMapSearch, snippet, `BTreeMap search docs must keep contract detail: ${snippet}`);
    assertIncludes(btreeSetSearch, snippet, `BTreeSet search docs must keep contract detail: ${snippet}`);
}

assert.doesNotMatch(btreeSetSearch, /\bchecks_exit_code\b/, "btreeset/search.nepl must not hide report details behind checks_exit_code");
assert.doesNotMatch(btreeSetSearch, /\bchecks_print_report\b/, "btreeset/search.nepl must use canonical TestReport output");
assert.doesNotMatch(btreeSetSearch, /\bchecks_new\b/, "btreeset/search.nepl must not reintroduce legacy Checks construction");
assert.doesNotMatch(btreeMapSearch, /\bchecks_exit_code\b/, "btreemap/search.nepl must not hide report details behind checks_exit_code");
assert.doesNotMatch(btreeMapSearch, /\bchecks_print_report\b/, "btreemap/search.nepl must use canonical TestReport output");
assert.doesNotMatch(btreeMapSearch, /\bchecks_new\b/, "btreemap/search.nepl must not reintroduce legacy Checks construction");

console.log("stdlib btree search doc report contract passed");
