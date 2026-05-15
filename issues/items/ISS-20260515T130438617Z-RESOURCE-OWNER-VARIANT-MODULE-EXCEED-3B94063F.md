---
id: ISS-20260515T130438617Z-RESOURCE-OWNER-VARIANT-MODULE-EXCEED-3B94063F
title: "Resource owner_variant module exceeds responsibility split limit again"
area: core
status: open
resolved: false
priority: P1
type: architecture
created: 2026-05-15
updated: 2026-05-15
target: "nepl-core/src/resource/owner_variant.rs, nodesrc/test_resource_checker_responsibility.js"
---

# ISS-20260515T130438617Z-RESOURCE-OWNER-VARIANT-MODULE-EXCEED-3B94063F: Resource owner_variant module exceeds responsibility split limit again

## 概要

After splitting owner_return_apply extent helpers, nodesrc/test_resource_checker_responsibility.js reaches the next blocker: owner_variant.rs has 871 lines while the enforced limit is 840. Variant match/materialization application has grown again after the previous lifecycle and record splits.

## 対象

- `nepl-core/src/resource/owner_variant.rs, nodesrc/test_resource_checker_responsibility.js`

## 根拠

- 未記入

## 問題

After splitting owner_return_apply extent helpers, nodesrc/test_resource_checker_responsibility.js reaches the next blocker: owner_variant.rs has 871 lines while the enforced limit is 840. Variant match/materialization application has grown again after the previous lifecycle and record splits.

## 影響

owner_variant.rs is a memory-safety authority for enum payload owner transfer and pending variant owner effects. Letting it grow past the policy limit makes it harder to audit exhaustive state transitions and increases the chance of mixing lifecycle, condition, and materialization rules again.

## 修正方針

Audit owner_variant.rs for the newly accumulated responsibility, split a coherent part into a focused module without changing semantics, lower/keep policy limits so future growth is caught, and preserve ResourceIR owner variant regressions.

## 検証

Run cargo fmt -p nepl-core --check, focused owner variant ResourceIR tests, nodesrc/test_resource_checker_responsibility.js, source policy warn-only, issues check, and diff whitespace check.
