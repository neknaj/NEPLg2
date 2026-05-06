---
id: ISS-20260505T235821002Z-FULLREVIEW-STATIC-CHECK-RESOURCE-REP-F45743C8
title: "Fullreview static-check resource report still describes UnsafeMemory gate as shadow-only"
area: core
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-05
updated: 2026-05-06
target: "doc/fullreview20260430/rust-compiler/static-check-resource.md, doc/neplg2/static_check_complexity_reduction_plan.md"
---

# ISS-20260505T235821002Z-FULLREVIEW-STATIC-CHECK-RESOURCE-REP-F45743C8: Fullreview static-check resource report still describes UnsafeMemory gate as shadow-only

## 概要

doc/fullreview20260430/rust-compiler/static-check-resource.md still says UnsafeMemoryInPureFunction is shadow-only, but main now maps Resource IR UnsafeMemoryInPureFunction diagnostics to compiler errors while preserving only the raw-memory-boundary migration allowance.

## 対象

- `doc/fullreview20260430/rust-compiler/static-check-resource.md, doc/neplg2/static_check_complexity_reduction_plan.md`

## 根拠

- `doc/fullreview20260430/rust-compiler/static-check-resource.md` の残る問題に `UnsafeMemoryInPureFunction` は shadow-only のままと書かれていた。
- 現在の `nepl-core/src/compiler.rs` は `ResourceEffectBoundaryDiagnostic::UnsafeMemoryInPureFunction` を compiler diagnostic に変換している。
- `doc/neplg2/static_check_complexity_reduction_plan.md` は 2026-05-06 時点の Stage 5 状態として、ExternalIo / Nondet / UnsafeMemory が Resource IR diagnostic から compiler error へ接続済みと整理している。

## 問題

doc/fullreview20260430/rust-compiler/static-check-resource.md still says UnsafeMemoryInPureFunction is shadow-only, but main now maps Resource IR UnsafeMemoryInPureFunction diagnostics to compiler errors while preserving only the raw-memory-boundary migration allowance.

## 影響

The fullreview document is used to prioritize static-check work. If it still says the unsafe memory Resource IR gate is shadow-only, agents can misclassify Stage 5 as unfinished in the wrong place and duplicate or weaken already-enforced compiler gates.

## 修正方針

Update the fullreview static-check resource report to distinguish completed Stage 5 compiler error gating from remaining Stage 6 stdlib/raw-memory-boundary migration. Keep the open blocker list focused on old move_check/drop insertion authority and MemPtr/Storage/InitializedCell separation.

## 検証

node nodesrc/issues.js check; git diff --check

## 解決

`doc/fullreview20260430/rust-compiler/static-check-resource.md` を現在の main に合わせて更新した。

- effect boundary の現状説明に unsafe memory operation と host effect pure boundary を含めた。
- `UnsafeMemoryInPureFunction` を shadow-only とする古い記述を削除した。
- 2026-05-06 追補を追加し、Stage 5 で完了済みの compiler error gate と、Stage 6 以降に残る raw-memory-boundary / stdlib migration / old checker authority を分けて記載した。

検証:

- `node nodesrc/issues.js check`: passed
- `git diff --check`: passed
