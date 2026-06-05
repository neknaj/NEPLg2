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

const storage = source("stdlib/alloc/collections/btreemap/storage.nepl");

for (const reportName of [
    "btreemap_key_at_doc",
    "btreemap_value_at_doc",
    "btreemap_store_key_doc",
    "btreemap_store_value_doc",
    "btreemap_key_slot_doc",
    "btreemap_value_slot_doc",
    "btreemap_store_key_slot_doc",
    "btreemap_store_value_slot_doc",
    "btreemap_free_storage_doc",
    "btreemap_alloc_storage_doc",
    "btreemap_copy_live_slots_doc",
    "btreemap_clear_slots_doc",
    "btreemap_shift_right_doc",
    "btreemap_shift_left_after_remove_doc",
    "btreemap_grow_doc",
]) {
    assertReport(storage, reportName);
}

for (const snippet of [
    "Vec Option .K",
    "Vec Option .V",
    "範囲外と empty slot は同じ戻り値",
    "Option::None",
    "diag_out_of_memory",
    "partial allocation cleanup",
    "先に確保した key storage を free",
    "BTreeMapInsertError",
    "元 map owner",
    "旧 storage owner を free",
    "storage invariant failure",
    "Copy",
    "O(cap)",
    "O(len0)",
]) {
    assertIncludes(storage, snippet, `BTreeMap storage docs must keep contract detail: ${snippet}`);
}

assert.match(
    storage,
    /Result::Err _e:\s*\n\s*vec::free keys\s*\n\s*diag_err diag_out_of_memory/u,
    "BTreeMap storage allocation docs and implementation must keep values-side failure cleanup visible",
);
assert.match(
    storage,
    /Result::Err d:\s*\n\s*Result::Err BTreeMapInsertError<\.K,\.V> \(BTreeMap<\.K,\.V> len0 cap0 storage\) d/u,
    "BTreeMap grow must return the consumed owner on allocation failure",
);
assert.doesNotMatch(
    storage,
    /neplg2:test(?!\[(?:stdio, normalize_newlines|compile_fail)\])/u,
    "BTreeMap storage doctests must use stdout report form or explicit compile_fail metadata",
);

console.log("stdlib btreemap storage doc report contract passed");
