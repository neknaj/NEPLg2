---
id: ISS-20260512T101613486Z-STATIC-CHECK-PLAN-STILL-SAYS-HIR-INS-7F765C9B
title: "Static check plan still says HIR insert_drops remains after Resource IR drop consumer"
area: docs
status: fixed
resolved: true
priority: P2
type: bug
created: 2026-05-12
updated: 2026-05-12
target: doc/neplg2/static_check_complexity_reduction_plan.md
---

# ISS-20260512T101613486Z-STATIC-CHECK-PLAN-STILL-SAYS-HIR-INS-7F765C9B: Static check plan still says HIR insert_drops remains after Resource IR drop consumer

## 概要

static_check_complexity_reduction_plan.md の 2026-04-30 設計確認が、2026-05-06 に ResourceDropElaborationPlan consumer へ置換済みの HIR insert_drops を未完了 authority として記述し続けている。

## 対象

- `doc/neplg2/static_check_complexity_reduction_plan.md`

## 根拠

- `doc/neplg2/static_check_complexity_reduction_plan.md` の Stage 4 進捗には、2026-05-06 に `passes::insert_resource_drops` が checked `ResourceDropElaborationPlan` を消費し、旧 HIR `passes::insert_drops` 呼び出しを削除済みであることが記録されている。
- 同じ文書の 2026-04-30 設計確認には、drop elaboration 自体がまだ HIR `passes::insert_drops` に残る、という古い記述が残っていた。
- 完了条件付近にも、`HIR insert_drops` が drop elaboration authority として残ることを未完了点として扱う記述が残っていた。

## 問題

static_check_complexity_reduction_plan.md の 2026-04-30 設計確認が、2026-05-06 に ResourceDropElaborationPlan consumer へ置換済みの HIR insert_drops を未完了 authority として記述し続けている。

## 影響

Stage 4 の現状判断を誤らせ、旧 HIR drop walker を再導入または温存する誤った開発方針につながる。

## 修正方針

設計確認の記述を現在の実装に合わせ、旧 move_check fallback と旧 passes::insert_drops 呼び出しは削除済み、残件は full review/regression と Stage 5/6 の raw-memory public API 移行であることを明記する。

## 検証

node nodesrc/issues.js check --dir issues と git diff --check を実行する。

実行済み:

- `node nodesrc/issues.js check --dir issues`: passed
- `git diff --check`: passed
