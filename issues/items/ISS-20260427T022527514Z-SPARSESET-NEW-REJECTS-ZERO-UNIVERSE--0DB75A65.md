---
id: ISS-20260427T022527514Z-SPARSESET-NEW-REJECTS-ZERO-UNIVERSE--0DB75A65
title: "SparseSet new rejects zero universe length despite documented non-negative domain"
area: stdlib
status: verified
resolved: true
priority: P2
type: bug
created: 2026-04-27
updated: 2026-04-27
target: "stdlib/alloc/collections/sparse_set.nepl, tests/stdlib/sparse_set_collections.n.md, nodesrc/test_stdlib_sparse_set_no_unsafe_unwraps.js"
---

# ISS-20260427T022527514Z-SPARSESET-NEW-REJECTS-ZERO-UNIVERSE--0DB75A65: SparseSet new rejects zero universe length despite documented non-negative domain

## 概要

SparseSet.new documents the domain as [0, n) and only rejects n < 0, but n = 0 reaches alloc_ptr<i32> 0 for dense storage. alloc_raw returns 0 for size <= 0 and alloc_ptr converts that to Err, so an empty SparseSet universe is reported as allocation failure.

## 対象

- `stdlib/alloc/collections/sparse_set.nepl, tests/stdlib/sparse_set_collections.n.md`

## 根拠

- `SparseSet.new` は `n < 0` だけを `CapacityExceeded` として扱い、domain `[0, n)` の説明上 `n = 0` は空 domain として自然に扱える。
- 実装は header を確保した後、`alloc_ptr<i32> mul n 4` で dense storage を確保するため、`n = 0` では `alloc_ptr<i32> 0` になる。
- `alloc_raw` は `size <= 0` を 0 とし、`alloc_ptr` は raw pointer 0 を allocation failure として `Result::Err` に変換するため、空 SparseSet が out-of-memory と区別できない。

## 問題

SparseSet.new documents the domain as [0, n) and only rejects n < 0, but n = 0 reaches alloc_ptr<i32> 0 for dense storage. alloc_raw returns 0 for size <= 0 and alloc_ptr converts that to Err, so an empty SparseSet universe is reported as allocation failure.

## 影響

Self-host symbol/id set code cannot uniformly construct an empty sparse set. Callers must special-case zero universe size even though [0, 0) is a natural empty domain.

## 修正方針

Handle n = 0 as a valid SparseSet with null dense/sparse pointers, skip initialization loops, make free skip zero-byte array cleanup, and add empty new/free regression coverage.

## 解決内容

- header allocation 後に `n = 0` を明示的に扱い、`dense` / `sparse` は `mem_ptr_wrap 0` の null owner pointer として格納するようにした。
- `n > 0` の場合だけ dense/sparse 配列を確保して初期化するようにし、zero-byte allocation failure と通常の out-of-memory を分離した。
- `free` は既存の `dealloc_raw` no-op semantics により、null dense/sparse と header cleanup を安全に扱う形を維持した。
- `new 0` / `universe_len` / empty `contains` error / `free` / 再確保 regression と、zero-universe branch が戻らない source guard を追加した。

## 検証

- `node nodesrc/test_stdlib_sparse_set_no_unsafe_unwraps.js`: pass
- `node nodesrc/tests.js -i stdlib/alloc/collections/sparse_set.nepl --no-tree -o tmp/sparse-set-zero-docs.json -j 1`: 7/7 passed
- `node nodesrc/tests.js -i tests/stdlib/sparse_set_collections.n.md -i stdlib/tests/sparse_set.n.md --no-tree -o tmp/sparse-set-zero-focused.json -j 1`: 5/5 passed
- source policy regressions: pass
- `node nodesrc/tests.js -i tests/stdlib --no-tree -o tmp/tests-stdlib-sparse-zero.json -j 4`: 303/303 passed
- `node nodesrc/tests.js -i stdlib --no-tree -o tmp/stdlib-sparse-zero.json -j 4`: 418/418 passed
- `node nodesrc/issues.js check`: pass
