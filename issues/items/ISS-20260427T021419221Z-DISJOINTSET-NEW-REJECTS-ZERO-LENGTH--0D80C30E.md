---
id: ISS-20260427T021419221Z-DISJOINTSET-NEW-REJECTS-ZERO-LENGTH--0D80C30E
title: "DisjointSet new rejects zero length despite documented non-negative length"
area: stdlib
status: open
resolved: false
priority: P2
type: bug
created: 2026-04-27
updated: 2026-04-27
target: "stdlib/alloc/collections/disjoint_set.nepl, tests/stdlib/disjoint_set_collections.n.md"
---

# ISS-20260427T021419221Z-DISJOINTSET-NEW-REJECTS-ZERO-LENGTH--0D80C30E: DisjointSet new rejects zero length despite documented non-negative length

## 概要

DisjointSet.new documents n < 0 as invalid, but n = 0 reaches alloc_ptr<i32> 0 for parent storage. alloc_raw returns 0 for size <= 0 and alloc_ptr converts that to Err, so an empty DisjointSet cannot be constructed even though the public contract implies it is valid.

## 対象

- `stdlib/alloc/collections/disjoint_set.nepl, tests/stdlib/disjoint_set_collections.n.md`

## 根拠

- `DisjointSet.new` は `n < 0` だけを `CapacityExceeded` として扱い、`n = 0` を不正値として扱っていない。
- `alloc_ptr<i32> mul n 4` は `n = 0` で `alloc_ptr<i32> 0` になり、`alloc_raw` の `size <= 0` は 0 を返す。
- `alloc_ptr` は raw pointer 0 を allocation failure として `Result::Err` に変換するため、仕様上は空集合として自然に扱える `n = 0` が out-of-memory と区別できない。

## 問題

DisjointSet.new documents n < 0 as invalid, but n = 0 reaches alloc_ptr<i32> 0 for parent storage. alloc_raw returns 0 for size <= 0 and alloc_ptr converts that to Err, so an empty DisjointSet cannot be constructed even though the public contract implies it is valid.

## 影響

Self-host graph algorithms and generic collection code cannot uniformly create empty union-find state. Callers must special-case zero elements or treat a valid empty collection as allocation failure.

## 修正方針

Handle n = 0 as a valid empty DisjointSet with null owned array pointers, skip initialization loops, make free a no-op for zero-byte arrays, and add empty new/free regression coverage.

## 検証

Run DisjointSet doctests, focused collection tests including new 0/free, stdlib suite, and nodesrc/issues.js check.
