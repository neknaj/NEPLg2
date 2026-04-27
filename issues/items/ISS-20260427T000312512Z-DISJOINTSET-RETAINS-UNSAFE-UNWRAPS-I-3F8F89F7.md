---
id: ISS-20260427T000312512Z-DISJOINTSET-RETAINS-UNSAFE-UNWRAPS-I-3F8F89F7
title: "DisjointSet retains unsafe unwraps in owned array internals"
area: stdlib
status: verified
resolved: true
priority: P2
type: bug
created: 2026-04-27
updated: 2026-04-27
target: "stdlib/alloc/collections/disjoint_set.nepl, tests/stdlib/disjoint_set_collections.n.md, nodesrc/test_stdlib_disjoint_set_no_unsafe_unwraps.js"
---

# ISS-20260427T000312512Z-DISJOINTSET-RETAINS-UNSAFE-UNWRAPS-I-3F8F89F7: DisjointSet retains unsafe unwraps in owned array internals

## 概要

DisjointSet uses uwok for parent/size array stores and cleanup paths even though those arrays are owned by the collection.

## 対象

- `stdlib/alloc/collections/disjoint_set.nepl, tests/stdlib/disjoint_set_collections.n.md`

## 根拠

- `DisjointSet.new` は `parent` と `sizes` の 2 本の `i32` 配列を確保し、`DisjointSet` owner がその配列を単独所有する。
- 初期化時の `parent[i]` / `sizes[i]` store、`union` の root/size 更新、`sizes` 確保失敗時の `parent` cleanup、`free` の `parent` / `sizes` cleanup はすべて owner invariant の内側にある。
- しかし実装は checked `store_i32` / `dealloc_ptr` の `Result` を `uwok` で unwrap しており、内部保守コードが public helper trap に依存していた。

## 問題

DisjointSet uses uwok for parent/size array stores and cleanup paths even though those arrays are owned by the collection.

## 影響

Union-find support for self-host graph algorithms can trap in internal maintenance code rather than exposing allocation failures through Result.

## 修正方針

Replace checked store/dealloc unwraps with raw owner-invariant helpers and dealloc_raw, add union/free regression coverage, and add a source guard.

## 解決内容

- `dsu_slot_ptr` / `dsu_load_owned` / `dsu_store_owned` を追加し、`parent` と `sizes` の owned array access を同じ helper に集約した。
- `new` の初期化と `union` の内部更新を owned helper 経由へ変更し、checked `store_i32` unwrap を削除した。
- `sizes` allocation failure cleanup と `free` の `parent` / `sizes` cleanup を `dealloc_raw` に変更した。
- union 後の `free` と再確保を確認する regression と、DisjointSet 実装に unsafe unwrap / checked deallocation が戻らない source policy guard を追加した。

## 検証

- `node nodesrc/test_stdlib_disjoint_set_no_unsafe_unwraps.js`: pass
- `node nodesrc/tests.js -i stdlib/alloc/collections/disjoint_set.nepl --no-tree -o tmp/disjoint-set-owned-cleanup-docs.json -j 1`: 6/6 passed
- `node nodesrc/tests.js -i tests/stdlib/disjoint_set_collections.n.md -i stdlib/tests/disjoint_set.n.md --no-tree -o tmp/disjoint-set-owned-cleanup-focused.json -j 1`: 4/4 passed
- source policy regressions: pass
- `node nodesrc/tests.js -i tests/stdlib --no-tree -o tmp/tests-stdlib-disjoint-set-owned-cleanup.json -j 4`: 293/293 passed
- `node nodesrc/tests.js -i stdlib --no-tree -o tmp/stdlib-disjoint-set-owned-cleanup.json -j 4`: 418/418 passed
- `node nodesrc/issues.js check`: pass
