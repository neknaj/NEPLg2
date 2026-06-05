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

const root = source("stdlib/alloc/collections/bitset.nepl");
const types = source("stdlib/alloc/collections/bitset/types.nepl");
const layout = source("stdlib/alloc/collections/bitset/layout.nepl");
const storage = source("stdlib/alloc/collections/bitset/storage.nepl");
const mutation = source("stdlib/alloc/collections/bitset/mutation.nepl");
const api = source("stdlib/alloc/collections/bitset/api.nepl");
const diagnostic = source("stdlib/alloc/collections/bitset/api/diagnostic.nepl");

for (const [code, reportName] of [
    [root, "bitset_facade_lifecycle_doc"],
    [api, "bitset_api_facade_doc"],
    [types, "bitset_type_invariant_doc"],
    [types, "bitset_update_error_type_doc"],
    [types, "bitset_update_error_diag_doc"],
    [types, "bitset_update_error_owner_doc"],
    [layout, "bitset_byte_index_doc"],
    [layout, "bitset_mask_doc"],
    [layout, "bitset_valid_index_doc"],
    [layout, "bitset_byte_len_doc"],
    [storage, "bitset_byte_at_doc"],
    [storage, "bitset_store_byte_doc"],
    [storage, "bitset_fill_bytes_doc"],
    [storage, "bitset_alloc_bits_doc"],
    [mutation, "bitset_write_masked_doc"],
    [diagnostic, "bitset_invalid_len_diag_doc"],
    [diagnostic, "bitset_index_diag_doc"],
]) {
    assertIncludes(
        code,
        "neplg2:test[stdio, normalize_newlines]",
        `${reportName} must use stdio report doctest metadata`,
    );
    assertIncludes(code, `test_report_new "${reportName}"`, `${reportName} report doctest is missing`);
}

for (const snippet of [
    "### [契約/けいやく]",
    "### [現状/げんじょう]の実装",
    "時間計算量",
    "Option::None",
    "StdErrorKind::OutOfMemory",
    "BitSetUpdateError",
]) {
    assertIncludes(
        [types, layout, storage, mutation, diagnostic].join("\n"),
        snippet,
        `BitSet docs must keep contract detail: ${snippet}`,
    );
}

assert.doesNotMatch(
    types,
    /let\s+e\s+%BitSetUpdateError\s+BitSetUpdateError/,
    "BitSetUpdateError docs must obtain owner-backed errors through public update APIs, not direct construction",
);
assertIncludes(
    types,
    "match insert bs",
    "BitSetUpdateError accessor docs must exercise the public insert failure path",
);
assertIncludes(
    types,
    "bitset_update_error_owner e",
    "BitSetUpdateError docs must demonstrate owner recovery",
);
assertIncludes(
    diagnostic,
    'diag_std_error_kind_str d',
    "BitSet diagnostics docs must assert typed error kind instead of display text",
);

console.log("bitset doc report contract passed");
