---
id: ISS-20260430T064747382Z-STATIC-CHECK-DESIGN-NEEDS-PASS-BY-PA-4ADB193A
title: "Static check design needs pass-by-pass soundness review"
area: docs
status: verified
resolved: true
priority: P1
type: architecture
created: 2026-04-30
updated: 2026-04-30
target: "doc/neplg2/static_check_soundness_review_20260430.md, doc/neplg2/static_check_complexity_reduction_plan.md, doc/neplg2/static_check_design_verification_20260430.md"
---

# ISS-20260430T064747382Z-STATIC-CHECK-DESIGN-NEEDS-PASS-BY-PA-4ADB193A: Static check design needs pass-by-pass soundness review

## 概要

The existing static-check docs describe the Resource IR migration direction, but the latest review needs a stricter pass-by-pass soundness matrix: what each stage guarantees, which gate is currently authoritative, which diagnostics are compiler errors, and which areas still depend on old HIR checks or shadow-only behavior.

## 対象

- `doc/neplg2/static_check_soundness_review_20260430.md, doc/neplg2/static_check_complexity_reduction_plan.md, doc/neplg2/static_check_design_verification_20260430.md`

## 根拠

- `nepl-core/src/compiler.rs` の `run_move_check` は `passes::move_check::run` を先に通した後、Resource IR lowering coverage / cell / borrow / effect / owner gate を実行する。
- `nepl-core/src/compiler.rs` の `prepare_module_for_codegen_with_source_map` は `passes::insert_drops` を HIR 上で実行する。
- `ResourceCheckDiagnostic::CellUnavailable` は compiler boundary で raw-memory cell operation のみ hard error へ写像され、通常 read/move/drop/call argument は旧 checker 防壁に依存している。
- `ResourceEffectBoundaryDiagnostic::UnsafeMemoryInPureFunction` は compiler diagnostic へ写像されず shadow-only であり、`EffectOp::Unknown` は count だけで hard error ではない。

## 問題

The existing static-check docs describe the Resource IR migration direction, but the latest review needs a stricter pass-by-pass soundness matrix: what each stage guarantees, which gate is currently authoritative, which diagnostics are compiler errors, and which areas still depend on old HIR checks or shadow-only behavior.

## 影響

Without a precise authority/invariant matrix, future Rust and self-host work can mistake transitional gates for final design, copy old HIR special cases into self-host, or miss that some Resource IR reports are still filtered by raw-memory boundaries or old move_check coverage.

## 修正方針

Add a detailed static-check soundness review document and link it from the complexity/design docs. The review must separate completed guarantees from transitional defenses and final-design blockers.

## 検証

Run source-policy regressions, focused issue checks, git diff --check, and update issues index.

実行済み:

- `node nodesrc/run_source_policy_regressions.js`: passed
- `node nodesrc/issues.js check`: passed
- `git diff --check`: passed
