---
id: ISS-20260516T104604371Z-ADJACENCYMATRIX-CREATE-DOCTEST-STILL-9ACAECC9
title: "AdjacencyMatrix create doctest still uses stale eq assertion style"
area: stdlib
status: fixed
resolved: true
priority: P2
type: bug
created: 2026-05-16
updated: 2026-05-17
target: stdlib/alloc/collections/adjacency_matrix/api/create.nepl
---

# ISS-20260516T104604371Z-ADJACENCYMATRIX-CREATE-DOCTEST-STILL-9ACAECC9: AdjacencyMatrix create doctest still uses stale eq assertion style

## 概要

After compiler source-capability fixes, adjacency_matrix/api/create.nepl::doctest#1 no longer fails at owner aggregate or owner token boundaries, but the doctest still compiles `let ok <bool> eq len &g 5` where `eq` is undefined in the current stdlib imports.

## 対象

- `stdlib/alloc/collections/adjacency_matrix/api/create.nepl`

## 根拠

- `node nodesrc/run_doctest.js -i stdlib/alloc/collections/adjacency_matrix/api/create.nepl -n 1 --dist web/dist` は、compiler boundary errors が解消した後に `/virtual/entry.nepl:8:19` の `resolve.identifier.undefined` for `eq` で失敗した。
- 同じ run では `type.owner_aggregate.constructor_restricted` と `type.owner_token.field_access_restricted` は出ていないため、これは compiler capability ではなく doctest fixture の現行 stdlib API 追従漏れである。

## 問題

After compiler source-capability fixes, adjacency_matrix/api/create.nepl::doctest#1 no longer fails at owner aggregate or owner token boundaries, but the doctest still compiles `let ok <bool> eq len &g 5` where `eq` is undefined in the current stdlib imports.

## 影響

The public documentation test for AdjacencyMatrix construction remains red after the compiler correctness issues are fixed, hiding whether the API example demonstrates the current assertion/report style.

## 修正方針

Rewrite the doctest to current std/test or explicit comparison style with stdout/exit_code metadata, without weakening the compiler checks or granting extra source capabilities.

## 検証

Run the focused adjacency_matrix create doctest and adjacency_matrix doctest suite after compiler-priority work allows stdlib doctest cleanup.

## 対応

- `AdjacencyMatrix.new` の doctest を stale な `eq` / bool return 形式から、現行の `std/test` `TestReport` stdout + `exit_code: 0` 形式へ更新した。
- `unwrap_ok` で構築結果を明示的に取り出し、`len &g` の結果を `assert_eq_i32 "matrix len" 5 size` で表示する典型例にした。
- `free g` を report 出力前に実行し、doctest が collection owner を明示的に閉じる例になっていることを維持した。
- `nodesrc/test_stdlib_adjacency_matrix_doc_report_contract.js` を追加し、stdout metadata、`test_report_new`、`assert_eq_i32`、`test_report_print_stdout` / `test_report_exit_code` の契約と stale `eq` 再導入禁止を固定した。

## 検証結果

- `node nodesrc/test_stdlib_adjacency_matrix_doc_report_contract.js`
- `node nodesrc/test_stdlib_adjacency_matrix_no_unsafe_unwraps.js`
- `node nodesrc/run_doctest.js -i stdlib/alloc/collections/adjacency_matrix/api/create.nepl -n 1 --dist web/dist`
- `node nodesrc/tests.js -i stdlib/alloc/collections/adjacency_matrix/api/create.nepl --no-tree -o tmp/agent1-adjacency-create-doc-final.json -j 1 --dist web/dist --assert-io`
- `node nodesrc/run_source_policy_regressions.js`
