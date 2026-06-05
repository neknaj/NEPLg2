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

const root = source("stdlib/alloc/collections/bloom_filter.nepl");
const types = source("stdlib/alloc/collections/bloom_filter/types.nepl");
const hash = source("stdlib/alloc/collections/bloom_filter/hash.nepl");
const layout = source("stdlib/alloc/collections/bloom_filter/layout.nepl");
const storage = source("stdlib/alloc/collections/bloom_filter/storage.nepl");
const mutation = source("stdlib/alloc/collections/bloom_filter/mutation.nepl");
const api = source("stdlib/alloc/collections/bloom_filter/api.nepl");

for (const [code, reportName] of [
    [root, "bloom_filter_facade_lifecycle_doc"],
    [types, "bloom_filter_type_invariant_doc"],
    [hash, "bloom_filter_hash0_doc"],
    [hash, "bloom_filter_hash1_doc"],
    [hash, "bloom_filter_probe_index_doc"],
    [layout, "bloom_filter_byte_len_doc"],
    [layout, "bloom_filter_byte_index_doc"],
    [layout, "bloom_filter_bit_mask_doc"],
    [storage, "bloom_filter_byte_at_doc"],
    [storage, "bloom_filter_store_byte_doc"],
    [storage, "bloom_filter_fill_bytes_doc"],
    [storage, "bloom_filter_alloc_bits_doc"],
    [storage, "bloom_filter_free_bits_doc"],
    [mutation, "bloom_filter_set_bit_doc"],
    [mutation, "bloom_filter_test_bit_doc"],
    [api, "bloom_filter_api_facade_doc"],
    [api, "bloom_filter_invalid_len_diag_doc"],
    [api, "bloom_filter_new_doc"],
    [api, "bloom_filter_len_doc"],
    [api, "bloom_filter_insert_doc"],
    [api, "bloom_filter_contains_doc"],
    [api, "bloom_filter_clear_doc"],
    [api, "bloom_filter_free_doc"],
]) {
    assertReport(code, reportName);
}

for (const snippet of [
    "nbits > 0",
    "Vec u8",
    "false positive",
    "false negative",
    "owner",
]) {
    assertIncludes(types, snippet, `BloomFilter type docs must keep invariant detail: ${snippet}`);
}

for (const snippet of [
    "StdErrorKind::CapacityExceeded",
    "typed error kind",
    "metadata-only observer",
    "nbits <= 0",
    "false positive",
    "false negative",
    "固定長 key では O(1)",
    "`str` では文字列長に比例",
]) {
    assertIncludes(api, snippet, `BloomFilter API docs must keep static-checking contract: ${snippet}`);
}

for (const snippet of [
    "Option::None",
    "typed `Vec u8`",
    "diag_out_of_memory",
    "O(nbytes)",
    "invalid index を error として返しません",
]) {
    assertIncludes(storage, snippet, `BloomFilter storage docs must keep typed storage contract: ${snippet}`);
}

for (const snippet of [
    "missing byte",
    "no-op",
    "false",
    "valid probe index",
]) {
    assertIncludes(mutation, snippet, `BloomFilter mutation docs must keep fail-closed contract: ${snippet}`);
}

for (const snippet of [
    "mixed hash が 0",
    "nbits > 0",
    "rem_u",
    "Hasher` / `HashKey",
    "固定長 key では O(1)",
]) {
    assertIncludes(hash, snippet, `BloomFilter hash docs must keep probe contract: ${snippet}`);
}

for (const snippet of [
    "ceil nbits / 8",
    "範囲検査は行いません",
    "1 <<",
]) {
    assertIncludes(layout, snippet, `BloomFilter layout docs must keep index precondition contract: ${snippet}`);
}

assert.doesNotMatch(
    [root, types, hash, layout, storage, mutation, api].join("\n"),
    /neplg2:test(?!\[stdio, normalize_newlines\])/u,
    "BloomFilter doc tests must stay in stdout report form",
);

console.log("bloom filter doc report contract passed");
