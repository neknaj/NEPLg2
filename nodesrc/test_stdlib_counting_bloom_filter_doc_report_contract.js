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

const root = source("stdlib/alloc/collections/counting_bloom_filter.nepl");
const types = source("stdlib/alloc/collections/counting_bloom_filter/types.nepl");
const hash = source("stdlib/alloc/collections/counting_bloom_filter/hash.nepl");
const storage = source("stdlib/alloc/collections/counting_bloom_filter/storage.nepl");
const mutation = source("stdlib/alloc/collections/counting_bloom_filter/mutation.nepl");
const api = source("stdlib/alloc/collections/counting_bloom_filter/api.nepl");

for (const [code, reportName] of [
    [root, "counting_bloom_filter_facade_lifecycle_doc"],
    [types, "counting_bloom_filter_type_invariant_doc"],
    [hash, "counting_bloom_filter_hash0_doc"],
    [hash, "counting_bloom_filter_hash1_doc"],
    [hash, "counting_bloom_filter_probe_index_doc"],
    [storage, "counting_bloom_filter_counter_at_doc"],
    [storage, "counting_bloom_filter_store_counter_doc"],
    [storage, "counting_bloom_filter_fill_counters_doc"],
    [storage, "counting_bloom_filter_alloc_counters_doc"],
    [storage, "counting_bloom_filter_free_counters_doc"],
    [mutation, "counting_bloom_filter_counter_inc_doc"],
    [mutation, "counting_bloom_filter_counter_dec_doc"],
    [mutation, "counting_bloom_filter_counter_nonzero_doc"],
    [api, "counting_bloom_filter_api_facade_doc"],
    [api, "counting_bloom_filter_invalid_len_diag_doc"],
    [api, "counting_bloom_filter_new_doc"],
    [api, "counting_bloom_filter_len_doc"],
    [api, "counting_bloom_filter_insert_doc"],
    [api, "counting_bloom_filter_remove_doc"],
    [api, "counting_bloom_filter_contains_doc"],
    [api, "counting_bloom_filter_clear_doc"],
    [api, "counting_bloom_filter_free_doc"],
]) {
    assertReport(code, reportName);
}

for (const snippet of [
    "nslots > 0",
    "Vec u8",
    "0..255",
    "飽和",
    "false negative",
    "owner",
]) {
    assertIncludes(types, snippet, `CountingBloomFilter type docs must keep invariant detail: ${snippet}`);
}

for (const snippet of [
    "StdErrorKind::CapacityExceeded",
    "typed error kind",
    "metadata-only observer",
    "nslots <= 0",
    "false positive",
    "false negative",
    "saturation",
    "未挿入 key",
    "固定長 key では O(1)",
    "`str` では文字列長に比例",
]) {
    assertIncludes(api, snippet, `CountingBloomFilter API docs must keep static-checking contract: ${snippet}`);
}

for (const snippet of [
    "Option::None",
    "typed `Vec u8`",
    "diag_out_of_memory",
    "O(nslots)",
    "invalid index を error として返しません",
]) {
    assertIncludes(storage, snippet, `CountingBloomFilter storage docs must keep typed storage contract: ${snippet}`);
}

for (const snippet of [
    "255",
    "0",
    "missing counter",
    "no-op",
    "false",
    "fail-closed",
]) {
    assertIncludes(mutation, snippet, `CountingBloomFilter mutation docs must keep counter boundary contract: ${snippet}`);
}

for (const snippet of [
    "mixed hash が 0",
    "nslots > 0",
    "rem_u",
    "Hasher` / `HashKey",
    "固定長 key では O(1)",
]) {
    assertIncludes(hash, snippet, `CountingBloomFilter hash docs must keep probe contract: ${snippet}`);
}

assert.doesNotMatch(
    [root, types, hash, storage, mutation, api].join("\n"),
    /neplg2:test(?!\[stdio, normalize_newlines\])/u,
    "CountingBloomFilter doc tests must stay in stdout report form",
);

console.log("counting bloom filter doc report contract passed");
