---
id: ISS-20260516T104604371Z-ADJACENCYMATRIX-CREATE-DOCTEST-STILL-9ACAECC9
title: "AdjacencyMatrix create doctest still uses stale eq assertion style"
area: stdlib
status: open
resolved: false
priority: P2
type: bug
created: 2026-05-16
updated: 2026-05-16
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
