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

const root = source("stdlib/alloc/collections/adjacency_matrix.nepl");
const types = source("stdlib/alloc/collections/adjacency_matrix/types.nepl");
const layout = source("stdlib/alloc/collections/adjacency_matrix/layout.nepl");
const storage = source("stdlib/alloc/collections/adjacency_matrix/storage.nepl");
const mutation = source("stdlib/alloc/collections/adjacency_matrix/mutation.nepl");
const api = source("stdlib/alloc/collections/adjacency_matrix/api.nepl");
const create = source("stdlib/alloc/collections/adjacency_matrix/api/create.nepl");
const observer = source("stdlib/alloc/collections/adjacency_matrix/api/observer.nepl");
const update = source("stdlib/alloc/collections/adjacency_matrix/api/update.nepl");
const bulk = source("stdlib/alloc/collections/adjacency_matrix/api/bulk.nepl");
const cleanup = source("stdlib/alloc/collections/adjacency_matrix/api/cleanup.nepl");
const diagnostic = source("stdlib/alloc/collections/adjacency_matrix/api/diagnostic.nepl");

for (const [code, reportName] of [
    [root, "adjacency_matrix_facade_lifecycle_doc"],
    [api, "adjacency_matrix_api_facade_doc"],
    [types, "adjacency_matrix_type_invariant_doc"],
    [types, "adjacency_matrix_update_error_type_doc"],
    [types, "adjacency_matrix_update_error_diag_doc"],
    [types, "adjacency_matrix_update_error_owner_doc"],
    [layout, "adjacency_matrix_bit_index_doc"],
    [layout, "adjacency_matrix_byte_index_doc"],
    [layout, "adjacency_matrix_mask_doc"],
    [layout, "adjacency_matrix_valid_vertex_doc"],
    [layout, "adjacency_matrix_valid_edge_doc"],
    [layout, "adjacency_matrix_byte_len_doc"],
    [storage, "adjacency_matrix_byte_at_doc"],
    [storage, "adjacency_matrix_store_byte_doc"],
    [storage, "adjacency_matrix_fill_bytes_doc"],
    [storage, "adjacency_matrix_alloc_bits_doc"],
    [mutation, "adjacency_matrix_write_masked_doc"],
    [diagnostic, "adjacency_matrix_invalid_len_diag_doc"],
    [diagnostic, "adjacency_matrix_vertex_diag_doc"],
    [create, "adjacency_matrix_new"],
    [observer, "adjacency_matrix_len_doc"],
    [observer, "adjacency_matrix_contains_doc"],
    [update, "adjacency_matrix_update_doc"],
    [update, "adjacency_matrix_insert_doc"],
    [update, "adjacency_matrix_remove_doc"],
    [bulk, "adjacency_matrix_fill_value_doc"],
    [bulk, "adjacency_matrix_clear_doc"],
    [cleanup, "adjacency_matrix_free_doc"],
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
    "時間計算量",
    "Option::None",
    "StdErrorKind::OutOfMemory",
    "StdErrorKind::IndexOutOfBounds",
    "AdjacencyMatrixUpdateError",
    "row-major",
    "nbytes",
]) {
    assertIncludes(
        [types, layout, storage, mutation, create, observer, update, bulk, cleanup, diagnostic].join("\n"),
        snippet,
        `AdjacencyMatrix docs must keep contract detail: ${snippet}`,
    );
}

assert.doesNotMatch(
    types,
    /let\s+e\s+%AdjacencyMatrixUpdateError\s+AdjacencyMatrixUpdateError/,
    "AdjacencyMatrixUpdateError docs must obtain owner-backed errors through public update APIs, not direct construction",
);
assertIncludes(
    types,
    "match insert g",
    "AdjacencyMatrixUpdateError accessor docs must exercise the public insert failure path",
);
assertIncludes(
    types,
    "match remove g",
    "AdjacencyMatrixUpdateError type docs must exercise the public remove failure path",
);
assertIncludes(
    types,
    "adjacency_matrix_update_error_owner e",
    "AdjacencyMatrixUpdateError docs must demonstrate owner recovery",
);
assertIncludes(
    diagnostic,
    "diag_std_error_kind_str d",
    "AdjacencyMatrix diagnostics docs must assert typed error kind instead of display text",
);
assertIncludes(
    observer,
    "Result::Err d",
    "AdjacencyMatrix contains docs must demonstrate the typed error branch",
);

console.log("adjacency matrix doc report contract passed");
