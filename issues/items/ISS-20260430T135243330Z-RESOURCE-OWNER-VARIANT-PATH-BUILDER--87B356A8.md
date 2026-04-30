---
id: ISS-20260430T135243330Z-RESOURCE-OWNER-VARIANT-PATH-BUILDER--87B356A8
title: "Resource owner variant path builder exceeds responsibility split policy"
area: core
status: open
resolved: false
priority: P1
type: architecture
created: 2026-04-30
updated: 2026-04-30
target: "nepl-core/src/resource/owner_summary_variant_paths.rs, nodesrc/test_resource_checker_responsibility.js"
---

# ISS-20260430T135243330Z-RESOURCE-OWNER-VARIANT-PATH-BUILDER--87B356A8: Resource owner variant path builder exceeds responsibility split policy

## 概要

Remote main expanded owner_summary_variant_paths.rs to 637 lines while source policy limit is 380. Result owner variant path enumeration, condition propagation, call effect reservation, and returned owner path collection are concentrated in one module.

## 対象

- `nepl-core/src/resource/owner_summary_variant_paths.rs, nodesrc/test_resource_checker_responsibility.js`

## 根拠

- 未記入

## 問題

Remote main expanded owner_summary_variant_paths.rs to 637 lines while source policy limit is 380. Result owner variant path enumeration, condition propagation, call effect reservation, and returned owner path collection are concentrated in one module.

## 影響

Resource checker responsibility policy fails and Resource IR owner summary logic is drifting back toward a monolithic checker. This blocks source-policy aggregate runs and makes selfhost static-check design harder to copy safely.

## 修正方針

Split owner variant path logic into smaller modules such as path collection, condition refinement, reserved effect handling, and path application. Keep owner_summary_variant_paths.rs as orchestration only and update the source policy limits after the split.

## 検証

Run node nodesrc/test_resource_checker_responsibility.js, node nodesrc/run_source_policy_regressions.js, and cargo test -p nepl-core --test resource_ir -- --nocapture.
