---
id: ISS-20260505T223432842Z-RESOURCE-INITIALIZED-SUMMARY-APPLY-E-FEA66B2D
title: "Resource initialized summary apply exceeds responsibility split limit"
area: core
status: open
resolved: false
priority: P1
type: architecture
created: 2026-05-05
updated: 2026-05-05
target: "nepl-core/src/resource/initialized_summary_apply.rs, nepl-core/src/resource/initialized_summary.rs, nodesrc/test_resource_checker_responsibility.js"
---

# ISS-20260505T223432842Z-RESOURCE-INITIALIZED-SUMMARY-APPLY-E-FEA66B2D: Resource initialized summary apply exceeds responsibility split limit

## 概要

After lower_raw_address.rs was split, the Resource checker responsibility policy reached the next existing violation: initialized_summary_apply.rs has 170 lines while the split limit is 160. The module concentrates initialized-cell summary application, condition-gated variant requirements, alias/cell propagation, and caller-side state updates.

## 対象

- `nepl-core/src/resource/initialized_summary_apply.rs, nepl-core/src/resource/initialized_summary.rs, nodesrc/test_resource_checker_responsibility.js`

## 根拠

- `lower_raw_address.rs` の責務分割後、`node nodesrc/test_resource_checker_responsibility.js` は次の未解決責務違反として `initialized_summary_apply.rs has 170 lines; responsibility split limit is 160` を報告した。
- `initialized_summary_apply.rs` は initialized summary application、condition-gated variant requirement application、alias/cell propagation、caller-side state update を同居させており、initialized/moved state 検査の監査境界が太くなっている。
- 関連計画: [NEPLg2 静的検査の複雑化解消計画 Stage 4](../../doc/neplg2/static_check_complexity_reduction_plan.md#stage-4-resource-check-%E3%81%B8%E3%81%AE%E7%A7%BB%E8%A1%8C)

## 問題

After lower_raw_address.rs was split, the Resource checker responsibility policy reached the next existing violation: initialized_summary_apply.rs has 170 lines while the split limit is 160. The module concentrates initialized-cell summary application, condition-gated variant requirements, alias/cell propagation, and caller-side state updates.

## 影響

Initialized-cell summary application feeds the Resource IR initialized/moved/drop-state checks. If apply logic remains over-concentrated, moved/uninitialized cell regressions can be hidden behind mixed caller-summary and variant-condition responsibilities.

## 修正方針

Split initialized_summary_apply.rs by semantic role instead of raising the limit: keep top-level summary application orchestration in initialized_summary_apply.rs and extract condition-gated variant requirement application or alias/cell update helpers into focused modules guarded by the responsibility policy.

## 検証

node nodesrc/test_resource_checker_responsibility.js; node nodesrc/run_source_policy_regressions.js --warn-only; focused Resource IR initialized summary tests; cargo check -p nepl-core --tests; node nodesrc/issues.js check; git diff --check
