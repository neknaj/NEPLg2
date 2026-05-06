---
id: ISS-20260506T005443213Z-MOVE-EFFECT-DOCTESTS-ARE-STALE-AFTER-6955AAD2
title: "move_effect doctests are stale after Resource IR and effect gates"
area: TEST
status: open
resolved: false
priority: P1
type: test
created: 2026-05-06
updated: 2026-05-06
target: "tests/compiler/move_effect.n.md, doc/neplg2/static_check_complexity_reduction_plan.md"
---

# ISS-20260506T005443213Z-MOVE-EFFECT-DOCTESTS-ARE-STALE-AFTER-6955AAD2: move_effect doctests are stale after Resource IR and effect gates

## 概要

A focused run of tests/compiler/move_effect.n.md after the Resource IR/effect gate migration reports 94 passed and 36 failed. Several fixtures still use raw memory operations in pure functions or load non-Copy raw cells from uninitialized fixed addresses while expecting older resource.cell/resource.move diagnostics.

## 対象

- `tests/compiler/move_effect.n.md, doc/neplg2/static_check_complexity_reduction_plan.md`

## 根拠

- 2026-05-06 の focused run で `tests/compiler/move_effect.n.md` は 130 件中 94 passed / 36 failed だった。
- 主な失敗は `effect.pure.calls_impure` が先に出る raw memory fixture、`resource.cell.uninit` が先に出る未初期化 raw load fixture、Resource IR の `resource.cell.*` が legacy `resource.move.*` より先に出る diagnostic taxonomy drift である。
- `doc/neplg2/static_check_complexity_reduction_plan.md` の Stage 4/5 方針では compiler を弱めず、fixture 側を現在の Resource IR / effect gate authority に合わせる必要がある。

## 問題

A focused run of tests/compiler/move_effect.n.md after the Resource IR/effect gate migration reports 94 passed and 36 failed. Several fixtures still use raw memory operations in pure functions or load non-Copy raw cells from uninitialized fixed addresses while expecting older resource.cell/resource.move diagnostics.

## 影響

The suite no longer cleanly isolates effect boundary, raw cell initialization, moved-cell, and legacy move diagnostics. CI failures can be misread as compiler regressions, or stale expectations can pressure the compiler to weaken static safety.

## 修正方針

Split the fixtures by invariant: keep pure raw operation tests expecting effect.pure.calls_impure, mark raw cell state fixtures impure, initialize raw storage before moved-cell assertions, and update diag_code expectations to Resource IR cell/owner/effect taxonomy without restoring legacy buckets.

## 検証

trunk build; node nodesrc/tests.js -i tests/compiler/move_effect.n.md -o output/move_effect.json --runner wasm --no-tree -j 1
