---
id: ISS-20260512T062230660Z-RESOURCE-OWNER-SUMMARY-VARIANT-CONDI-F79EFC3E
title: "Resource owner summary variant conditions exceeds split limit"
area: core
status: open
resolved: false
priority: P1
type: architecture
created: 2026-05-12
updated: 2026-05-12
target: "nepl-core/src/resource/owner_summary_variant_conditions.rs, nodesrc/test_resource_checker_responsibility.js"
---

# ISS-20260512T062230660Z-RESOURCE-OWNER-SUMMARY-VARIANT-CONDI-F79EFC3E: Resource owner summary variant conditions exceeds split limit

## 概要

After splitting owner_check utilities, nodesrc/test_resource_checker_responsibility.js reaches the next blocker: owner_summary_variant_conditions.rs has 295 lines while the responsibility split limit is 260. Branch condition conversion and payload/value condition handling have grown inside one module.

## 対象

- `nepl-core/src/resource/owner_summary_variant_conditions.rs, nodesrc/test_resource_checker_responsibility.js`

## 根拠

- 未記入

## 問題

After splitting owner_check utilities, nodesrc/test_resource_checker_responsibility.js reaches the next blocker: owner_summary_variant_conditions.rs has 295 lines while the responsibility split limit is 260. Branch condition conversion and payload/value condition handling have grown inside one module.

## 影響

Owner variant summary condition handling is accumulating multiple condition-conversion responsibilities in one file, making the memory-safety summary logic harder to audit and weakening the responsibility split policy.

## 修正方針

Do not raise the limit. Split value/payload condition conversion helpers or shared condition utilities out of owner_summary_variant_conditions.rs, update mod.rs and source policy, and keep variant condition semantics covered by existing resource_ir tests.

## 検証

Run node nodesrc/test_resource_checker_responsibility.js, node nodesrc/run_source_policy_regressions.js --warn-only, cargo check -p nepl-core --tests, relevant resource_ir tests, node nodesrc/issues.js check, and git diff --check.
