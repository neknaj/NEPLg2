---
id: ISS-20260427T000312512Z-DISJOINTSET-RETAINS-UNSAFE-UNWRAPS-I-3F8F89F7
title: "DisjointSet retains unsafe unwraps in owned array internals"
area: stdlib
status: open
resolved: false
priority: P2
type: bug
created: 2026-04-27
updated: 2026-04-27
target: "stdlib/alloc/collections/disjoint_set.nepl, tests/stdlib/disjoint_set_collections.n.md"
---

# ISS-20260427T000312512Z-DISJOINTSET-RETAINS-UNSAFE-UNWRAPS-I-3F8F89F7: DisjointSet retains unsafe unwraps in owned array internals

## 概要

DisjointSet uses uwok for parent/size array stores and cleanup paths even though those arrays are owned by the collection.

## 対象

- `stdlib/alloc/collections/disjoint_set.nepl, tests/stdlib/disjoint_set_collections.n.md`

## 根拠

- 未記入

## 問題

DisjointSet uses uwok for parent/size array stores and cleanup paths even though those arrays are owned by the collection.

## 影響

Union-find support for self-host graph algorithms can trap in internal maintenance code rather than exposing allocation failures through Result.

## 修正方針

Replace checked store/dealloc unwraps with raw owner-invariant helpers and dealloc_raw, add union/free regression coverage, and add a source guard.

## 検証

Run DisjointSet doctests, focused collection tests, source guard, stdlib suite, and nodesrc/issues.js check.
