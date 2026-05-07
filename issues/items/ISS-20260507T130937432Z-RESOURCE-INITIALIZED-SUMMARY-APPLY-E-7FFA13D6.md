---
id: ISS-20260507T130937432Z-RESOURCE-INITIALIZED-SUMMARY-APPLY-E-7FFA13D6
title: "Resource initialized summary apply exceeds responsibility split limit again"
area: core
status: open
resolved: false
priority: P1
type: architecture
created: 2026-05-07
updated: 2026-05-07
target: "nepl-core/src/resource/initialized_summary_apply.rs, nodesrc/test_resource_checker_responsibility.js"
---

# ISS-20260507T130937432Z-RESOURCE-INITIALIZED-SUMMARY-APPLY-E-7FFA13D6: Resource initialized summary apply exceeds responsibility split limit again

## 概要

After splitting initialized summary byte range/count models, the direct Resource checker responsibility policy advanced to initialized_summary_apply.rs and reported 151 lines against the 130-line limit. The module has re-accumulated returned/param range count application and caller-side initialized summary update responsibilities after recent dependent range work.

## 対象

- `nepl-core/src/resource/initialized_summary_apply.rs, nodesrc/test_resource_checker_responsibility.js`

## 根拠

- `ISS-20260507T125821563Z-RESOURCE-INITIALIZED-SUMMARY-MODEL-E-992DF2EE` の修正で `initialized_summary.rs` は 70 / 80 lines、`initialized_summary_byte_range_model.rs` は 64 / 80 lines まで分割された。
- その直後の `node nodesrc/test_resource_checker_responsibility.js` は次の未解決 responsibility violation として `initialized_summary_apply.rs has 151 lines; responsibility split limit is 130` を報告した。
- `node nodesrc/run_source_policy_regressions.js --warn-only` でも同じ warning を確認した。
- 関連計画: [NEPLg2 静的検査の複雑化解消計画 Stage 4](../../doc/neplg2/static_check_complexity_reduction_plan.md#stage-4-resource-check-%E3%81%B8%E3%81%AE%E7%A7%BB%E8%A1%8C)

## 問題

After splitting initialized summary byte range/count models, the direct Resource checker responsibility policy advanced to initialized_summary_apply.rs and reported 151 lines against the 130-line limit. The module has re-accumulated returned/param range count application and caller-side initialized summary update responsibilities after recent dependent range work.

## 影響

initialized_summary_apply.rs applies ResourceIR initialized summary facts that decide moved/uninitialized raw-cell availability. If it keeps growing past the strict split guard, memory-safety proof propagation becomes harder to audit and future changes can hide incorrect summary application.

## 修正方針

Split initialized_summary_apply.rs by semantic role instead of raising the limit. Keep top-level call summary application in initialized_summary_apply.rs, and move returned/param range count application or caller-side raw range update helpers into focused modules guarded by the responsibility policy.

## 検証

Run node nodesrc/test_resource_checker_responsibility.js, node nodesrc/run_source_policy_regressions.js --warn-only, cargo fmt/check, and focused ResourceIR initialized summary regressions.
