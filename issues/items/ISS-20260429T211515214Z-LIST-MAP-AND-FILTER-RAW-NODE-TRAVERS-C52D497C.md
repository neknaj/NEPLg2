---
id: ISS-20260429T211515214Z-LIST-MAP-AND-FILTER-RAW-NODE-TRAVERS-C52D497C
title: "List map and filter raw-node traversal stalls ResourceIR verification"
area: stdlib
status: verified
resolved: true
priority: P1
type: bug
created: 2026-04-29
updated: 2026-04-30
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

## 対応

- `map` / `filter` の内部実装を raw node pointer recursive traversal から、入力 `List` owner を消費する forward traversal に変更した。
- 結果構築は `List` owner accumulator へ先頭追加し、最後に `reverse` で元の順序へ戻す方式にした。
- accumulator 追加用の `list_cons_owned` を追加し、node 確保失敗時は受け取った tail owner を関数内で解放する契約にした。
- ResourceIR が `failed` と `acc` owner state の相関を使わなくても検査できるよう、失敗側でも `free acc` を明示した。実行時には failure path で `acc` は空に戻しているため no-op になる。
- 現行 `List` には payload drop traversal がまだないため、`map` / `filter` の public API は `.T: Copy` / `.U: Copy` に限定した。
- `nodesrc/test_stdlib_list_no_unsafe_unwraps.js` を更新し、recursive raw-node rebuild への退行、unsafe unwrap、raw accumulator pointer への退行を防ぐ source policy を追加した。

## 検証

node nodesrc/run_doctest.js -i stdlib/tests/list.n.md -n 2 --dist web/dist currently times out and must pass after the redesign; node nodesrc/run_doctest.js -i tests/stdlib/list_collections.n.md -n 3 --dist web/dist must also pass.

- `node nodesrc/test_stdlib_list_no_unsafe_unwraps.js`: passed
- `node nodesrc/tests.js -i stdlib/tests/list.n.md --no-tree -o tmp/list-functional-after-owned-acc.json -j 1 --dist web/dist`: `total=2`, `passed=2`
- `node nodesrc/tests.js -i tests/stdlib/list_collections.n.md --no-tree -o tmp/list-collections-after-owned-acc.json -j 1 --dist web/dist`: `total=3`, `passed=3`
