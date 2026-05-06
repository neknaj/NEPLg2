---
id: ISS-20260506T000648028Z-DIAGNOSTICS-PLAN-STILL-DESCRIBES-UNS-748EBB08
title: "unsafe memory gate docs still describe diagnostics as shadow-only"
area: core
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-06
updated: 2026-05-06
target: "doc/neplg2/compiler_diagnostics_redesign_plan.md, doc/neplg2/static_check_design_verification_20260430.md, doc/neplg2/static_check_soundness_review_20260430.md, doc/fullreview20260430/crosscutting/static-safety.md, doc/fullreview20260430/rust-compiler/pipeline-diagnostics.md, issues/items/ISS-20260429T040748194Z-RUST-COMPILER-DIAGNOSTICS-ARE-NOT-AL-1617747D.md"
---

# ISS-20260506T000648028Z-DIAGNOSTICS-PLAN-STILL-DESCRIBES-UNS-748EBB08: unsafe memory gate docs still describe diagnostics as shadow-only

## 概要

The diagnostics redesign plan, related static-check reviews, and the diagnostics parent issue still say ResourceEffectBoundaryDiagnostic::UnsafeMemoryInPureFunction is shadow-only during raw-memory-backed stdlib migration. Current main maps that diagnostic to effect.pure.calls_impure while keeping raw-memory-boundary source capability as the migration allowance.

## 対象

- `doc/neplg2/compiler_diagnostics_redesign_plan.md, doc/neplg2/static_check_design_verification_20260430.md, doc/neplg2/static_check_soundness_review_20260430.md, doc/fullreview20260430/crosscutting/static-safety.md, doc/fullreview20260430/rust-compiler/pipeline-diagnostics.md, issues/items/ISS-20260429T040748194Z-RUST-COMPILER-DIAGNOSTICS-ARE-NOT-AL-1617747D.md`

## 根拠

- `doc/neplg2/compiler_diagnostics_redesign_plan.md` の Stage D2 進捗が、`UnsafeMemoryInPureFunction` を shadow-only と説明していた。
- `issues/items/ISS-20260429T040748194Z-RUST-COMPILER-DIAGNOSTICS-ARE-NOT-AL-1617747D.md` の Stage D2 追記にも同じ説明が残っていた。
- `doc/neplg2/static_check_design_verification_20260430.md`、`doc/neplg2/static_check_soundness_review_20260430.md`、`doc/fullreview20260430/crosscutting/static-safety.md`、`doc/fullreview20260430/rust-compiler/pipeline-diagnostics.md` にも同じ古い状態が残っていた。
- 現在の `doc/neplg2/static_check_complexity_reduction_plan.md` と `doc/fullreview20260430/rust-compiler/static-check-resource.md` は、`UnsafeMemoryInPureFunction` が `effect.pure.calls_impure` として error 化済みであることを正としている。

## 問題

The diagnostics redesign plan, related static-check reviews, and the diagnostics parent issue still say ResourceEffectBoundaryDiagnostic::UnsafeMemoryInPureFunction is shadow-only during raw-memory-backed stdlib migration. Current main maps that diagnostic to effect.pure.calls_impure while keeping raw-memory-boundary source capability as the migration allowance.

## 影響

Future work can incorrectly treat unsafe memory effects in pure functions as non-authoritative, weakening the Stage 5 Resource IR effect boundary design and contradicting static_check_complexity_reduction_plan.md.

## 修正方針

Update the diagnostics plan, related static-check reviews, and parent issue so Stage D2 states that UnsafeMemoryInPureFunction is compiler-error mapped, raw-memory-boundary capability is the remaining Stage 6 migration allowance, and raw identity escape remains separately classified as resource.raw.identity_escape.

## 検証

node nodesrc/issues.js check; git diff --check

## 対応結果

`doc/neplg2/compiler_diagnostics_redesign_plan.md` と関連する static-check review doc の unsafe memory gate status を現在の Resource IR gate 実装に合わせて更新した。

- `UnsafeMemoryInPureFunction` は `Effect(PureCallsImpure)` / `effect.pure.calls_impure` へ error として写像する。
- raw-memory-backed stdlib migration の許可は shadow-only diagnostic ではなく、compiler-owned raw-memory-boundary capability を持つ source への限定許可として扱う。
- `resource.raw.*` は raw identity escape、raw capability/provenance boundary、pointer provenance そのものへ限定する。

親 issue `ISS-20260429T040748194Z-RUST-COMPILER-DIAGNOSTICS-ARE-NOT-AL-1617747D` には 2026-05-06 の Stage D2 status を追記した。親 issue は D3 以降の表示整理、test migration、self-host parity を追跡するため open のまま維持する。
