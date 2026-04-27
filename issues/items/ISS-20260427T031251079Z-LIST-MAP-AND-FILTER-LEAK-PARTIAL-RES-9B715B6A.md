---
id: ISS-20260427T031251079Z-LIST-MAP-AND-FILTER-LEAK-PARTIAL-RES-9B715B6A
title: "List map and filter leak partial results when final cons allocation fails"
area: stdlib
status: verified
resolved: true
priority: P2
type: bug
created: 2026-04-27
updated: 2026-04-27
target: "stdlib/alloc/collections/list.nepl, tests/stdlib/list_collections.n.md, nodesrc/test_stdlib_list_no_unsafe_unwraps.js"
---

# ISS-20260427T031251079Z-LIST-MAP-AND-FILTER-LEAK-PARTIAL-RES-9B715B6A: List map and filter leak partial results when final cons allocation fails

## 概要

list_map_impl and list_filter_impl recursively allocate the tail first and then call cons for the current head. If that final cons allocation fails, the already-built mapped or filtered tail is dropped from the Result path without cleanup.

## 対象

- `stdlib/alloc/collections/list.nepl, tests/stdlib/list_collections.n.md`

## 根拠

- `list_map_impl` / `list_filter_impl` は、まず tail 側を再帰的に構築し、その後で現在ノードを `cons` して元の順序を保っていた。
- tail 側の構築が成功した後の最後の `cons` はノード確保を行うため `Err(Diag)` になり得る。
- この `cons` 失敗では、すでに構築済みの `mapped_tail` / `filtered_tail` owner を呼び出し側へ返せず、cleanup もされないため、allocation pressure 時に部分 list が leak する。

## 問題

list_map_impl and list_filter_impl recursively allocate the tail first and then call cons for the current head. If that final cons allocation fails, the already-built mapped or filtered tail is dropped from the Result path without cleanup.

## 影響

Allocation pressure in self-host list transformations can leak nodes and leave callers with only an Err, making repeated parser/helper transforms progressively exhaust memory.

## 修正方針

When cons fails after a tail list has been built, free the partial tail before returning Err, and add a source regression covering the cleanup path.

## 解決内容

- `list_map_impl` は `cons` に `mapped_tail` owner を渡さず、`mapped_tail_ptr` と `list_alloc_node` で現在ノードを確保するようにした。
- `list_filter_impl` も同様に `filtered_tail_ptr` と `list_alloc_node` を使い、現在ノードを残す場合の最後の allocation failure を明示的に扱うようにした。
- 最後のノード確保に失敗した場合は、構築済みの `mapped_tail` / `filtered_tail` を `free` してから `Err(Diag)` を返すようにした。
- `nodesrc/test_stdlib_list_no_unsafe_unwraps.js` を拡張し、map/filter の partial tail cleanup 分岐を source policy として固定した。

## 検証

- `node nodesrc/test_stdlib_list_no_unsafe_unwraps.js`: pass
- `node nodesrc/tests.js -i stdlib/alloc/collections/list.nepl --no-tree -o tmp/list-map-filter-docs.json -j 1`: 15/15 passed
- `node nodesrc/tests.js -i tests/stdlib/list_collections.n.md -i stdlib/tests/list.n.md --no-tree -o tmp/list-map-filter-focused.json -j 1`: 5/5 passed
- source policy regressions: pass
- `node nodesrc/tests.js -i tests/stdlib --no-tree -o tmp/tests-stdlib-list-map-filter.json -j 4`: 301/301 passed
- `node nodesrc/tests.js -i stdlib --no-tree -o tmp/stdlib-list-map-filter.json -j 4`: 418/418 passed
- `node nodesrc/issues.js check`: pass
