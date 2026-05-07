---
id: ISS-20260507T125821563Z-RESOURCE-INITIALIZED-SUMMARY-MODEL-E-992DF2EE
title: "Resource initialized summary model exceeds responsibility split limit again"
area: core
status: open
resolved: false
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

## 問題

After lower_aggregate_projection was split, the direct Resource checker responsibility policy advanced to initialized_summary.rs and reported 123 lines against the 80-line model limit. The model file has re-accumulated raw cell summary data variants and helper-shaped contracts after recent returned-range summary work.

## 影響

ResourceIR initialized summary is part of Stage 4 memory-safety proof propagation. If the model file keeps growing, raw cell range/value/load requirement contracts become harder to audit and source policy cannot stay green.

## 修正方針

Split initialized_summary.rs by stable data responsibility instead of raising the limit. Move returned/parameter range count model or load requirement/release entries into focused model modules, update resource/mod.rs and responsibility policy, and preserve existing summary semantics.

## 検証

Run node nodesrc/test_resource_checker_responsibility.js, node nodesrc/run_source_policy_regressions.js --warn-only, cargo fmt/check, and focused ResourceIR initialized summary regressions.
