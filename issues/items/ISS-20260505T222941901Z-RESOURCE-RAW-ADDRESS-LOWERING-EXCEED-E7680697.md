---
id: ISS-20260505T222941901Z-RESOURCE-RAW-ADDRESS-LOWERING-EXCEED-E7680697
title: "Resource raw address lowering exceeds responsibility split limit"
area: core
status: open
resolved: false
priority: P1
type: architecture
created: 2026-05-05
updated: 2026-05-05
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
