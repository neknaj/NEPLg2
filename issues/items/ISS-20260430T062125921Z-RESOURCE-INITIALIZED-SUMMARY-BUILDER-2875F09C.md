---
id: ISS-20260430T062125921Z-RESOURCE-INITIALIZED-SUMMARY-BUILDER-2875F09C
title: "Resource initialized summary builder exceeds responsibility split limit"
area: core
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-04-30
updated: 2026-05-01
target: "nepl-core/src/resource/initialized_summary.rs, nepl-core/src/resource/initialized_summary_build.rs, nepl-core/src/resource/initialized_summary_cells.rs, nepl-core/src/resource/initialized_summary_condition.rs, nepl-core/src/resource/initialized_summary_destruction.rs, nepl-core/src/resource/initialized_summary_destruction_address.rs, nepl-core/src/resource/initialized_summary_variant_build.rs, nepl-core/src/resource/initialized_summary_variant_condition.rs, nepl-core/src/resource/initialized_summary_variant_requirement.rs, nodesrc/test_resource_checker_responsibility.js"
---

# ISS-20260430T062125921Z-RESOURCE-INITIALIZED-SUMMARY-BUILDER-2875F09C: Resource initialized summary builder exceeds responsibility split limit

## 概要

After syncing remote main at 92a77c44, strict source policy fails because nepl-core/src/resource/initialized_summary_build.rs has 450 lines while nodesrc/test_resource_checker_responsibility.js limits initialized_summary_build.rs to 260 lines. The module now mixes fixed-point summary assembly, unconditional return/param fact collection, variant-gated Result/Option fact collection, raw load requirement collection, and uniqueness helpers.

## 対象

- `nepl-core/src/resource/initialized_summary_build.rs, nodesrc/test_resource_checker_responsibility.js`

## 根拠

- [NEPLg2 静的検査の複雑化解消計画 Stage 4](../../doc/neplg2/static_check_complexity_reduction_plan.md)

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

## 2026-05-01 修正

再発の根本原因は、summary data model、fixed-point orchestration、return / param initialized cell collection、param destruction propagation、variant condition / requirement collection が `initialized_summary_build.rs` と `initialized_summary_variant_build.rs` に再集約されていたこと。

責務境界を以下に分離した。

- `initialized_summary.rs`: raw cell initialization summary のデータ構造。
- `initialized_summary_condition.rs`: raw cell value condition enum と判定。
- `initialized_summary_build.rs`: function summary の fixed-point orchestration。
- `initialized_summary_cells.rs`: return / param initialized raw cell fact collection。
- `initialized_summary_destruction.rs`: ResourceOp traversal による param destruction propagation。
- `initialized_summary_destruction_address.rs`: destructive raw memory / call summary から param-relative address destruction への変換。
- `initialized_summary_variant_build.rs`: Result / Option variant-gated summary collection の入口。
- `initialized_summary_variant_condition.rs`: branch condition から variant condition fact への変換。
- `initialized_summary_variant_requirement.rs`: variant-gated raw load requirement collection。

分割後の行数は `initialized_summary.rs` 70/80、`initialized_summary_build.rs` 156/260、`initialized_summary_cells.rs` 90/120、`initialized_summary_condition.rs` 22/60、`initialized_summary_destruction.rs` 262/300、`initialized_summary_destruction_address.rs` 173/200、`initialized_summary_variant_build.rs` 209/260、`initialized_summary_variant_condition.rs` 84/120、`initialized_summary_variant_requirement.rs` 62/100。`node nodesrc/test_resource_checker_responsibility.js` は pass に戻った。

確認:

- `cargo fmt --check`: pass
- `cargo check -p nepl-core`: pass
- `node nodesrc/test_resource_checker_responsibility.js`: pass
- `cargo test -p nepl-core --test resource_ir resource_ir_cell_check_preserves_initialized_raw_cells_returned_by_branch_helper -- --nocapture`: pass
- `cargo test -p nepl-core --test resource_ir resource_ir_cell_check_keeps_conditional_unit_helper_argument_init_conservative -- --nocapture`: pass
- `cargo test -p nepl-core --test resource_ir resource_ir_cell_check_applies_result_ok_param_raw_cell_initialization -- --nocapture`: pass
- `cargo test -p nepl-core --test resource_ir resource_ir_cell_check_does_not_apply_result_err_param_raw_cell_initialization -- --nocapture`: pass
- `cargo test -p nepl-core --test resource_ir resource_ir_cell_check_skips_unreachable_mem_ptr_load_some_requirement -- --nocapture`: pass
- `cargo test -p nepl-core --test resource_ir resource_ir_cell_check_keeps_reachable_mem_ptr_load_some_requirement -- --nocapture`: pass
