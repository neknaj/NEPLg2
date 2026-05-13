---
id: ISS-20260513T131452642Z-RESOURCE-OWNER-VARIANT-LIFECYCLE-REM-1A524C29
title: "Resource owner variant lifecycle remains coupled to apply logic"
area: core
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-13
updated: 2026-05-13
target: "nepl-core/src/resource/owner_variant.rs; nepl-core/src/resource/owner_variant_lifecycle.rs; nodesrc/test_resource_checker_responsibility.js"
---

# ISS-20260513T131452642Z-RESOURCE-OWNER-VARIANT-LIFECYCLE-REM-1A524C29: Resource owner variant lifecycle remains coupled to apply logic

## 概要

Resource IR owner variant pending effects still keep result copy/clear/resolve, path merge, and unique-entry lifecycle management inside owner_variant.rs alongside match/materialization apply logic. This keeps non-local state lifecycle rules coupled to owner consumption/return application and makes the static-check authority harder to review.

## 対象

- `nepl-core/src/resource/owner_variant.rs; nepl-core/src/resource/owner_variant_lifecycle.rs; nodesrc/test_resource_checker_responsibility.js`

## 関連計画

- [静的検査の不必要な複雑化の解消計画 Stage 4](../../doc/neplg2/static_check_complexity_reduction_plan.md#stage-4-resource-check-への移行)

## 根拠

- `owner_variant.rs` は前段の call recording 分割後も 1037 行あり、match/materialization への owner effect 適用と、pending result の copy / clear / resolve、path merge、dedupe が同じ `impl PendingVariantOwnerEffects` に残っていた。
- pending lifecycle は branch merge と function call return の間で owner consumption / return / unreachable / value condition を保守する静的検査の中核であり、適用処理と同じ module に置くと Resource IR owner checker が再び monolithic 化する。
- `nodesrc/test_resource_checker_responsibility.js` は `owner_variant.rs` の増大を監視していたが、lifecycle 専用moduleの存在はまだ要求していなかった。

## 問題

Resource IR owner variant pending effects still keep result copy/clear/resolve, path merge, and unique-entry lifecycle management inside owner_variant.rs alongside match/materialization apply logic. This keeps non-local state lifecycle rules coupled to owner consumption/return application and makes the static-check authority harder to review.

## 影響

Future Resource IR fixes can reintroduce monolithic owner-variant logic or mutate pending lifecycle state inconsistently, increasing the risk of missed owner/provenance regressions around enum payload memory-safety checks.

## 修正方針

Split pending variant owner lifecycle, merge, and dedupe methods into a dedicated owner_variant_lifecycle module, register it in the resource policy, and lower owner_variant.rs line limit so the separation remains enforced.

## 検証

Run cargo check -p nepl-core --tests, trunk build, focused move_effect doctests, node nodesrc/test_resource_checker_responsibility.js, node nodesrc/run_source_policy_regressions.js --warn-only, and node nodesrc/issues.js check --dir issues.

## 対応内容

- `owner_variant_lifecycle.rs` を追加し、pending variant owner effects の `copy_result` / `clear_result` / `resolve_result`、path merge、variant reachability query、unique-entry push を専用moduleへ分離した。
- `owner_variant.rs` は match/materialization への owner effect 適用、owner summary/source collection、raw view/storage origin との接続に集中させた。
- Resource checker responsibility policy に `owner_variant_lifecycle.rs` の必須存在、`mod` declaration、行数上限を追加し、`owner_variant.rs` の上限を 1120 行から 840 行へ下げた。
- 分割後の行数は `owner_variant.rs` 783 行、`owner_variant_lifecycle.rs` 269 行。

## 検証結果

- `cargo check -p nepl-core --tests`: passed
- `trunk build`: passed
- `node nodesrc/tests.js -i tests/compiler/move_effect.n.md --no-tree -o tmp/agent1-owner-variant-lifecycle-move-effect.json -j 1 --dist web/dist`: total=113, passed=113
- `node nodesrc/test_resource_checker_responsibility.js`: passed
- `node nodesrc/run_source_policy_regressions.js --warn-only`: passed
- `node nodesrc/issues.js check --dir issues`: passed
