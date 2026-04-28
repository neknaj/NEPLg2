---
id: ISS-20260428T203931325Z-RESOURCE-IR-RAW-ADDRESS-SUMMARIES-DO-C7473DEA
title: "Resource IR raw address summaries do not evaluate literal arithmetic helper returns"
area: core
status: open
resolved: false
priority: P1
type: bug
created: 2026-04-28
updated: 2026-04-29
target: "nepl-core/src/resource/initialized_alias.rs, nepl-core/src/resource/lower.rs"
---

# ISS-20260428T203931325Z-RESOURCE-IR-RAW-ADDRESS-SUMMARIES-DO-C7473DEA: Resource IR raw address summaries do not evaluate literal arithmetic helper returns

## 概要

With temporary RawMemoryLoadCell enforcement, tests/compiler/move_effect.n.md::doctest#30 still reports false D3100 because a pure helper such as slot_ptr(base, 0) returns an address expression equivalent to base, but Resource IR raw address summaries do not evaluate literal arithmetic across helper boundaries.

## 対象

- `nepl-core/src/resource/initialized_alias.rs, nepl-core/src/resource/lower.rs`

## 根拠

- [NEPLg2 静的検査の複雑化解消計画 Stage 4](../../doc/neplg2/static_check_complexity_reduction_plan.md)
- 親 issue: [ISS-20260428T170745661Z-RESOURCE-IR-RAWMEMORYLOADCELL-GATE-N-1CBE1D0E](./ISS-20260428T170745661Z-RESOURCE-IR-RAWMEMORYLOADCELL-GATE-N-1CBE1D0E.md)
- `ISS-20260428T202704426Z-RESOURCE-IR-LOWERING-DOES-NOT-EXPOSE-0104A160` の修正後、一時 `RawMemoryLoadCell` gate で `tests/compiler/move_effect.n.md` は 109/110 まで改善し、残った失敗は `doctest#30` のみになった。
- `slot_ptr(base, 0)` のように literal 引数から結果 address が `base` と確定する helper を、Resource IR の raw address summary が表現できていない。

## 問題

With temporary RawMemoryLoadCell enforcement, tests/compiler/move_effect.n.md::doctest#30 still reports false D3100 because a pure helper such as slot_ptr(base, 0) returns an address expression equivalent to base, but Resource IR raw address summaries do not evaluate literal arithmetic across helper boundaries.

## 影響

RawMemoryLoadCell cannot become fully authoritative: valid raw address helpers with literal arguments fail before intended memory safety diagnostics, leaving one old HIR arithmetic summary dependency.

## 修正方針

Represent raw address return summaries as address expressions with parameter references and literal arithmetic, then instantiate them at call sites when arguments are literals or known raw address places.

## 検証

Add resource_ir regression for slot_ptr(base, 0) and confirm temporary RawMemoryLoadCell gate passes tests/compiler/move_effect.n.md.
