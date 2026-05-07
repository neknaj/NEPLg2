---
id: ISS-20260507T011238860Z-RESOURCE-OWNER-SUMMARY-LEAF-EXCEEDS--EE0957DE
title: "Resource owner summary leaf exceeds responsibility split policy"
area: core
status: open
resolved: false
priority: P1
type: architecture
created: 2026-05-07
updated: 2026-05-07
target: "nepl-core/src/resource/owner_summary_leaf.rs; nodesrc/test_resource_checker_responsibility.js"
---

# ISS-20260507T011238860Z-RESOURCE-OWNER-SUMMARY-LEAF-EXCEEDS--EE0957DE: Resource owner summary leaf exceeds responsibility split policy

## 概要

After splitting initialized availability checks, the Resource checker responsibility policy reveals owner_summary_leaf.rs at 387 lines against a 260-line limit, concentrating owner-summary leaf and record traversal logic.

## 対象

- `nepl-core/src/resource/owner_summary_leaf.rs; nodesrc/test_resource_checker_responsibility.js`

## 根拠

- `ISS-20260507T011054993Z-RESOURCE-INITIALIZED-AVAILABILITY-CH-ED41979E` の分割後、`node nodesrc/test_resource_checker_responsibility.js` が次の未解決超過として検出した。
- `owner_summary_leaf.rs has 387 lines; responsibility split limit is 260`
- `node nodesrc/run_source_policy_regressions.js --warn-only` でも同じ warning が残り、downstream CI は継続するが source policy debt として残っている。

## 問題

After splitting initialized availability checks, the Resource checker responsibility policy reveals owner_summary_leaf.rs at 387 lines against a 260-line limit, concentrating owner-summary leaf and record traversal logic.

## 影響

Owner obligation summaries can re-centralize in a large helper module, reducing match-based auditability of free-obligation transfer and making memory-safety regressions harder to localize.

## 修正方針

Split owner_summary_leaf.rs into coherent leaf classification and traversal/update modules without raising the policy limit, then register the new module in the responsibility policy.

## 検証

Run node nodesrc/test_resource_checker_responsibility.js after the split and focused owner-summary/resource tests.
