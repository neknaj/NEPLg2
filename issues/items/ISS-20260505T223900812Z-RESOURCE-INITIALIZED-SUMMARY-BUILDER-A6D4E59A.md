---
id: ISS-20260505T223900812Z-RESOURCE-INITIALIZED-SUMMARY-BUILDER-A6D4E59A
title: "Resource initialized summary builder exceeds responsibility split limit again"
area: core
status: open
resolved: false
priority: P1
type: architecture
created: 2026-05-05
updated: 2026-05-05
target: "nepl-core/src/resource/initialized_summary_build.rs, nepl-core/src/resource/initialized_summary_variant_build.rs, nodesrc/test_resource_checker_responsibility.js"
---

# ISS-20260505T223900812Z-RESOURCE-INITIALIZED-SUMMARY-BUILDER-A6D4E59A: Resource initialized summary builder exceeds responsibility split limit again

## 概要

After initialized_summary_apply.rs was split, the Resource checker responsibility policy reached the next existing violation: initialized_summary_build.rs has 628 lines while the split limit is 260. The module appears to concentrate raw-cell initialization summary fixed-point construction, return/parameter cell extraction, release requirement construction, and helper classification.

## 対象

- `nepl-core/src/resource/initialized_summary_build.rs, nepl-core/src/resource/initialized_summary_variant_build.rs, nodesrc/test_resource_checker_responsibility.js`

## 根拠

- `initialized_summary_apply.rs` の責務分割後、`node nodesrc/test_resource_checker_responsibility.js` は次の未解決責務違反として `initialized_summary_build.rs has 628 lines; responsibility split limit is 260` を報告した。
- `initialized_summary_build.rs` は raw-cell initialization summary の固定点構築、return cell collection、parameter cell collection、release requirement construction、helper classification を同居させており、initialized-state summary の監査境界が大きく崩れている。
- 関連計画: [NEPLg2 静的検査の複雑化解消計画 Stage 4](../../doc/neplg2/static_check_complexity_reduction_plan.md#stage-4-resource-check-%E3%81%B8%E3%81%AE%E7%A7%BB%E8%A1%8C)

## 問題

After initialized_summary_apply.rs was split, the Resource checker responsibility policy reached the next existing violation: initialized_summary_build.rs has 628 lines while the split limit is 260. The module appears to concentrate raw-cell initialization summary fixed-point construction, return/parameter cell extraction, release requirement construction, and helper classification.

## 影響

Initialized summary build determines which raw cells are initialized, released, or carried through call boundaries. If builder responsibilities remain concentrated, Resource IR initialized-state soundness can regress when raw memory or variant-return cases change.

## 修正方針

Split initialized_summary_build.rs by semantic role instead of raising the limit: keep fixed-point orchestration in the builder and extract return-cell collection, parameter-cell collection, release requirement construction, and helper classification into focused modules guarded by responsibility policy.

## 検証

node nodesrc/test_resource_checker_responsibility.js; node nodesrc/run_source_policy_regressions.js --warn-only; focused Resource IR initialized summary/raw-cell tests; cargo check -p nepl-core --tests; node nodesrc/issues.js check; git diff --check
