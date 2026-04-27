---
id: ISS-20260427T000312692Z-SPARSESET-RETAINS-UNSAFE-UNWRAPS-IN--D292C789
title: "SparseSet retains unsafe unwraps in dense/sparse internals"
area: stdlib
status: verified
resolved: true
priority: P2
type: bug
created: 2026-04-27
updated: 2026-04-27
target: "stdlib/alloc/collections/sparse_set.nepl, tests/stdlib/sparse_set_collections.n.md, nodesrc/test_stdlib_sparse_set_no_unsafe_unwraps.js"
---

# ISS-20260427T000312692Z-SPARSESET-RETAINS-UNSAFE-UNWRAPS-IN--D292C789: SparseSet retains unsafe unwraps in dense/sparse internals

## 概要

SparseSet header writes, initialization cleanup, and free paths use uwok on checked stores/deallocations for owned dense/sparse arrays.

## 対象

- `stdlib/alloc/collections/sparse_set.nepl, tests/stdlib/sparse_set_collections.n.md`

## 根拠

- `SparseSet.new` は 16 byte の header と、`n * 4` byte の `dense` / `sparse` 配列を確保し、`SparseSet` owner がそれらを単独所有する。
- header field write、dense/sparse slot write、途中確保失敗時の cleanup、`free` の cleanup はすべて owner invariant の内側の処理である。
- しかし実装は checked `store_i32` / `dealloc_ptr` / `dealloc` の `Result` を `uwok` で unwrap しており、collection 内部の保守処理が unsafe helper trap に依存していた。

## 問題

SparseSet header writes, initialization cleanup, and free paths use uwok on checked stores/deallocations for owned dense/sparse arrays.

## 影響

Self-host symbol/id sets can fail through unreachable traps and the collection remains outside the Queue/Deque unsafe-helper policy.

## 修正方針

Add raw owner-invariant header/slot stores, replace cleanup with dealloc_raw, preserve Result errors for allocation failure, and add source and behavior regressions.

## 解決内容

- `sparse_set_store_owned` と `sparse_set_hdr_store_i32` を追加し、dense/sparse slot と header field の raw owner write を集約した。
- `new` の初期化、`insert`、`remove`、`clear` の内部更新を owned helper 経由へ変更し、checked store unwrap を削除した。
- `new` の途中確保失敗 cleanup と `free` の dense/sparse/header cleanup を `dealloc_raw` に変更した。
- `clear` 後の `free` と再確保を確認する regression と、SparseSet 実装に unsafe unwrap / checked deallocation が戻らない source policy guard を追加した。

## 検証

- `node nodesrc/test_stdlib_sparse_set_no_unsafe_unwraps.js`: pass
- `node nodesrc/tests.js -i stdlib/alloc/collections/sparse_set.nepl --no-tree -o tmp/sparse-set-owned-cleanup-docs.json -j 1`: 7/7 passed
- `node nodesrc/tests.js -i tests/stdlib/sparse_set_collections.n.md -i stdlib/tests/sparse_set.n.md --no-tree -o tmp/sparse-set-owned-cleanup-focused.json -j 1`: 4/4 passed
- source policy regressions: pass
- `node nodesrc/tests.js -i tests/stdlib --no-tree -o tmp/tests-stdlib-sparse-set-owned-cleanup.json -j 4`: 294/294 passed
- `node nodesrc/tests.js -i stdlib --no-tree -o tmp/stdlib-sparse-set-owned-cleanup.json -j 4`: 418/418 passed
- `node nodesrc/issues.js check`: pass
