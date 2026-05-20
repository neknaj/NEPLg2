---
id: ISS-20260520T113031972Z-VEC-DATA-VIEW-COLLAPSES-INVALID-INVA-D6378EBA
title: "Vec data view collapses invalid invariant into null MemPtr"
area: stdlib
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-20
updated: 2026-05-20
target: "stdlib/alloc/collections/vec/access/data.nepl, stdlib/alloc/collections/vec callers, nodesrc/test_stdlib_vec_no_unsafe_unwraps.js"
---

# ISS-20260520T113031972Z-VEC-DATA-VIEW-COLLAPSES-INVALID-INVA-D6378EBA: Vec data view collapses invalid invariant into null MemPtr

## 概要

Vec.data_mem_ptr returns MemPtr<T> directly, so malformed Vec invariant and valid empty storage both become mem_ptr_wrap 0. This hides typed invalid evidence and lets raw-boundary callers treat an invalid owner aggregate as the same shape as an empty Vec.

## 対象

- `stdlib/alloc/collections/vec/access/data.nepl, stdlib/alloc/collections/vec callers, nodesrc/test_stdlib_vec_no_unsafe_unwraps.js`

## 根拠

- `data_mem_ptr<T>(&Vec<T>) -> MemPtr<T>` は戻り値型に valid empty storage と invalid owner aggregate の違いを表せなかった。
- 直前の修正で `VecCopyInvariant::Invalid(VecCopyInvariantInvalid)` を導入したにもかかわらず、`data_mem_ptr` は invalid reason を `mem_ptr_wrap 0` へ潰していた。
- `VecStorage::Empty` も同じ `mem_ptr_wrap 0` を返すため、caller は `match` の網羅性で valid empty / invalid / actual data を区別できなかった。

## 問題

Vec.data_mem_ptr returns MemPtr<T> directly, so malformed Vec invariant and valid empty storage both become mem_ptr_wrap 0. This hides typed invalid evidence and lets raw-boundary callers treat an invalid owner aggregate as the same shape as an empty Vec.

## 影響

Stage 6 static checking requires proof/refutation evidence to remain visible through enum matches. Collapsing invalid Vec state into a pointer sentinel weakens the design and makes future ResourceIR or self-host code likely to miss malformed storage states.

## 修正方針

Replace the direct MemPtr observer with a typed VecDataView<T> enum: Empty, Data(MemPtr<T>), Invalid(VecCopyInvariantInvalid). Update all Vec raw-boundary call sites and doctests to exhaustively match the enum; do not keep a compatibility data_mem_ptr alias.

## 検証

Add source policy coverage for VecDataView and for removal of data_mem_ptr, run Vec source policy, borrowed observer policy, Vec doctests, memory safety/sort regressions that referenced the old observer, and issues check.

## 2026-05-20 Agent 1 修正

`data_mem_ptr<T>(&Vec<T>) -> MemPtr<T>` を互換 alias なしで削除し、`data_mem_view<T>(&Vec<T>) -> VecDataView<T>` へ置き換えた。

`VecDataView<T>` は `Empty | Data(MemPtr<T>) | Invalid(VecCopyInvariantInvalid)` の enum である。これにより valid empty storage、actual backing data view、malformed `OwnedBuffer<T>` が `match` の網羅性で分岐され、invalid reason が pointer sentinel に潰れない。

`get` / `replace` / `pop` / transform family / sort family / merge sort は `VecDataView::Data` branch の中だけで `mem_ptr_addr` / `load` / `store` へ進むよう更新した。`VecDataView::Empty` と `Invalid(reason)` は raw traversal に入らない。

関連設計:

