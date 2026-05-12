---
id: ISS-20260512T064001916Z-RESOURCE-INITIALIZED-ALIAS-I32-FACTS-4C84FD0F
title: "Resource initialized alias i32 facts exceeds split limit"
area: core
status: open
resolved: false
priority: P1
type: architecture
created: 2026-05-12
updated: 2026-05-12
target: "nepl-core/src/resource/initialized_alias_i32_facts.rs, nodesrc/test_resource_checker_responsibility.js"
---

# ISS-20260512T064001916Z-RESOURCE-INITIALIZED-ALIAS-I32-FACTS-4C84FD0F: Resource initialized alias i32 facts exceeds split limit

## 概要

After splitting initialized_alias helpers, nodesrc/test_resource_checker_responsibility.js reaches the next blocker: initialized_alias_i32_facts.rs has 318 lines while the responsibility split limit is 180. Direct-call i32 facts and related scalar fact handling have outgrown the module budget.

## 対象

- `nepl-core/src/resource/initialized_alias_i32_facts.rs, nodesrc/test_resource_checker_responsibility.js`

## 根拠

- 未記入

## 問題

After splitting initialized_alias helpers, nodesrc/test_resource_checker_responsibility.js reaches the next blocker: initialized_alias_i32_facts.rs has 318 lines while the responsibility split limit is 180. Direct-call i32 facts and related scalar fact handling have outgrown the module budget.

## 影響

i32 fact propagation is used by Resource IR to prove memory safety around computed raw sizes and variant conditions. A large module makes the exact fact responsibilities harder to audit.

## 修正方針

Do not raise the limit. Split direct-call i32 fact recording into focused modules, likely scale/difference/relation or call-target helper modules, then update source policy.

## 検証

Run node nodesrc/test_resource_checker_responsibility.js, node nodesrc/run_source_policy_regressions.js --warn-only, cargo check -p nepl-core --tests, relevant resource_ir tests, node nodesrc/issues.js check, and git diff --check.
