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

const storage = source("stdlib/alloc/collections/btreeset/storage.nepl");

for (const reportName of [
    "btreeset_key_at_doc",
    "btreeset_store_key_doc",
    "btreeset_key_slot_doc",
    "btreeset_store_key_slot_doc",
    "btreeset_free_storage_doc",
    "btreeset_alloc_storage_doc",
    "btreeset_copy_live_slots_doc",
    "btreeset_clear_slots_doc",
    "btreeset_shift_right_doc",
    "btreeset_shift_left_after_remove_doc",
    "btreeset_grow_doc",
]) {
    assertReport(storage, reportName);
}

for (const snippet of [
    "Vec Option .T",
    "Option::Some key",
    "Option::None",
    "範囲外と empty slot は同じ戻り値",
    "diag_out_of_memory",
    "BTreeSetInsertError",
    "btreeset_insert_error_owner",
    "元 set owner",
    "旧 storage owner を free",
    "old last slot",
    "storage invariant failure",
    "Copy",
    "O(cap)",
    "O(len0)",
]) {
    assertIncludes(storage, snippet, `BTreeSet storage docs must keep contract detail: ${snippet}`);
}

assert.doesNotMatch(
    storage,
    /Vec Option \.V|value storage|partial allocation cleanup|\.V: Copy/u,
    "BTreeSet storage docs must not copy BTreeMap value-storage-only contracts",
);
assert.match(
    storage,
    /Result::Err d:\s*\n\s*Result::Err BTreeSetInsertError<\.T> \(BTreeSet<\.T> len0 cap0 storage\) d/u,
    "BTreeSet grow must return the consumed owner on allocation failure",
);
assert.match(
    storage,
    /Result::Ok next_storage:\s*\n\s*btreeset_copy_live_slots<\.T> &storage &next_storage len0\s*\n\s*btreeset_free_storage<\.T> storage/u,
    "BTreeSet grow success must copy live keys and free the old storage owner",
);
assert.doesNotMatch(
    storage,
    /neplg2:test(?!\[(?:stdio, normalize_newlines|compile_fail)\])/u,
    "BTreeSet storage doctests must use stdout report form or explicit compile_fail metadata",
);

console.log("stdlib btreeset storage doc report contract passed");
