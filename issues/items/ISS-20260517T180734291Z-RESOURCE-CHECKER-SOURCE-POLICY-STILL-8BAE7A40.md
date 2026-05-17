---
id: ISS-20260517T180734291Z-RESOURCE-CHECKER-SOURCE-POLICY-STILL-8BAE7A40
title: "resource checker source policy still expects removed field accessor name classifier"
area: TEST
status: open
resolved: false
priority: P1
type: test
created: 2026-05-17
updated: 2026-05-17
target: "nodesrc/test_resource_checker_responsibility.js, nepl-core/src/resource/lower.rs"
---

# ISS-20260517T180734291Z-RESOURCE-CHECKER-SOURCE-POLICY-STILL-8BAE7A40: resource checker source policy still expects removed field accessor name classifier

## 概要

nodesrc/test_resource_checker_responsibility.js still requires FieldAccessorKind::from_call_base_name in resource/lower.rs, even though direct call-name based field accessor classification was removed to avoid treating ordinary get/get_ref calls as proof evidence.

## 対象

- `nodesrc/test_resource_checker_responsibility.js, nepl-core/src/resource/lower.rs`

## 根拠

- 未記入

## 問題

nodesrc/test_resource_checker_responsibility.js still requires FieldAccessorKind::from_call_base_name in resource/lower.rs, even though direct call-name based field accessor classification was removed to avoid treating ordinary get/get_ref calls as proof evidence.

## 影響

Running the source policy registry fails on main and can pressure developers to reintroduce an unsound name-based classifier instead of the Resource IR/source-capability proof boundary.

## 修正方針

Rewrite the source policy to enforce the new proof model: ordinary direct calls must not be classified as field accessors, transparent raw-address return proof evidence must remain explicit, and the removed from_call_base_name classifier must stay absent.

## 検証

Run node nodesrc/test_resource_checker_responsibility.js and node nodesrc/run_source_policy_regressions.js after the policy update.
