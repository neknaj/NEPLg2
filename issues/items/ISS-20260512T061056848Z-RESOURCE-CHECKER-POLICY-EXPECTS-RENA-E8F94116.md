---
id: ISS-20260512T061056848Z-RESOURCE-CHECKER-POLICY-EXPECTS-RENA-E8F94116
title: "Resource checker policy expects renamed i32 facts module"
area: core
status: open
resolved: false
priority: P2
type: test
created: 2026-05-12
updated: 2026-05-12
target: "nodesrc/test_resource_checker_responsibility.js, nepl-core/src/resource/mod.rs"
---

# ISS-20260512T061056848Z-RESOURCE-CHECKER-POLICY-EXPECTS-RENA-E8F94116: Resource checker policy expects renamed i32 facts module

## 概要

After remote main commit 3487e386, nepl-core/src/resource/initialized_direct_call_scalar.rs was renamed to i32_call_facts.rs and mod.rs now declares mod i32_call_facts. nodesrc/test_resource_checker_responsibility.js still asserts the old file and module name, so run_source_policy_regressions --warn-only reports a stale policy warning.

## 対象

- `nodesrc/test_resource_checker_responsibility.js, nepl-core/src/resource/mod.rs`

## 根拠

- 未記入

## 問題

After remote main commit 3487e386, nepl-core/src/resource/initialized_direct_call_scalar.rs was renamed to i32_call_facts.rs and mod.rs now declares mod i32_call_facts. nodesrc/test_resource_checker_responsibility.js still asserts the old file and module name, so run_source_policy_regressions --warn-only reports a stale policy warning.

## 影響

Source policy no longer gives a clean signal after the resource checker refactor. Real resource checker responsibility regressions can be hidden behind the stale filename failure.

## 修正方針

Update nodesrc/test_resource_checker_responsibility.js to use i32_call_facts.rs and mod i32_call_facts, keeping the same responsibility boundaries for direct-call i32 facts.

## 検証

Run node nodesrc/test_resource_checker_responsibility.js and node nodesrc/run_source_policy_regressions.js --warn-only.
