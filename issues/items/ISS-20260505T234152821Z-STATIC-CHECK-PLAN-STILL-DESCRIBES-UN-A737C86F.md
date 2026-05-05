---
id: ISS-20260505T234152821Z-STATIC-CHECK-PLAN-STILL-DESCRIBES-UN-A737C86F
title: "Static check plan still describes unsafe memory as shadow-only"
area: doc
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-05
updated: 2026-05-06
target: "doc/neplg2/static_check_complexity_reduction_plan.md, issues/items/ISS-20260427T132406497Z-CORE-MEM-RAW-MEMORY-OPERATIONS-BYPAS-DC67DF04.md"
---

# ISS-20260505T234152821Z-STATIC-CHECK-PLAN-STILL-DESCRIBES-UN-A737C86F: Static check plan still describes unsafe memory as shadow-only

## 概要

doc/neplg2/static_check_complexity_reduction_plan.md still says UnsafeMemoryInPureFunction remains shadow-only, but Resource IR unsafe memory diagnostics are now mapped to compiler errors outside raw-memory-boundary sources.

## 対象

- `doc/neplg2/static_check_complexity_reduction_plan.md, issues/items/ISS-20260427T132406497Z-CORE-MEM-RAW-MEMORY-OPERATIONS-BYPAS-DC67DF04.md`

## 根拠

- `doc/neplg2/static_check_complexity_reduction_plan.md` の 2026-04-30 設計確認 section が `UnsafeMemoryInPureFunction` を shadow-only と記述していた。
- 2026-05-06 の Stage 5 実装で `UnsafeMemoryInPureFunction` は `effect.pure.calls_impure` へ error mapping されるようになった。
- raw-memory-boundary capability による移行中許可は残るが、これは shadow-only ではなく source capability による明示的な gate 例外である。

## 問題

doc/neplg2/static_check_complexity_reduction_plan.md still says UnsafeMemoryInPureFunction remains shadow-only, but Resource IR unsafe memory diagnostics are now mapped to compiler errors outside raw-memory-boundary sources.

## 影響

The static-check migration plan gives the wrong authority boundary to later agents and can cause future work to preserve a debt that has already been removed.

## 修正方針

Update the Stage 5 status section to state that UnsafeMemoryInPureFunction is now gated by Resource IR, while raw-memory-boundary source capability remains the migration allowance.

## 対応

- 静的検査計画の Stage 5 状態を、`UnsafeMemoryInPureFunction` が Resource IR gate から compiler error へ接続済みである説明へ更新した。
- 未完了点を「shadow-only」ではなく「raw-memory-boundary capability による stdlib 移行中許可が残る」へ修正した。
- 親 raw memory boundary issue に、仕様 doc の Stage 5 status 更新を追記した。

## 検証

- `node nodesrc/issues.js check`: commit 前に実行
- `git diff --check`: commit 前に実行
