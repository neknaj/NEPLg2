---
id: ISS-20260505T190408092Z-RESOURCE-INITIALIZED-SUMMARY-VARIANT-436C996D
title: "Resource initialized summary variant builder exceeds responsibility split limit again"
area: core
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-05
updated: 2026-05-06
target: "nepl-core/src/resource/initialized_summary_variant_build.rs, nepl-core/src/resource/initialized_summary_variant_condition.rs, nepl-core/src/resource/initialized_summary_variant_requirement.rs, nodesrc/test_resource_checker_responsibility.js"
---

# ISS-20260505T190408092Z-RESOURCE-INITIALIZED-SUMMARY-VARIANT-436C996D: Resource initialized summary variant builder exceeds responsibility split limit again

## 概要

After initialized_summary.rs was reduced to the data contract, the direct Resource checker responsibility policy reaches initialized_summary_variant_build.rs and reports 337 lines over the 260-line limit. Variant-gated initialized summary construction has accumulated condition collection, requirement collection, payload path traversal, and uniqueness helpers again.

## 対象

- `nepl-core/src/resource/initialized_summary_variant_build.rs, nodesrc/test_resource_checker_responsibility.js`

## 根拠

- `node nodesrc/test_resource_checker_responsibility.js` は `initialized_summary.rs` 分割後、次の未解決責務違反として `initialized_summary_variant_build.rs has 337 lines; responsibility split limit is 260` を報告する。
- `ISS-20260430T062125921Z-RESOURCE-INITIALIZED-SUMMARY-BUILDER-2875F09C` で variant builder は一度分離済みだが、その後の Result/Option branch gating、condition collection、raw load requirement collection が増え、variant builder 自体が再び上限を超えた。
- Stage 4 Resource check では variant-gated initialized summary が branch-specific memory-safety fact を担うため、path traversal、condition extraction、requirement collection、deduplication の境界を再確認する必要がある。

## 問題

After initialized_summary.rs was reduced to the data contract, the direct Resource checker responsibility policy reaches initialized_summary_variant_build.rs and reports 337 lines over the 260-line limit. Variant-gated initialized summary construction has accumulated condition collection, requirement collection, payload path traversal, and uniqueness helpers again.

## 影響

Variant-gated initialized summaries determine which Result/Option branches make raw cells initialized or required. If this builder remains concentrated, memory-safety checks can become difficult to audit and regressions in branch-specific initialization facts can hide behind a monolithic helper.

## 修正方針

Split initialized_summary_variant_build.rs by responsibility, such as variant path traversal, condition extraction, requirement collection, and uniqueness helpers, while preserving exact Result/Option gated summary semantics.

## 検証

Run node nodesrc/test_resource_checker_responsibility.js, cargo fmt --check -p nepl-core, cargo check -p nepl-core --tests, and focused initialized summary Resource IR tests.

## 2026-05-06 対応結果

`initialized_summary_variant_build.rs` から variant condition extraction と raw-load requirement collection を分離した。

- `initialized_summary_variant_condition.rs`: branch condition fact を variant-gated param condition へ変換し、dedup する。
- `initialized_summary_variant_requirement.rs`: variant path 内の raw memory load requirement を param suffix へ変換し、dedup する。
- `initialized_summary_variant_build.rs`: branch path traversal、path-local ResourceCheckEngine 実行、variant param initialized cell collection、enum construct detection に集中する。

分割後の行数は `initialized_summary_variant_build.rs` 209 lines、`initialized_summary_variant_condition.rs` 84 lines、`initialized_summary_variant_requirement.rs` 62 lines で、既存の 260 lines 上限内に戻った。`nodesrc/test_resource_checker_responsibility.js` には新 module の存在確認と line limit を追加した。

検証:

- `cargo fmt --check -p nepl-core`: passed
- `cargo check -p nepl-core --tests`: passed
- `node nodesrc/test_resource_checker_responsibility.js`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_cell_check_summarizes_unit_helper_argument_raw_cell_initialization -- --nocapture`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_cell_check_keeps_conditional_unit_helper_argument_init_conservative -- --nocapture`: passed
- `node nodesrc/run_source_policy_regressions.js --warn-only`: passed
- `node nodesrc/issues.js check`: passed
- `git diff --check`: passed
- Result::Ok/Err variant-gated source tests は既存の `ShadowSameSignatureCallable` warning を `typecheck_resource_source` helper が失敗扱いするため未完了。

関連計画: [NEPLg2 静的検査の複雑化解消計画 Stage 4](../../doc/neplg2/static_check_complexity_reduction_plan.md#stage-4-resource-check-%E3%81%B8%E3%81%AE%E7%A7%BB%E8%A1%8C)
