---
id: ISS-20260427T000312112Z-FENWICK-RETAINS-UNSAFE-UNWRAPS-IN-OW-902C92DE
title: "Fenwick retains unsafe unwraps in owned tree internals"
area: stdlib
status: open
resolved: false
priority: P2
type: bug
created: 2026-04-27
updated: 2026-04-27
target: "stdlib/alloc/collections/fenwick.nepl, tests/stdlib/fenwick_collections.n.md"
---

# ISS-20260427T000312112Z-FENWICK-RETAINS-UNSAFE-UNWRAPS-IN-OW-902C92DE: Fenwick retains unsafe unwraps in owned tree internals

## 概要

Fenwick initialization, update, and free paths call uwok on checked store_i32/dealloc_ptr even though the tree owns the backing array.

## 対象

- `stdlib/alloc/collections/fenwick.nepl, tests/stdlib/fenwick_collections.n.md`

## 根拠

- 未記入

## 問題

Fenwick initialization, update, and free paths call uwok on checked store_i32/dealloc_ptr even though the tree owns the backing array.

## 影響

Self-host frequency/prefix-sum helpers can trap on internal bookkeeping paths, and allocation cleanup semantics remain inconsistent across collections.

## 修正方針

Introduce raw owned-array store helpers, replace owned cleanup with dealloc_raw, keep public failures as Result, and add source and behavior regressions.

## 検証

Run Fenwick doctests, focused collection tests, source guard, stdlib suite, and nodesrc/issues.js check.
