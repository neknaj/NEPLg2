---
id: ISS-20260430T024900212Z-SPARSESET-READ-ONLY-APIS-CONSUME-OWN-E281EAC2
title: "SparseSet read-only APIs consume owners by value instead of borrowing"
area: stdlib
status: open
resolved: false
priority: P1
type: architecture
created: 2026-04-30
updated: 2026-04-30
target: stdlib/alloc/collections/sparse_set.nepl
---

# ISS-20260430T024900212Z-SPARSESET-READ-ONLY-APIS-CONSUME-OWN-E281EAC2: SparseSet read-only APIs consume owners by value instead of borrowing

## 概要

SparseSet len/universe_len/contains take SparseSet by value while reading only the header and arrays. Observing membership or length consumes the owner, so callers cannot safely free the dense/sparse storage afterwards.

## 対象

- `stdlib/alloc/collections/sparse_set.nepl`

## 根拠

- `stdlib/alloc/collections/sparse_set.nepl` に `fn len <(SparseSet)->i32>`、`fn universe_len <(SparseSet)->i32>`、`fn contains <(SparseSet,i32)*>Result<bool, Diag>>` が残っている。
- BitSet の owner-consuming observer 修正中に raw-array collection を確認し、header と dense/sparse arrays を読むだけの SparseSet observer も値 receiver のままだと判明した。

## 問題

SparseSet len/universe_len/contains take SparseSet by value while reading only the header and arrays. Observing membership or length consumes the owner, so callers cannot safely free the dense/sparse storage afterwards.

## 影響

SparseSet public observers encourage leaks or raw header workarounds in checked code. This conflicts with the no-technical-debt policy and with self-host collection use under strict owner checking.

## 修正方針

Redesign SparseSet observers around &SparseSet, update tests/doctests to borrow and then free, and remove by-value observer forms instead of retaining bad compatibility APIs.

## 検証

Add tests that run borrowed length and membership checks on one SparseSet, then free the same owner with owner checking enabled.
