---
id: ISS-20260531T025203216Z-PRIVATE-CACHE-EFFECT-MASKING-FOR-PUR-DF36DE4F
title: "Private cache effect masking for pure memo_call"
area: core
status: open
resolved: false
priority: P1
type: architecture
created: 2026-05-31
updated: 2026-05-31
target: "doc/neplg2/private_effect_memoization_purity_design.md; nepl-core/src/effects.rs; nepl-core/src/resource/effect_check.rs; nepl-core/src/typecheck"
---

# ISS-20260531T025203216Z-PRIVATE-CACHE-EFFECT-MASKING-FOR-PUR-DF36DE4F: Private cache effect masking for pure memo_call

## 概要

NEPLg2 currently exposes only Pure/Impure at the surface and has internal Resource IR effect classes, but memoization needs private cache mutation to remain distinct from Pure until a fresh non-escaping region boundary masks it.

## 対象

- `doc/neplg2/private_effect_memoization_purity_design.md; nepl-core/src/effects.rs; nepl-core/src/resource/effect_check.rs; nepl-core/src/typecheck`

## 根拠

- Zenn の開発方針では、純粋性、静的検査、ゼロコスト抽象化を活かし、探索範囲や計算量を削減することが求められている。
- 現行 NEPLg2 の表層 effect は `Effect::Pure` / `Effect::Impure` の二値だが、compiler 内部には `InternalEffect` と Resource IR `EffectOp` があり、raw allocation、unsafe memory、external I/O、nondeterminism を区別している。
- `memo_call` は内部で cache mutation を行うが、cache region が fresh で外部から観測不能なら外部観測上は `func(key)` と同値にできる。
- ただし `PrivateCache` を直接 `Pure` と同一視すると、cache size、hit/miss、raw pointer、cache storage identity などを public API へ漏らす誤設計を検査できなくなる。

## 問題

NEPLg2 currently exposes only Pure/Impure at the surface and has internal Resource IR effect classes, but memoization needs private cache mutation to remain distinct from Pure until a fresh non-escaping region boundary masks it.

## 影響

Without an explicit private effect and escape/masking proof, memo_call can either be rejected even when observationally pure or, worse, be accepted by treating private mutable state as ordinary Pure and allowing public cache/state observation.

## 修正方針

Define PrivateCache/PrivateState internal effects, a mask boundary that proves fresh non-escaping region ownership, MemoKey/MemoValue trait constraints, and higher-order function value rules before exposing memo_call as pure fn -> pure fn.

## 2026-05-31 design checkpoint

- [NEPLg2 private effect / memoization purity design](../../doc/neplg2/private_effect_memoization_purity_design.md) を追加し、`Pure = no observable effect`、`PrivateCache rho は Pure ではなく mask 条件を満たす場合だけ Pure へ畳み込める` という contract を固定した。
- Phase 1 の `memo_call` は non-capturing named pure function value と Copy 相当の `MemoKey` / `MemoValue` だけを対象にする。Clone / non-Copy owner / Drop value は、cache hit 時の複製と drop obligation を Resource IR で証明するまで入れない。
- Resource IR の private state escape proof は [ISS-20260531T025408584Z-PRIVATE-STATE-MASKING-REQUIRES-RESOU-FCB116B4](./ISS-20260531T025408584Z-PRIVATE-STATE-MASKING-REQUIRES-RESOU-FCB116B4.md) に分離した。
- 高階関数の function identity / capture / partial application boundary は [ISS-20260531T025211459Z-HIGHER-ORDER-FUNCTION-PURITY-REQUIRE-A9CB99EE](./ISS-20260531T025211459Z-HIGHER-ORDER-FUNCTION-PURITY-REQUIRE-A9CB99EE.md) で扱う。

## 検証

Design doc, focused type/effect checker regressions, Resource IR escape regressions, and stdlib memo_call acceptance/rejection tests.
