---
id: ISS-20260427T000312692Z-SPARSESET-RETAINS-UNSAFE-UNWRAPS-IN--D292C789
title: "SparseSet retains unsafe unwraps in dense/sparse internals"
area: stdlib
status: open
resolved: false
priority: P2
type: bug
created: 2026-04-27
updated: 2026-04-27
target: "stdlib/alloc/collections/sparse_set.nepl, tests/stdlib/sparse_set_collections.n.md"
---

# ISS-20260427T000312692Z-SPARSESET-RETAINS-UNSAFE-UNWRAPS-IN--D292C789: SparseSet retains unsafe unwraps in dense/sparse internals

## 概要

SparseSet header writes, initialization cleanup, and free paths use uwok on checked stores/deallocations for owned dense/sparse arrays.

## 対象

- `stdlib/alloc/collections/sparse_set.nepl, tests/stdlib/sparse_set_collections.n.md`

## 根拠

- 未記入

## 問題

SparseSet header writes, initialization cleanup, and free paths use uwok on checked stores/deallocations for owned dense/sparse arrays.

## 影響

Self-host symbol/id sets can fail through unreachable traps and the collection remains outside the Queue/Deque unsafe-helper policy.

## 修正方針

Add raw owner-invariant header/slot stores, replace cleanup with dealloc_raw, preserve Result errors for allocation failure, and add source and behavior regressions.

## 検証

Run SparseSet doctests, focused collection tests, source guard, stdlib suite, and nodesrc/issues.js check.
