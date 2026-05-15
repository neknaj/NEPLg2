---
id: ISS-20260515T225627980Z-RESOURCE-OWNER-VARIANT-UTILS-EXCEEDS-E7268B15
title: "resource owner variant utils exceeds responsibility split limit"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-15
updated: 2026-05-16
target: "nepl-core/src/resource/owner_variant_utils.rs, nepl-core/src/resource/owner_variant_source_list.rs, nodesrc/test_resource_checker_responsibility.js"
---

# ISS-20260515T225627980Z-RESOURCE-OWNER-VARIANT-UTILS-EXCEEDS-E7268B15: resource owner variant utils exceeds responsibility split limit

## 概要

After splitting raw owner use call summary helpers, node nodesrc/test_resource_checker_responsibility.js reaches the next Resource IR responsibility blocker: owner_variant_utils.rs has 223 lines while the enforced limit is 220. Variant owner utility logic has started to grow beyond its tight review boundary.

## 対象

- `nepl-core/src/resource/owner_variant_utils.rs, nodesrc/test_resource_checker_responsibility.js`

## 根拠

- `node nodesrc/test_resource_checker_responsibility.js` が raw owner use call helper 分割後に `owner_variant_utils.rs has 223 lines; responsibility split limit is 220` を報告した。
- `owner_variant_utils.rs` は enum payload owner state / variant path utility の周辺責務を持つため、静的検査のメモリ安全境界として小さいレビュー単位を維持する必要がある。
- 行数上限を緩めるだけでは variant owner transfer 周辺の複雑化を隠すため、次の一貫した helper 責務を module 分離する必要がある。

## 問題

After splitting raw owner use call summary helpers, node nodesrc/test_resource_checker_responsibility.js reaches the next Resource IR responsibility blocker: owner_variant_utils.rs has 223 lines while the enforced limit is 220. Variant owner utility logic has started to grow beyond its tight review boundary.

## 影響

Resource IR variant owner transfer helpers can accumulate unrelated utility logic in one module. This weakens static-check reviewability around enum payload owner state and can hide memory-safety regressions in variant handling.

## 修正方針

Inspect owner_variant_utils.rs and split the next coherent helper responsibility into a dedicated module without weakening the line limit, then register the new module in resource/mod.rs and nodesrc/test_resource_checker_responsibility.js.

## 検証

Run node nodesrc/test_resource_checker_responsibility.js, focused nepl-core owner variant ResourceIR tests, node nodesrc/issues.js check --dir issues, and git diff --check.

## 2026-05-16 修正

variant owner utility から source-list dedup helper を分離した。

- `(Place, suffix, TypeId)` の dedup / containment helper を `owner_variant_source_list.rs` へ移した。
- `owner_variant.rs` と `owner_variant_lifecycle.rs` は source-list helper を新 module から import する。
- `owner_variant_utils.rs` は owner projection source、variant consumed source、variant projection return、condition、payload bind suffix の utility に戻した。
- `nodesrc/test_resource_checker_responsibility.js` に新 module の存在、`mod` 宣言、80 行上限を追加した。

検証:

- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_preserves_branch_result_variant_owner_return -- --nocapture`: 1 passed
- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_does_not_reconsume_unconditional_variant_argument -- --nocapture`: 1 passed
- `node nodesrc/test_resource_checker_responsibility.js`: `owner_variant_utils.rs` blocker は解消。次の別 issue として `effect_return_escape.rs has 363 lines; responsibility split limit is 120` を検出したため `ISS-20260515T230145475Z-RESOURCE-EFFECT-RETURN-ESCAPE-MODULE-2ED8211B` に分離した。
