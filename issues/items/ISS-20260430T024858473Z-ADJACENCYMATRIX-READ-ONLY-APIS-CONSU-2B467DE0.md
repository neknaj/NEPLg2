---
id: ISS-20260430T024858473Z-ADJACENCYMATRIX-READ-ONLY-APIS-CONSU-2B467DE0
title: "AdjacencyMatrix read-only APIs consume owners by value instead of borrowing"
area: stdlib
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-04-30
updated: 2026-04-30
target: stdlib/alloc/collections/adjacency_matrix.nepl
---

# ISS-20260430T024858473Z-ADJACENCYMATRIX-READ-ONLY-APIS-CONSU-2B467DE0: AdjacencyMatrix read-only APIs consume owners by value instead of borrowing

## 概要

AdjacencyMatrix len/contains take AdjacencyMatrix by value even though they only read the graph. Callers that inspect an edge or vertex count lose the owner and cannot free it afterwards, so strict owner checking forces either leaks or field-level workarounds.

## 対象

- `stdlib/alloc/collections/adjacency_matrix.nepl`

## 根拠

- `stdlib/alloc/collections/adjacency_matrix.nepl` に `fn len <(AdjacencyMatrix)->i32>` と `fn contains <(AdjacencyMatrix,i32,i32)*>Result<bool, Diag>>` が残っている。
- BitSet の owner-consuming observer 修正中に sibling collection を確認し、同じ raw bit storage owner を持つ AdjacencyMatrix でも読み取り API が値 receiver のままだと判明した。

## 問題

AdjacencyMatrix len/contains take AdjacencyMatrix by value even though they only read the graph. Callers that inspect an edge or vertex count lose the owner and cannot free it afterwards, so strict owner checking forces either leaks or field-level workarounds.

## 影響

Self-host graph/set style code cannot use AdjacencyMatrix observers as normal public APIs under mandatory memory-safety checking. The API shape encourages duplicated values or omitted frees instead of checked borrowing.

## 修正方針

Redesign AdjacencyMatrix len/contains around &AdjacencyMatrix, update doctests and tests to borrow for observation, then explicitly free the same owner. Remove by-value observer entry points rather than keeping compatibility wrappers.

## 検証

Add compile/run tests that call len and contains through &AdjacencyMatrix multiple times, then free the same matrix without move-check or owner-check diagnostics.

確認済み:

- `node nodesrc/tests.js -i stdlib/tests/adjacency_matrix.n.md --no-tree -o tmp/adjacency-stdlib-borrowed-observers.json -j 1` (`total=2`, `passed=2`, `failed=0`)
- `node nodesrc/tests.js -i tests/stdlib/adjacency_matrix_collections.n.md --no-tree -o tmp/adjacency-collections-borrowed-observers.json -j 1` (`total=2`, `passed=2`, `failed=0`)
- `node nodesrc/tests.js -i stdlib/alloc/collections/adjacency_matrix.nepl --no-tree -o tmp/adjacency-doctest-borrowed-observers.json -j 1` (`total=6`, `passed=6`, `failed=0`)
- `node nodesrc/test_stdlib_adjacency_matrix_borrowed_observers.js`: passed
- `node nodesrc/test_stdlib_adjacency_matrix_no_unsafe_unwraps.js`: passed
- `node nodesrc/issues.js check`: passed

## 修正内容

- `AdjacencyMatrix.len` と `AdjacencyMatrix.contains` を `&AdjacencyMatrix` receiver に変更し、読み取りで owner を移動しない公開 API にした。
- AdjacencyMatrix doctest / `.n.md` tests を、1 つの `AdjacencyMatrix` に対して複数回 borrowed observer を呼び、その後 `free` する形に直した。
- `nodesrc/test_stdlib_adjacency_matrix_borrowed_observers.js` を追加し、by-value observer signature と by-value test usage が戻らないよう source policy に登録した。
