---
id: ISS-20260507T131613193Z-RESOURCE-INITIALIZED-SUMMARY-BYTE-RA-F56D00D0
title: "Resource initialized summary byte range builder exceeds responsibility split limit"
area: core
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-07
updated: 2026-05-07
target: "nepl-core/src/resource/initialized_summary_byte_ranges.rs, nodesrc/test_resource_checker_responsibility.js"
---

# ISS-20260507T131613193Z-RESOURCE-INITIALIZED-SUMMARY-BYTE-RA-F56D00D0: Resource initialized summary byte range builder exceeds responsibility split limit

## 概要

After splitting initialized summary apply parameter updates, the direct Resource checker responsibility policy advanced to initialized_summary_byte_ranges.rs and reported 268 lines against the 140-line limit. Returned range collection, param range collection, count-source extraction, and deduplication are still concentrated in one memory-safety summary builder.

## 対象

- `nepl-core/src/resource/initialized_summary_byte_ranges.rs, nodesrc/test_resource_checker_responsibility.js`

## 根拠

- `ISS-20260507T130937432Z-RESOURCE-INITIALIZED-SUMMARY-APPLY-E-7FFA13D6` の修正で `initialized_summary_apply.rs` は 97 / 130 lines、`initialized_summary_apply_param.rs` は 69 / 100 lines まで分割された。
- その直後の `node nodesrc/test_resource_checker_responsibility.js` は次の未解決 responsibility violation として `initialized_summary_byte_ranges.rs has 268 lines; responsibility split limit is 140` を報告した。
- `node nodesrc/run_source_policy_regressions.js --warn-only` でも同じ warning を確認した。
- 関連計画: [NEPLg2 静的検査の複雑化解消計画 Stage 4](../../doc/neplg2/static_check_complexity_reduction_plan.md#stage-4-resource-check-%E3%81%B8%E3%81%AE%E7%A7%BB%E8%A1%8C)

## 問題

After splitting initialized summary apply parameter updates, the direct Resource checker responsibility policy advanced to initialized_summary_byte_ranges.rs and reported 268 lines against the 140-line limit. Returned range collection, param range collection, count-source extraction, and deduplication are still concentrated in one memory-safety summary builder.

## 影響

initialized_summary_byte_ranges.rs builds the dependent raw byte range facts used by ResourceIR initialized-state checks. If returned/param range collection and count proof extraction stay concentrated, range summary bugs can be hidden behind an oversized module and weaken auditability of guarded raw-memory loads.

## 修正方針

Split initialized_summary_byte_ranges.rs by stable responsibility instead of raising the limit. Separate returned range count/source extraction, param range count/source extraction, and uniqueness helpers into focused modules guarded by the responsibility policy while preserving guarded range semantics.

## 検証

Run node nodesrc/test_resource_checker_responsibility.js, node nodesrc/run_source_policy_regressions.js --warn-only, cargo fmt/check, and focused ResourceIR returned/param byte range regressions.

## 2026-05-07 対応結果

`initialized_summary_byte_ranges.rs` を削除し、returned / param raw byte range builder と count-source extraction を専用 module へ分離した。

- `initialized_summary_return_byte_ranges.rs`: returned raw byte range の address suffix collection と return summary entry construction を担当する。
- `initialized_summary_return_byte_range_count.rs`: returned raw byte range count の `KnownI32` / return value projection extraction を担当する。
- `initialized_summary_param_byte_ranges.rs`: parameter raw byte range の address suffix collection と param summary entry construction を担当する。
- `initialized_summary_param_byte_range_count.rs`: parameter raw byte range count の `KnownI32` / parameter projection extraction を担当する。
- `nodesrc/test_resource_checker_responsibility.js`: 4 module の存在、`mod` 宣言、line limit を固定した。
- 分割後の行数は param builder 86 / 140、param count 64 / 100、return builder 82 / 140、return count 60 / 100。

検証:

- `cargo fmt -p nepl-core --check`: passed
- `cargo check -p nepl-core`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_cell_check_returned_raw_header_preserves_guarded_byte_range -- --nocapture`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_cell_check_returned_raw_header_rejects_unguarded_byte_range -- --nocapture`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_cell_check_returned_aggregate -- --nocapture`: 2 passed
- `node nodesrc/run_source_policy_regressions.js --warn-only`: `initialized_summary_byte_ranges.rs` warning は解消。残る warning は別 issue `ISS-20260507T132339456Z-RESOURCE-INITIALIZED-SUMMARY-VARIANT-32AEE691` の `initialized_summary_variant_build.rs has 274 lines; responsibility split limit is 260`。
- `node nodesrc/test_resource_checker_responsibility.js`: byte range builder 超過は解消。次の別件として `initialized_summary_variant_build.rs` 超過を検出した。

関連計画: [NEPLg2 静的検査の複雑化解消計画 Stage 4](../../doc/neplg2/static_check_complexity_reduction_plan.md#stage-4-resource-check-%E3%81%B8%E3%81%AE%E7%A7%BB%E8%A1%8C)
