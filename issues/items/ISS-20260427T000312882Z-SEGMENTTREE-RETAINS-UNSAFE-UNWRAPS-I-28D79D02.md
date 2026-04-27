---
id: ISS-20260427T000312882Z-SEGMENTTREE-RETAINS-UNSAFE-UNWRAPS-I-28D79D02
title: "SegmentTree retains unsafe unwraps in owned tree storage"
area: stdlib
status: verified
resolved: true
priority: P2
type: bug
created: 2026-04-27
updated: 2026-04-27
target: "stdlib/alloc/collections/segment_tree.nepl, tests/stdlib/segment_tree_collections.n.md, nodesrc/test_stdlib_segment_tree_no_unsafe_unwraps.js"
---

# ISS-20260427T000312882Z-SEGMENTTREE-RETAINS-UNSAFE-UNWRAPS-I-28D79D02: SegmentTree retains unsafe unwraps in owned tree storage

## 概要

SegmentTree uses uwok for owned array stores and free cleanup.

## 対象

- `stdlib/alloc/collections/segment_tree.nepl, tests/stdlib/segment_tree_collections.n.md`

## 根拠

- `SegmentTree.new` は `2 * base` 個の `i32` tree storage を確保し、`SegmentTree` owner がその配列を単独所有する。
- `new` の初期化、`replace` / `add` の leaf/internal node 更新、`sum_range` の読み取り、`free` の cleanup は owner invariant の内側にある。
- しかし実装は checked `store_i32` と `dealloc_ptr` の `Result` を `uwok` で unwrap しており、range-query helper の内部保守処理が unsafe helper trap に依存していた。

## 問題

SegmentTree uses uwok for owned array stores and free cleanup.

## 影響

Range-query helpers for self-host analysis can still trap in normal internal storage paths and weaken collection consistency.

## 修正方針

Introduce raw owned-array store helper, replace owned cleanup with dealloc_raw, add update/query/free regression coverage, and add a source guard.

## 解決内容

- `seg_load_owned` / `seg_store_owned` を追加し、owned tree storage の load/store を内部 helper に集約した。
- `new`、`replace`、`add`、`sum_range` の内部配列 access を owned helper 経由へ変更し、checked store unwrap を削除した。
- `free` の owned tree storage cleanup を `dealloc_raw` に変更した。
- update 後の `free` と `new 0` の cleanup、その後の再確保/query を確認する regression と、SegmentTree 実装に unsafe unwrap / checked deallocation が戻らない source policy guard を追加した。

## 検証

- `node nodesrc/test_stdlib_segment_tree_no_unsafe_unwraps.js`: pass
- `node nodesrc/tests.js -i stdlib/alloc/collections/segment_tree.nepl --no-tree -o tmp/segment-tree-owned-cleanup-docs.json -j 1`: 5/5 passed
- `node nodesrc/tests.js -i tests/stdlib/segment_tree_collections.n.md -i stdlib/tests/segment_tree.n.md --no-tree -o tmp/segment-tree-owned-cleanup-focused.json -j 1`: 4/4 passed
- source policy regressions: pass
- `node nodesrc/tests.js -i tests/stdlib --no-tree -o tmp/tests-stdlib-segment-tree-owned-cleanup.json -j 4`: 295/295 passed
- `node nodesrc/tests.js -i stdlib --no-tree -o tmp/stdlib-segment-tree-owned-cleanup.json -j 4`: 418/418 passed
- `node nodesrc/issues.js check`: pass
