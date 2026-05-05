---
id: ISS-20260505T184012396Z-RESOURCE-IR-LOWERING-TRAVERSAL-EXCEE-8A0A5A86
title: "Resource IR lowering traversal exceeds responsibility split limit"
area: core
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-05
updated: 2026-05-06
target: "nepl-core/src/resource/lower.rs, nodesrc/test_resource_checker_responsibility.js"
---

# ISS-20260505T184012396Z-RESOURCE-IR-LOWERING-TRAVERSAL-EXCEE-8A0A5A86: Resource IR lowering traversal exceeds responsibility split limit

## 概要

After owner variant path splitting, the direct Resource checker responsibility policy now reaches nepl-core/src/resource/lower.rs and reports 1315 lines over the 1300-line limit. This indicates Resource IR lowering traversal has accumulated enough logic again that Stage 4 static-check input construction is no longer safely bounded by the existing split policy.

## 対象

- `nepl-core/src/resource/lower.rs, nodesrc/test_resource_checker_responsibility.js`

## 根拠

- `node nodesrc/test_resource_checker_responsibility.js` は owner variant path builder 分割後、次の未解決責務違反として `lower.rs has 1315 lines; responsibility split limit is 1300` を報告する。
- 既に `lower_raw_address.rs` と `lower_raw_memory.rs` は分離済みだが、`lower.rs` 本体が再び上限を超えているため、raw address 以外の lowering traversal / op construction / condition lowering / aggregate projection lowering が同居している可能性が高い。
- これは `ISS-20260430T135243330Z-RESOURCE-OWNER-VARIANT-PATH-BUILDER--87B356A8` の対応中に発覚した別責務の問題であり、owner variant path builder の修正とは別 issue として扱う。

## 問題

After owner variant path splitting, the direct Resource checker responsibility policy now reaches nepl-core/src/resource/lower.rs and reports 1315 lines over the 1300-line limit. This indicates Resource IR lowering traversal has accumulated enough logic again that Stage 4 static-check input construction is no longer safely bounded by the existing split policy.

## 影響

Resource IR lowering is the trusted input for initialized, owner, borrow, and effect checks. If lower.rs keeps growing, future MemPtr/non-owning pointer, OwnedRegion/storage owner, and Resource IR state fixes can be coupled to general HIR traversal without a focused audit boundary.

## 修正方針

Split the remaining lower.rs responsibilities by semantic role rather than raising the limit: keep traversal orchestration in lower.rs, and extract condition/variant lowering, aggregate projection lowering, and call/effect resource op construction into focused modules with source-policy guards.

## 検証

Run node nodesrc/test_resource_checker_responsibility.js, node nodesrc/run_source_policy_regressions.js --warn-only, cargo fmt --check -p nepl-core, cargo check -p nepl-core --tests, and focused Resource IR lowering/owner regression tests.

## 2026-05-06 対応結果

`lower.rs` から condition fact extraction と aggregate/raw field projection lowering を分離した。

- `lower_condition.rs`: branch condition の `ResourceConditionFact` 変換、zero comparison / boolean composition / block condition value extraction を担当する。
- `lower_aggregate.rs`: compiler field load、`get` / `get_field`、raw aggregate field source、struct/tuple field projection resolution を担当する。
- `lower.rs`: HIR traversal、scope/temporary 管理、call/effect lowering orchestration、construct lowering、direct-call iterative lowering に集中する。

分割後の行数は `lower.rs` 970 lines、`lower_aggregate.rs` 262 lines、`lower_condition.rs` 112 lines で、`lower.rs` の上限を 1300 から 1150 に下げて再肥大化を検出できるようにした。

検証:

- `cargo fmt --check -p nepl-core`: passed
- `cargo check -p nepl-core --tests`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_lowering -- --nocapture`: 9 passed / 1 failed。失敗は既存の `ShadowSameSignatureCallable` warning を `typecheck_resource_source` helper が失敗扱いする問題で、今回分離した lower aggregate tests までは通過した。
- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_transfers_aggregate_owner_descendants_returned_by_helper -- --nocapture`: passed
- `node nodesrc/test_resource_checker_responsibility.js`: `lower.rs` 超過は解消。次の別件として `initialized_alias.rs has 619 lines; responsibility split limit is 550` を検出したため、`ISS-20260505T185456921Z-RESOURCE-INITIALIZED-ALIAS-MODULE-EX-06DA441F` を追加した。
- `node nodesrc/run_source_policy_regressions.js --warn-only`: downstream policy は継続実行。`lower.rs` 超過は解消し、同じ `initialized_alias.rs` 別件を warning として確認した。
- `node nodesrc/issues.js check`: passed
- `git diff --check`: passed

関連計画: [NEPLg2 静的検査の複雑化解消計画 Stage 4](../../doc/neplg2/static_check_complexity_reduction_plan.md#stage-4-resource-check-%E3%81%B8%E3%81%AE%E7%A7%BB%E8%A1%8C)