- [NEPLg2 静的検査の複雑化解消計画](https://github.com/neknaj/NEPLg2/blob/main/doc/neplg2/static_check_complexity_reduction_plan.md)
- [stdlib collection/mem/string と静的検査の安全設計](https://github.com/neknaj/NEPLg2/blob/main/doc/neplg2/stdlib_collection_mem_string_static_safety_design.md)

focused verification:

- `node nodesrc/test_stdlib_vec_no_unsafe_unwraps.js`: passed
- `node nodesrc/test_stdlib_vec_borrowed_observers.js`: passed
- `node nodesrc/test_selfhost_cli_args_no_owner_field_reads.js`: passed
- `node nodesrc/test_stdlib_diag_error_no_unsafe_unwraps.js`: passed
- `node nodesrc/tests.js -i stdlib/alloc/collections/vec/access/data.nepl --no-tree --dist web/dist -o tmp/agent1-vec-data-view-access.json -j 1 --assert-io`: 2/2 passed
- `node nodesrc/tests.js -i stdlib/alloc/collections/vec/query/get.nepl --no-tree --dist web/dist -o tmp/agent1-vec-data-view-get.json -j 1 --assert-io`: 1/1 passed
- `node nodesrc/tests.js -i stdlib/alloc/collections/vec/mutation/pop.nepl --no-tree --dist web/dist -o tmp/agent1-vec-data-view-pop.json -j 1 --assert-io`: 3/3 passed
- `node nodesrc/tests.js -i stdlib/alloc/collections/vec/transform/map.nepl --no-tree --dist web/dist -o tmp/agent1-vec-data-view-map.json -j 1 --assert-io`: 1/1 passed
- `node nodesrc/tests.js -i stdlib/alloc/collections/vec/transform/filter.nepl --no-tree --dist web/dist -o tmp/agent1-vec-data-view-filter.json -j 1 --assert-io`: 7/7 passed
- `node nodesrc/tests.js -i stdlib/alloc/collections/vec/transform/prefix.nepl --no-tree --dist web/dist -o tmp/agent1-vec-data-view-prefix.json -j 1 --assert-io`: 2/2 passed
- `node nodesrc/tests.js -i stdlib/alloc/collections/vec/sort.nepl --no-tree --dist web/dist -o tmp/agent1-vec-data-view-sort.json -j 1 --assert-io`: 3/3 passed
- `node nodesrc/tests.js -i tests/stdlib/memory_safety.n.md --no-tree --dist web/dist -o tmp/agent1-vec-data-view-memory-safety.json -j 1 --assert-io`: 63/63 passed
- `node nodesrc/tests.js -i tests/stdlib/sort.n.md --no-tree --dist web/dist -o tmp/agent1-vec-data-view-sort-fixture.json -j 1 --assert-io`: 20/20 passed
- `node nodesrc/tests.js -i stdlib/alloc/collections/vec.nepl --no-tree --dist web/dist -o tmp/agent1-vec-data-view-root.json -j 1 --assert-io`: 3/3 passed
- `node nodesrc/issues.js check`: passed
- `git diff --check`: passed
- final focused rerun:
  - `node nodesrc/tests.js -i stdlib/alloc/collections/vec/access/data.nepl --no-tree --dist web/dist -o tmp/agent1-vec-data-view-access-final.json -j 1 --assert-io`: 2/2 passed
  - `node nodesrc/tests.js -i tests/stdlib/memory_safety.n.md --no-tree --dist web/dist -o tmp/agent1-vec-data-view-memory-safety-final.json -j 1 --assert-io`: 63/63 passed
  - `node nodesrc/tests.js -i stdlib/alloc/collections/vec/sort.nepl --no-tree --dist web/dist -o tmp/agent1-vec-data-view-sort-final.json -j 1 --assert-io`: 3/3 passed
  - `node nodesrc/tests.js -i tests/stdlib/sort.n.md --no-tree --dist web/dist -o tmp/agent1-vec-data-view-sort-fixture-final.json -j 1 --assert-io`: 20/20 passed
  - `node nodesrc/tests.js -i stdlib/alloc/collections/vec/transform/filter.nepl --no-tree --dist web/dist -o tmp/agent1-vec-data-view-filter-final.json -j 1 --assert-io`: 7/7 passed
