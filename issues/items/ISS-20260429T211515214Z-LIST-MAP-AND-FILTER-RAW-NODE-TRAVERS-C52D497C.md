---
id: ISS-20260429T211515214Z-LIST-MAP-AND-FILTER-RAW-NODE-TRAVERS-C52D497C
title: "List map and filter raw-node traversal stalls ResourceIR verification"
area: stdlib
status: open
resolved: false
priority: P1
type: bug
created: 2026-04-29
updated: 2026-04-29
target: "stdlib/alloc/collections/list.nepl, stdlib/tests/list.n.md, tests/stdlib/list_collections.n.md"
---

# ISS-20260429T211515214Z-LIST-MAP-AND-FILTER-RAW-NODE-TRAVERS-C52D497C: List map and filter raw-node traversal stalls ResourceIR verification

## 概要

After List.reverse was changed to owner-preserving relink, List functional helper doctests that exercise map/filter still do not finish under the current ResourceIR verification. The remaining recursive map/filter implementation still walks raw node pointers and allocates replacement nodes through raw List pointer ownership.

## 対象

- `stdlib/alloc/collections/list.nepl, stdlib/tests/list.n.md, tests/stdlib/list_collections.n.md`

## 根拠

- `node nodesrc/run_doctest.js -i stdlib/tests/list.n.md -n 2 --dist web/dist` は 180 秒で timeout した。
- `node nodesrc/run_doctest.js -i tests/stdlib/list_collections.n.md -n 3 --dist web/dist` も 90 秒で timeout した。
- 同じ List file の reverse focused doctest は通過したため、残件は `reverse` 修正ではなく `map` / `filter` を通る raw-node traversal 側に分離できる。
- `list_map_impl` / `list_filter_impl` は `i32` raw node pointer を再帰的にたどり、tail を構築してから現在 node を `list_alloc_node` で作る設計のままで、ResourceIR が owner transfer を型構造として追跡しづらい。

## 問題

After List.reverse was changed to owner-preserving relink, List functional helper doctests that exercise map/filter still do not finish under the current ResourceIR verification. The remaining recursive map/filter implementation still walks raw node pointers and allocates replacement nodes through raw List pointer ownership.

## 影響

Full List/stdout doctest aggregation can time out after source policy passes, and self-host helper transformations still rely on raw-node ownership paths that are not yet tractable for the static checker.

## 修正方針

Redesign List map/filter ownership so traversal and result construction expose owner transfer explicitly, or migrate List storage away from raw pointer chains before relying on these helpers in self-host code.

## 検証

node nodesrc/run_doctest.js -i stdlib/tests/list.n.md -n 2 --dist web/dist currently times out and must pass after the redesign; node nodesrc/run_doctest.js -i tests/stdlib/list_collections.n.md -n 3 --dist web/dist must also pass.
