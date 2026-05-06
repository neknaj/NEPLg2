---
id: ISS-20260506T034331265Z-RESOURCE-AGGREGATE-LOWERING-MODULE-E-2806B0D6
title: "Resource aggregate lowering module exceeds responsibility split limit again"
area: core
status: open
resolved: false
priority: P1
type: architecture
created: 2026-05-06
updated: 2026-05-06
target: "nepl-core/src/resource/lower_aggregate.rs, nodesrc/test_resource_checker_responsibility.js"
---

# ISS-20260506T034331265Z-RESOURCE-AGGREGATE-LOWERING-MODULE-E-2806B0D6: Resource aggregate lowering module exceeds responsibility split limit again

## 概要

After splitting coverage_hir_projection.rs, source policy reaches the next responsibility violation: lower_aggregate.rs has 366 lines while the enforced split limit is 320.

## 対象

- `nepl-core/src/resource/lower_aggregate.rs, nodesrc/test_resource_checker_responsibility.js`

## 根拠

- `ISS-20260506T030752155Z-RESOURCE-HIR-PROJECTION-COVERAGE-MOD-ED65CFB3` の修正後、`node nodesrc/test_resource_checker_responsibility.js` が次の未解決責務違反として `lower_aggregate.rs has 366 lines; responsibility split limit is 320` を報告した。
- `node nodesrc/run_source_policy_regressions.js --warn-only` でも同じ warning を確認した。coverage_hir_projection 関連は通過済みで、この warning は別 module の独立した責務分割問題である。

## 問題

After splitting coverage_hir_projection.rs, source policy reaches the next responsibility violation: lower_aggregate.rs has 366 lines while the enforced split limit is 320.

## 影響

lower_aggregate.rs now mixes compiler field load lowering, get/get_field projection lowering, raw aggregate field sources, and struct/tuple field resolution. Keeping these responsibilities together makes Resource IR lowering coverage and static-check correctness harder to audit.

## 修正方針

Split lower_aggregate.rs by aggregate lowering responsibility, for example moving field-resolution/source classification helpers away from lowering entrypoints, without weakening Resource IR lowering semantics.

## 検証

Run node nodesrc/test_resource_checker_responsibility.js and node nodesrc/run_source_policy_regressions.js --warn-only; the lower_aggregate.rs warning must disappear.
