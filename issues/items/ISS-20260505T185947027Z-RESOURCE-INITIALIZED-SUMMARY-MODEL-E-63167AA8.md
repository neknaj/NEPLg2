---
id: ISS-20260505T185947027Z-RESOURCE-INITIALIZED-SUMMARY-MODEL-E-63167AA8
title: "Resource initialized summary model exceeds responsibility split limit"
area: core
status: fixed
resolved: true
priority: P2
type: architecture
created: 2026-05-05
updated: 2026-05-06
target: "nepl-core/src/resource/initialized_summary.rs, nodesrc/test_resource_checker_responsibility.js"
---

# ISS-20260505T185947027Z-RESOURCE-INITIALIZED-SUMMARY-MODEL-E-63167AA8: Resource initialized summary model exceeds responsibility split limit

## 概要

After initialized_alias.rs was split, the direct Resource checker responsibility policy reaches initialized_summary.rs and reports 83 lines over the 80-line limit. The summary data model is only slightly over the current guard, but it still indicates initialized summary model helpers have grown past the documented boundary.

## 対象

- `nepl-core/src/resource/initialized_summary.rs, nodesrc/test_resource_checker_responsibility.js`

## 根拠

- `node nodesrc/test_resource_checker_responsibility.js` は `initialized_alias.rs` 分割後、次の未解決責務違反として `initialized_summary.rs has 83 lines; responsibility split limit is 80` を報告する。
- `initialized_summary_build.rs` と `initialized_summary_variant_build.rs` は既に分割済みだが、summary data model 側も guard をわずかに超えている。
- Stage 4 Resource check では initialized summary model が raw cell initialization / load requirement の contract になるため、data model と helper logic の境界を再確認する必要がある。

## 問題

After initialized_alias.rs was split, the direct Resource checker responsibility policy reaches initialized_summary.rs and reports 83 lines over the 80-line limit. The summary data model is only slightly over the current guard, but it still indicates initialized summary model helpers have grown past the documented boundary.

## 影響

Initialized summaries describe caller-visible raw cell initialization and load requirements. If the model module keeps accumulating helper logic, summary construction and application can drift away from a small auditable data contract for memory-safety checks.

## 修正方針

Review initialized_summary.rs and either move helper-only logic to the builder/apply modules or introduce a focused summary model helper module. Do not raise the limit unless the documented data contract genuinely requires it.

## 検証

Run node nodesrc/test_resource_checker_responsibility.js, cargo fmt --check -p nepl-core, cargo check -p nepl-core --tests, and focused initialized summary Resource IR tests.

## 2026-05-06 対応結果

`initialized_summary.rs` から `RawCellValueCondition` enum と `holds` 判定を `initialized_summary_condition.rs` へ分離した。`initialized_summary.rs` は `RawCellInitializationFunctionSummary` と各 raw cell summary entry の data contract に集中し、condition の意味論は専用 module で扱う。

分割後の行数は `initialized_summary.rs` 59 lines、`initialized_summary_condition.rs` 22 lines で、既存の 80 lines 上限内に戻った。`nodesrc/test_resource_checker_responsibility.js` には新 module の存在確認と line limit を追加した。

検証:

- `cargo fmt --check -p nepl-core`: passed
- `cargo check -p nepl-core --tests`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_cell_check_summarizes_unit_helper_argument_raw_cell_initialization -- --nocapture`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_cell_check_keeps_conditional_unit_helper_argument_init_conservative -- --nocapture`: passed
- `node nodesrc/test_resource_checker_responsibility.js`: `initialized_summary.rs` 超過は解消。次の別件として `initialized_summary_variant_build.rs has 337 lines; responsibility split limit is 260` を検出したため、`ISS-20260505T190408092Z-RESOURCE-INITIALIZED-SUMMARY-VARIANT-436C996D` を追加した。

関連計画: [NEPLg2 静的検査の複雑化解消計画 Stage 4](../../doc/neplg2/static_check_complexity_reduction_plan.md#stage-4-resource-check-%E3%81%B8%E3%81%AE%E7%A7%BB%E8%A1%8C)
