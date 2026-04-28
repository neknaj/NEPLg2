---
id: ISS-20260428T031445330Z-DISJOINT-SET-AND-SEGMENT-TREE-RAW-OW-D2F6ED1A
title: "disjoint_set and segment_tree raw owner detours fail under strict move checking"
area: stdlib
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-04-28
updated: 2026-04-28
target: "stdlib/alloc/collections/disjoint_set.nepl, stdlib/alloc/collections/segment_tree.nepl, stdlib/tests/disjoint_set.n.md, stdlib/tests/segment_tree.n.md"
---

# ISS-20260428T031445330Z-DISJOINT-SET-AND-SEGMENT-TREE-RAW-OW-D2F6ED1A: disjoint_set and segment_tree raw owner detours fail under strict move checking

## 概要

Latest strict move checking rejects disjoint_set and segment_tree functions that store the owning struct in raw memory only to read Copy fields and later deallocate the scratch cell. stdlib/tests/disjoint_set.n.md and stdlib/tests/segment_tree.n.md fail with D3100 deallocating raw memory place containing non-Copy value: mem.

## 対象

- `stdlib/alloc/collections/disjoint_set.nepl, stdlib/alloc/collections/segment_tree.nepl, stdlib/tests/disjoint_set.n.md, stdlib/tests/segment_tree.n.md`

## 根拠

- `node nodesrc/tests.js -i stdlib/tests --no-tree -o tmp/stdlib-tests-static-followup-20260428.json -j 1` で `stdlib/tests/disjoint_set.n.md::doctest#1/#2` と `stdlib/tests/segment_tree.n.md::doctest#1/#2` が D3100 になった。
- 失敗は `deallocating raw memory place containing non-Copy value: mem` で、`DisjointSet` / `SegmentTree` owner struct を scratch raw cell に `store` してから Copy field を `load` する実装に由来していた。

## 問題

Latest strict move checking rejects disjoint_set and segment_tree functions that store the owning struct in raw memory only to read Copy fields and later deallocate the scratch cell. stdlib/tests/disjoint_set.n.md and stdlib/tests/segment_tree.n.md fail with D3100 deallocating raw memory place containing non-Copy value: mem.

## 影響

These basic data structures are self-host prerequisites. The current implementation uses raw aggregate detours as a compiler workaround, so it no longer follows the intended borrowed-field style and prevents clean stdlib collection verification.

## 修正方針

Remove the scratch raw aggregate cells from disjoint_set and segment_tree. Read Copy fields through field::get_ref on the owning value, keep the owner value live for return/free, and leave only the actual i32 backing arrays in raw memory.

## 検証

Run node nodesrc/tests.js -i stdlib/tests/disjoint_set.n.md -i stdlib/tests/segment_tree.n.md --no-tree -o tmp/stdlib-dsu-segtree-after-fix.json -j 1, run the broader stdlib/tests shard if feasible, and node nodesrc/issues.js check.

## 解決

- `DisjointSet` / `SegmentTree` owner struct を `alloc_raw size_of<T>` の scratch cell に退避する処理を削除した。
- Copy field は `field::get_ref` で借用読み取りし、owner value をそのまま success path の戻り値または `free` に渡すようにした。
- read-only query である `find` / `same` / `size` / `sum_range` は `&DisjointSet` / `&SegmentTree` を受け取る signature に変更し、owner を移動しない API にした。
- `union` / `replace` / `add` の invalid input path は owner を消費する signature のままなので、`Err` を返す前に `free` して backing array を残さないようにした。
- doctest と focused collection tests の呼び出し側を参照渡しに更新した。

## 検証結果

