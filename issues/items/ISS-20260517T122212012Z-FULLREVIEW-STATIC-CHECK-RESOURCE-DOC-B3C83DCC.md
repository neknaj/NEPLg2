---
id: ISS-20260517T122212012Z-FULLREVIEW-STATIC-CHECK-RESOURCE-DOC-B3C83DCC
title: "Fullreview static-check resource document still describes resolved drop insertion blockers"
area: docs
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-17
updated: 2026-05-17
target: "doc/fullreview20260430/rust-compiler/static-check-resource.md, doc/neplg2/static_check_complexity_reduction_plan.md"
---

# ISS-20260517T122212012Z-FULLREVIEW-STATIC-CHECK-RESOURCE-DOC-B3C83DCC: Fullreview static-check resource document still describes resolved drop insertion blockers

## 概要

doc/fullreview20260430/rust-compiler/static-check-resource.md still says HIR passes::insert_drops is a remaining blocker and that ResourceDropElaborationPlan is not yet connected to real drop call generation, even though the compiler now consumes checked ResourceDropElaborationPlan and removed the legacy VarState walker path.

## 対象

- `doc/fullreview20260430/rust-compiler/static-check-resource.md, doc/neplg2/static_check_complexity_reduction_plan.md`

## 根拠

- `doc/fullreview20260430/rust-compiler/static-check-resource.md` の冒頭「残る問題」は、HIR `passes::insert_drops` / `VarState` scope walker がまだ残ると説明していた。
- 同じ文書の後半には `ResourceDropElaborationPlan consumer` 追補があり、実 drop call 生成は checked plan consumer へ移行済みであると説明していた。
- `ISS-20260506T113709479Z-RESOURCE-DROP-ELABORATION-PLAN-IS-NO-6CFFA860` は fixed / resolved で、compiler は旧 HIR drop scope walker を削除済みである。

## 問題

doc/fullreview20260430/rust-compiler/static-check-resource.md still says HIR passes::insert_drops is a remaining blocker and that ResourceDropElaborationPlan is not yet connected to real drop call generation, even though the compiler now consumes checked ResourceDropElaborationPlan and removed the legacy VarState walker path.

## 影響

The static-check review document gives a stale view of the compiler authority model. Developers may believe a legacy HIR drop insertion authority still exists and either duplicate work or reintroduce fallback design that Stage 4/6 already removed.

## 修正方針

Update the fullreview static-check resource document to distinguish the historical review snapshot from the current resolved state. Note the ResourceDropElaborationPlan consumer, final monomorphize path, and current remaining Stage 6 work accurately. Link the cleanup issue from the static check complexity reduction plan.

## 関連計画

- [静的検査の不必要な複雑化の解消についての大規模な修正の仕様と実装計画](../../doc/neplg2/static_check_complexity_reduction_plan.md)

## 対応内容

- fullreview の冒頭「残る問題」を更新し、旧 `passes::move_check::run` fallback と旧 HIR `passes::insert_drops` scope walker が削除済みであることを明記した。
- 設計評価を、旧 checker と Resource IR の二重防壁ではなく、現在の Resource IR authority と Stage 6 残件を区別する記述に直した。
- 2026-05-06 追補の優先順位説明を更新し、drop insertion 再実装ではなく owner/provenance capability、stdlib raw-memory-backed API 境界、Resource IR authority regression 固定が現在の優先事項であると整理した。
- 次の確認対象を更新し、owner variant path builder は解消済みで source policy 監視対象であることを明記した。

## 検証

- `node nodesrc/issues.js check --dir issues`: pass
- `git diff --check`: pass
