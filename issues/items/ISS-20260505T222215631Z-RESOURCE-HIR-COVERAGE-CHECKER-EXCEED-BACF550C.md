---
id: ISS-20260505T222215631Z-RESOURCE-HIR-COVERAGE-CHECKER-EXCEED-BACF550C
title: "Resource HIR coverage checker exceeds responsibility split limit"
area: core
status: open
resolved: false
priority: P1
type: architecture
created: 2026-05-05
updated: 2026-05-05
target: "nepl-core/src/resource/coverage_hir.rs, nepl-core/src/resource/coverage.rs, nepl-core/src/resource/coverage_resource.rs, nodesrc/test_resource_checker_responsibility.js"
---

# ISS-20260505T222215631Z-RESOURCE-HIR-COVERAGE-CHECKER-EXCEED-BACF550C: Resource HIR coverage checker exceeds responsibility split limit

## 概要

After owner_return.rs was split, the Resource checker responsibility policy reached the next existing violation: coverage_hir.rs has 463 lines while the split limit is 420. The module now concentrates HIR coverage traversal, typed lowering comparison, expression/source classification, and coverage diagnostic construction.

## 対象

- `nepl-core/src/resource/coverage_hir.rs, nepl-core/src/resource/coverage.rs, nepl-core/src/resource/coverage_resource.rs, nodesrc/test_resource_checker_responsibility.js`

## 根拠

- `owner_return.rs` の責務分割後、`node nodesrc/test_resource_checker_responsibility.js` は次の未解決責務違反として `coverage_hir.rs has 463 lines; responsibility split limit is 420` を報告した。
- `node nodesrc/run_source_policy_regressions.js --warn-only` でも同じ違反を warning として確認した。downstream policy は継続実行されるが、Resource IR coverage boundary の設計負債として残る。
- 関連計画: [NEPLg2 静的検査の複雑化解消計画 Stage 4](../../doc/neplg2/static_check_complexity_reduction_plan.md#stage-4-resource-check-%E3%81%B8%E3%81%AE%E7%A7%BB%E8%A1%8C)

## 問題

After owner_return.rs was split, the Resource checker responsibility policy reached the next existing violation: coverage_hir.rs has 463 lines while the split limit is 420. The module now concentrates HIR coverage traversal, typed lowering comparison, expression/source classification, and coverage diagnostic construction.

## 影響

HIR/resource coverage is the audit layer that proves Resource IR lowering still covers the source constructs used by owner, borrow, lifetime, and effect checks. If coverage_hir.rs keeps growing, static-check completeness regressions become harder to review and future Resource IR changes can lose coverage without a focused boundary.

## 修正方針

Split coverage_hir.rs by coverage responsibility instead of raising the limit. Keep HIR traversal orchestration in coverage_hir.rs and extract expression/source classification or diagnostic construction into focused modules with explicit source-policy guards.

## 検証

node nodesrc/test_resource_checker_responsibility.js; node nodesrc/run_source_policy_regressions.js --warn-only; focused Resource IR coverage tests; cargo check -p nepl-core --tests; node nodesrc/issues.js check; git diff --check
