---
id: ISS-20260515T225627980Z-RESOURCE-OWNER-VARIANT-UTILS-EXCEEDS-E7268B15
title: "resource owner variant utils exceeds responsibility split limit"
area: core
status: open
resolved: false
priority: P1
type: bug
created: 2026-05-15
updated: 2026-05-15
target: "nepl-core/src/resource/owner_variant_utils.rs, nodesrc/test_resource_checker_responsibility.js"
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
