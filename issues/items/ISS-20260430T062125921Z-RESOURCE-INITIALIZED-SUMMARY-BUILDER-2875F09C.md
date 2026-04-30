---
id: ISS-20260430T062125921Z-RESOURCE-INITIALIZED-SUMMARY-BUILDER-2875F09C
title: "Resource initialized summary builder exceeds responsibility split limit"
area: core
status: open
resolved: false
priority: P1
type: architecture
created: 2026-04-30
updated: 2026-05-01
target: "nepl-core/src/resource/initialized_summary.rs, nepl-core/src/resource/initialized_summary_build.rs, nepl-core/src/resource/initialized_summary_variant_build.rs, nodesrc/test_resource_checker_responsibility.js"
---

# ISS-20260430T062125921Z-RESOURCE-INITIALIZED-SUMMARY-BUILDER-2875F09C: Resource initialized summary builder exceeds responsibility split limit

## 概要

After syncing remote main at 92a77c44, strict source policy fails because nepl-core/src/resource/initialized_summary_build.rs has 450 lines while nodesrc/test_resource_checker_responsibility.js limits initialized_summary_build.rs to 260 lines. The module now mixes fixed-point summary assembly, unconditional return/param fact collection, variant-gated Result/Option fact collection, raw load requirement collection, and uniqueness helpers.

## 対象

- `nepl-core/src/resource/initialized_summary_build.rs, nodesrc/test_resource_checker_responsibility.js`

## 根拠

- 未記入

## 問題

After syncing remote main at 92a77c44, strict source policy fails because nepl-core/src/resource/initialized_summary_build.rs has 450 lines while nodesrc/test_resource_checker_responsibility.js limits initialized_summary_build.rs to 260 lines. The module now mixes fixed-point summary assembly, unconditional return/param fact collection, variant-gated Result/Option fact collection, raw load requirement collection, and uniqueness helpers.

## 影響

The local strict source-policy gate fails on main-derived code, and Resource IR initialized summary responsibilities are concentrated in a single module. This weakens the static-check architecture boundary the project policy uses to keep memory-safety logic reviewable.

## 修正方針

Split variant-gated initialized-summary construction into a dedicated resource module, leaving initialized_summary_build.rs responsible for fixed-point orchestration and unconditional summary facts. Keep public behavior unchanged, update resource/mod.rs, and preserve the existing responsibility line limits.

## 検証

Run node nodesrc/run_source_policy_regressions.js, node nodesrc/issues.js check, git diff --check, and focused Resource IR tests covering initialized summaries.

## 対応結果

`initialized_summary_build.rs` から、戻り値 enum variant で gated された initialized summary の構築処理を `initialized_summary_variant_build.rs` へ分離した。

- `initialized_summary_build.rs` は fixed-point orchestration、unconditional return/param initialized fact の収集、guaranteed fact merge に責務を絞った。
- `initialized_summary_variant_build.rs` は `Result::Ok` / `Option::Some` などの variant-gated param cell initialization と raw load requirement の構築に責務を絞った。
- `resource/mod.rs` に新 module を追加し、外部 API と checker behavior は変更していない。
- 分割後の line count は `initialized_summary_build.rs` が 229 行、`initialized_summary_variant_build.rs` が 241 行で、source policy 上限内に戻った。

## 検証結果

- `cargo test -p nepl-core --test resource_ir -- --nocapture`: 141 passed
- `node nodesrc/run_source_policy_regressions.js`: passed
- `node nodesrc/issues.js check`: files=449, passed
- `git diff --check`: passed

## 2026-05-01 再発

Resource checker responsibility policy の再確認で、initialized summary 系が再び上限を超えている。

- `initialized_summary.rs`: 93/80
- `initialized_summary_build.rs`: 647/260
- `initialized_summary_variant_build.rs`: 337/260

fixed-point orchestration、summary data model、variant-gated fact collection、raw load requirement collection が再び大きい module に集まっている。owner/lower 側の責務分割とは別に、initialized summary builder を再分割する。
