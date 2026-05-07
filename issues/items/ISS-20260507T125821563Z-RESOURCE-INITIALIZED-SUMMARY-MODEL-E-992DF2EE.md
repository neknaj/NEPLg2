---
id: ISS-20260507T125821563Z-RESOURCE-INITIALIZED-SUMMARY-MODEL-E-992DF2EE
title: "Resource initialized summary model exceeds responsibility split limit again"
area: core
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-07
updated: 2026-05-07
target: "nepl-core/src/resource/initialized_summary.rs, nodesrc/test_resource_checker_responsibility.js"
---

# ISS-20260507T125821563Z-RESOURCE-INITIALIZED-SUMMARY-MODEL-E-992DF2EE: Resource initialized summary model exceeds responsibility split limit again

## 概要

After lower_aggregate_projection was split, the direct Resource checker responsibility policy advanced to initialized_summary.rs and reported 123 lines against the 80-line model limit. The model file has re-accumulated raw cell summary data variants and helper-shaped contracts after recent returned-range summary work.

## 対象

- `nepl-core/src/resource/initialized_summary.rs, nodesrc/test_resource_checker_responsibility.js`

## 根拠

- `ISS-20260507T112048241Z-RESOURCE-AGGREGATE-PROJECTION-MODULE-595EC35D` の修正後、`lower_aggregate_projection.rs` は 149 lines / limit 180、`lower_aggregate_selector.rs` は 60 lines / limit 100 まで分割できた。
- その直後の `node nodesrc/test_resource_checker_responsibility.js` は次の未解決 responsibility violation として `initialized_summary.rs has 123 lines; responsibility split limit is 80` を報告した。
- `node nodesrc/run_source_policy_regressions.js --warn-only` でも同じ warning を確認した。
- 関連計画: [NEPLg2 静的検査の複雑化解消計画 Stage 4](../../doc/neplg2/static_check_complexity_reduction_plan.md#stage-4-resource-check-%E3%81%B8%E3%81%AE%E7%A7%BB%E8%A1%8C)

## 問題

After lower_aggregate_projection was split, the direct Resource checker responsibility policy advanced to initialized_summary.rs and reported 123 lines against the 80-line model limit. The model file has re-accumulated raw cell summary data variants and helper-shaped contracts after recent returned-range summary work.

## 影響

ResourceIR initialized summary is part of Stage 4 memory-safety proof propagation. If the model file keeps growing, raw cell range/value/load requirement contracts become harder to audit and source policy cannot stay green.

## 修正方針

Split initialized_summary.rs by stable data responsibility instead of raising the limit. Move returned/parameter range count model or load requirement/release entries into focused model modules, update resource/mod.rs and responsibility policy, and preserve existing summary semantics.

## 検証

Run node nodesrc/test_resource_checker_responsibility.js, node nodesrc/run_source_policy_regressions.js --warn-only, cargo fmt/check, and focused ResourceIR initialized summary regressions.

## 2026-05-07 対応結果

`initialized_summary.rs` から returned / param / variant param byte range と range count enum を `initialized_summary_byte_range_model.rs` へ分離した。

- `initialized_summary.rs`: raw cell initialization function summary、return/param/variant cell、variant requirement/condition の集約 contract を担当する。
- `initialized_summary_byte_range_model.rs`: returned raw byte range、param raw byte range、variant param raw byte range、および `KnownI32` / projection count enum を担当する。
- `nodesrc/test_resource_checker_responsibility.js`: 新 module の存在、`mod initialized_summary_byte_range_model;`、80 lines 上限を固定した。
- 分割後の行数は `initialized_summary.rs` 70 / 80、`initialized_summary_byte_range_model.rs` 64 / 80。

検証:

- `cargo fmt -p nepl-core --check`: passed
- `cargo check -p nepl-core`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_cell_check_returned_raw_header_preserves_guarded_byte_range -- --nocapture`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_cell_check_returned_raw_header_rejects_unguarded_byte_range -- --nocapture`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_cell_check_returned_aggregate -- --nocapture`: 2 passed
- `node nodesrc/run_source_policy_regressions.js --warn-only`: `initialized_summary.rs` warning は解消。残る warning は別 issue `ISS-20260507T130937432Z-RESOURCE-INITIALIZED-SUMMARY-APPLY-E-7FFA13D6` の `initialized_summary_apply.rs has 151 lines; responsibility split limit is 130`。
- `node nodesrc/test_resource_checker_responsibility.js`: `initialized_summary.rs` 超過は解消。次の別件として `initialized_summary_apply.rs` 超過を検出した。

関連計画: [NEPLg2 静的検査の複雑化解消計画 Stage 4](../../doc/neplg2/static_check_complexity_reduction_plan.md#stage-4-resource-check-%E3%81%B8%E3%81%AE%E7%A7%BB%E8%A1%8C)
