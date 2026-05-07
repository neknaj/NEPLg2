---
id: ISS-20260507T132339456Z-RESOURCE-INITIALIZED-SUMMARY-VARIANT-32AEE691
title: "Resource initialized summary variant builder exceeds responsibility split limit after byte range split"
area: core
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-07
updated: 2026-05-07
target: "nepl-core/src/resource/initialized_summary_variant_build.rs, nodesrc/test_resource_checker_responsibility.js"
---

# ISS-20260507T132339456Z-RESOURCE-INITIALIZED-SUMMARY-VARIANT-32AEE691: Resource initialized summary variant builder exceeds responsibility split limit after byte range split

## 概要

After splitting initialized summary byte range builders, the direct Resource checker responsibility policy advanced to initialized_summary_variant_build.rs and reported 274 lines against the 260-line limit. Variant return path traversal, payload initialization, byte range propagation, and uniqueness helpers are still close enough to the limit that later memory-safety changes can re-concentrate responsibilities.

## 対象

- `nepl-core/src/resource/initialized_summary_variant_build.rs, nodesrc/test_resource_checker_responsibility.js`

## 根拠

- `ISS-20260507T131613193Z-RESOURCE-INITIALIZED-SUMMARY-BYTE-RA-F56D00D0` の修正で returned / param byte range builder と count-source extraction は 4 module に分離された。
- その直後の `node nodesrc/test_resource_checker_responsibility.js` は次の未解決 responsibility violation として `initialized_summary_variant_build.rs has 274 lines; responsibility split limit is 260` を報告した。
- `node nodesrc/run_source_policy_regressions.js --warn-only` でも同じ warning を確認した。
- 関連計画: [NEPLg2 静的検査の複雑化解消計画 Stage 4](../../doc/neplg2/static_check_complexity_reduction_plan.md#stage-4-resource-check-%E3%81%B8%E3%81%AE%E7%A7%BB%E8%A1%8C)

## 問題

After splitting initialized summary byte range builders, the direct Resource checker responsibility policy advanced to initialized_summary_variant_build.rs and reported 274 lines against the 260-line limit. Variant return path traversal, payload initialization, byte range propagation, and uniqueness helpers are still close enough to the limit that later memory-safety changes can re-concentrate responsibilities.

## 影響

Variant-gated initialized summary construction feeds branch-specific raw cell and byte range facts. If this builder remains over the strict split limit, Result/Option payload memory-safety proofs are harder to audit and future guarded range fixes can hide regressions.

## 修正方針

Split initialized_summary_variant_build.rs by stable responsibility instead of raising the limit. Move variant byte range propagation or uniqueness helpers into focused modules guarded by the responsibility policy, preserving variant-gated summary semantics.

## 検証

Run node nodesrc/test_resource_checker_responsibility.js, node nodesrc/run_source_policy_regressions.js --warn-only, cargo fmt/check, and focused ResourceIR variant/returned byte range regressions.

## 2026-05-07 対応結果

`initialized_summary_variant_build.rs` から variant param cell / byte range の dedup helper を `initialized_summary_variant_unique.rs` へ分離した。

- `initialized_summary_variant_build.rs`: variant return path traversal、path-local ResourceIR check、variant-gated param cell / byte range / requirement / condition construction を担当する。
- `initialized_summary_variant_unique.rs`: variant param cell と variant param byte range の uniqueness enforcement を担当する。
- `nodesrc/test_resource_checker_responsibility.js`: 新 module の存在、`mod initialized_summary_variant_unique;`、80 lines 上限を固定した。
- 分割後の行数は `initialized_summary_variant_build.rs` 258 / 260、`initialized_summary_variant_unique.rs` 24 / 80。

検証:

- `cargo fmt -p nepl-core --check`: passed
- `cargo check -p nepl-core`: passed
- `node nodesrc/test_resource_checker_responsibility.js`: passed
- `node nodesrc/run_source_policy_regressions.js --warn-only`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_cell_check_returned_raw_header_preserves_guarded_byte_range -- --nocapture`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_cell_check_returned_raw_header_rejects_unguarded_byte_range -- --nocapture`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_cell_check_returned_aggregate -- --nocapture`: 2 passed

関連計画: [NEPLg2 静的検査の複雑化解消計画 Stage 4](../../doc/neplg2/static_check_complexity_reduction_plan.md#stage-4-resource-check-%E3%81%B8%E3%81%AE%E7%A7%BB%E8%A1%8C)
