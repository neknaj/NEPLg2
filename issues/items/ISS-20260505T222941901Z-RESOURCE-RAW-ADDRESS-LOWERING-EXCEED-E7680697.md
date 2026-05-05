---
id: ISS-20260505T222941901Z-RESOURCE-RAW-ADDRESS-LOWERING-EXCEED-E7680697
title: "Resource raw address lowering exceeds responsibility split limit"
area: core
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-05
updated: 2026-05-06
target: "nepl-core/src/resource/lower_raw_address.rs, nepl-core/src/resource/lower.rs, nodesrc/test_resource_checker_responsibility.js"
---

# ISS-20260505T222941901Z-RESOURCE-RAW-ADDRESS-LOWERING-EXCEED-E7680697: Resource raw address lowering exceeds responsibility split limit

## 概要

After coverage_hir.rs was split, the Resource checker responsibility policy reached the next existing violation: lower_raw_address.rs has 727 lines while the split limit is 700. The module concentrates raw address return semantics, field/address source lowering, raw view construction, and named aggregate recognition.

## 対象

- `nepl-core/src/resource/lower_raw_address.rs, nepl-core/src/resource/lower.rs, nodesrc/test_resource_checker_responsibility.js`

## 根拠

- `coverage_hir.rs` の責務分割後、`node nodesrc/test_resource_checker_responsibility.js` は次の未解決責務違反として `lower_raw_address.rs has 727 lines; responsibility split limit is 700` を報告した。
- `lower_raw_address.rs` は raw address return semantics、field/address source lowering、raw view construction、named aggregate recognition を同居させており、Resource IR の raw address 境界が再び太くなり始めている。
- 関連計画: [NEPLg2 静的検査の複雑化解消計画 Stage 4](../../doc/neplg2/static_check_complexity_reduction_plan.md#stage-4-resource-check-%E3%81%B8%E3%81%AE%E7%A7%BB%E8%A1%8C)

## 問題

After coverage_hir.rs was split, the Resource checker responsibility policy reached the next existing violation: lower_raw_address.rs has 727 lines while the split limit is 700. The module concentrates raw address return semantics, field/address source lowering, raw view construction, and named aggregate recognition.

## 影響

Raw address lowering feeds the Resource IR that owner, initialized-cell, borrow, and effect checks trust. If raw address semantics remain concentrated, MemPtr as non-owning pointer and OwnedRegion/storage owner separation become harder to audit and future raw identity escapes can be introduced without a narrow boundary.

## 修正方針

Split lower_raw_address.rs by semantic role instead of raising the limit. Keep raw address lowering orchestration in lower_raw_address.rs and extract return-semantics classification, aggregate field address source classification, or named aggregate/type predicates into focused modules with responsibility policy guards.

## 検証

node nodesrc/test_resource_checker_responsibility.js; node nodesrc/run_source_policy_regressions.js --warn-only; focused Resource IR raw-address lowering tests; cargo check -p nepl-core --tests; node nodesrc/issues.js check; git diff --check

## 2026-05-06 対応結果

`lower_raw_address.rs` から raw address place/type helper を分離した。

- `lower_raw_address.rs`: core mem wrapper / user return / named raw address semantics と raw address source 推論を担当する。
- `lower_raw_address_place.rs`: `MemPtr` / `RegionToken` の raw field place、borrowed reference deref place、raw address alias target、raw-address-returnable type 判定、named struct 判定を担当する。
- `lower.rs` / `lower_aggregate.rs` / `coverage_hir_raw.rs`: `is_named_struct_type` の import を新 module へ更新した。
- `nodesrc/test_resource_checker_responsibility.js`: 新 module の存在と行数上限を固定し、`lower_raw_address.rs` 上限を 700 から 620 に下げた。

検証:

- `cargo fmt --check -p nepl-core`: passed
- `cargo check -p nepl-core --tests`: passed
- `cargo test -p nepl-core --test resource_ir raw_address -- --nocapture`: 9 passed
- `node nodesrc/test_resource_checker_responsibility.js`: `lower_raw_address.rs` 超過は解消。次の別件として `initialized_summary_apply.rs has 170 lines; responsibility split limit is 160` を検出したため、`ISS-20260505T223432842Z-RESOURCE-INITIALIZED-SUMMARY-APPLY-E-FEA66B2D` を追加した。

関連計画: [NEPLg2 静的検査の複雑化解消計画 Stage 4](../../doc/neplg2/static_check_complexity_reduction_plan.md#stage-4-resource-check-%E3%81%B8%E3%81%AE%E7%A7%BB%E8%A1%8C)
