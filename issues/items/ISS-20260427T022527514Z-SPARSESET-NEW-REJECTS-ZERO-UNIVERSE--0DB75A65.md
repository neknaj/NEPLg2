---
id: ISS-20260427T022527514Z-SPARSESET-NEW-REJECTS-ZERO-UNIVERSE--0DB75A65
title: "SparseSet new rejects zero universe length despite documented non-negative domain"
area: stdlib
status: open
resolved: false
priority: P2
type: bug
created: 2026-04-27
updated: 2026-04-27
target: "stdlib/alloc/collections/sparse_set.nepl, tests/stdlib/sparse_set_collections.n.md"
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

## 検証

Run SparseSet doctests, focused collection tests including new 0/free, stdlib suite, and nodesrc/issues.js check.