- `node nodesrc/tests.js -i stdlib/tests/disjoint_set.n.md --no-tree -o tmp/dsu-static-followup-stdlib.json -j 1`: 2/2 passed
- `node nodesrc/tests.js -i stdlib/tests/segment_tree.n.md --no-tree -o tmp/segtree-static-followup-stdlib.json -j 1`: 2/2 passed
- `node nodesrc/tests.js -i tests/stdlib/disjoint_set_collections.n.md --no-tree -o tmp/dsu-static-followup-tests.json -j 1`: 3/3 passed
- `node nodesrc/tests.js -i tests/stdlib/segment_tree_collections.n.md --no-tree -o tmp/segtree-static-followup-tests.json -j 1`: 2/2 passed
- `node nodesrc/tests.js -i stdlib/alloc/collections/disjoint_set.nepl -i stdlib/alloc/collections/segment_tree.nepl --no-tree -o tmp/dsu-segtree-static-followup-docs.json -j 1`: 11/11 passed
- remote main `412a69c` 取り込み後:
  - `trunk build`: pass
  - `node nodesrc/tests.js -i stdlib/tests/disjoint_set.n.md -i stdlib/tests/segment_tree.n.md --no-tree -o tmp/stdlib-dsu-segtree-after-rebase.json -j 1`: 4/4 passed
  - `node nodesrc/tests.js -i tests/stdlib/disjoint_set_collections.n.md -i tests/stdlib/segment_tree_collections.n.md --no-tree -o tmp/tests-dsu-segtree-after-rebase.json -j 1`: 5/5 passed
  - `node nodesrc/tests.js -i stdlib/alloc/collections/disjoint_set.nepl -i stdlib/alloc/collections/segment_tree.nepl --no-tree -o tmp/dsu-segtree-docs-after-rebase.json -j 1`: 11/11 passed
  - `node nodesrc/tests.js -i stdlib/tests --no-tree -o tmp/stdlib-tests-static-followup-after-dsu-segtree-20260428.json -j 1`: 80 件中 75 件 passed、残り 5 件は `ISS-20260428T031445156Z-STDLIB-DIAG-AND-ERROR-RAW-AGGREGATE--D64EF00F` の diag/error D3100。
- remote main `492bcce` 取り込み後:
  - `trunk build`: pass
  - `node nodesrc/tests.js -i stdlib/tests/disjoint_set.n.md -i stdlib/tests/segment_tree.n.md --no-tree -o tmp/stdlib-dsu-segtree-final.json -j 1`: 4/4 passed
  - `node nodesrc/tests.js -i tests/stdlib/disjoint_set_collections.n.md -i tests/stdlib/segment_tree_collections.n.md --no-tree -o tmp/tests-dsu-segtree-final.json -j 1`: 5/5 passed
  - `node nodesrc/tests.js -i stdlib/alloc/collections/disjoint_set.nepl -i stdlib/alloc/collections/segment_tree.nepl --no-tree -o tmp/dsu-segtree-docs-final.json -j 1`: 11/11 passed
- remote main `6127d99` 取り込み後:
  - `trunk build`: pass
  - `node nodesrc/tests.js -i stdlib/tests/disjoint_set.n.md -i stdlib/tests/segment_tree.n.md --no-tree -o tmp/stdlib-dsu-segtree-post-binding-rules.json -j 1`: 4/4 passed
  - `node nodesrc/tests.js -i tests/stdlib/disjoint_set_collections.n.md -i tests/stdlib/segment_tree_collections.n.md --no-tree -o tmp/tests-dsu-segtree-post-binding-rules.json -j 1`: 5/5 passed
  - `node nodesrc/tests.js -i stdlib/alloc/collections/disjoint_set.nepl -i stdlib/alloc/collections/segment_tree.nepl --no-tree -o tmp/dsu-segtree-docs-post-binding-rules.json -j 1`: 11/11 passed
- remote main `cef5f6e` 取り込み後:
  - `trunk build`: pass
  - `node nodesrc/tests.js -i stdlib/tests/disjoint_set.n.md -i stdlib/tests/segment_tree.n.md --no-tree -o tmp/stdlib-dsu-segtree-post-ascription.json -j 1`: 4/4 passed
  - `node nodesrc/tests.js -i tests/stdlib/disjoint_set_collections.n.md -i tests/stdlib/segment_tree_collections.n.md --no-tree -o tmp/tests-dsu-segtree-post-ascription.json -j 1`: 5/5 passed
  - `node nodesrc/issues.js check`: pass
