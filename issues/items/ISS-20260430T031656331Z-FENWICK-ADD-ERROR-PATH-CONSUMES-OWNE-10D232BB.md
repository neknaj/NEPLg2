---
id: ISS-20260430T031656331Z-FENWICK-ADD-ERROR-PATH-CONSUMES-OWNE-10D232BB
title: "Fenwick add error path consumes owner without cleanup or return"
area: stdlib
status: open
resolved: false
priority: P1
type: architecture
created: 2026-04-30
updated: 2026-04-30
target: "stdlib/alloc/collections/fenwick.nepl, stdlib/tests/fenwick.n.md"
---

# ISS-20260430T031656331Z-FENWICK-ADD-ERROR-PATH-CONSUMES-OWNE-10D232BB: Fenwick add error path consumes owner without cleanup or return

## 概要

Fenwick.add takes Fenwick by value and returns Result<Fenwick, Diag>, but the out-of-bounds branch returns Err(Diag) without returning the input Fenwick owner or freeing its internal bit array. The current bounds-error test matches add fw 5 1 and cannot free fw afterwards because the owner has been moved into add.

## 対象

- `stdlib/alloc/collections/fenwick.nepl, stdlib/tests/fenwick.n.md`

## 根拠

- Fenwick query observer の借用化中に、`stdlib/alloc/collections/fenwick.nepl` の `add` が `fn add <(Fenwick,i32,i32)*>Result<Fenwick, Diag>>` で入力 owner を値で受け取ることを確認した。
- `add` の範囲外 branch は `err<Fenwick, Diag> d` を返すだけで、入力 `fw` の `bit` owner を `free` せず、`Err` payload にも戻していない。
- `stdlib/tests/fenwick.n.md` の bounds-error test は `match add fw 5 1` で `Err` だけを確認しており、移動済みの `fw` を cleanup できる contract を検証していない。

## 問題

Fenwick.add takes Fenwick by value and returns Result<Fenwick, Diag>, but the out-of-bounds branch returns Err(Diag) without returning the input Fenwick owner or freeing its internal bit array. The current bounds-error test matches add fw 5 1 and cannot free fw afterwards because the owner has been moved into add.

## 影響

Callers that hit an invalid Fenwick update have no ownership-safe way to recover or dispose of the tree. This is a real API contract problem under mandatory memory safety and should not be hidden by tests that only check the Err value.

## 修正方針

Redesign Fenwick mutating error semantics so invalid updates either leave the owner borrowed and return a diagnostic, or return an error type that contains the original Fenwick owner for cleanup. Update tests to verify owner recovery or cleanup on Err.

## 検証

Add a test that triggers an invalid add and then either frees the returned owner or proves the borrowed owner remains usable and can be freed, with Resource owner checking enabled.
