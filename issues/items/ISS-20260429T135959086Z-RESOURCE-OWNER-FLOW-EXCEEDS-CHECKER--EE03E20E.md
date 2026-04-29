---
id: ISS-20260429T135959086Z-RESOURCE-OWNER-FLOW-EXCEEDS-CHECKER--EE03E20E
title: "Resource owner_flow exceeds checker responsibility source policy limit"
area: core
status: open
resolved: false
priority: P1
type: architecture
created: 2026-04-29
updated: 2026-04-29
target: "nepl-core/src/resource/owner_flow.rs, nodesrc/test_resource_checker_responsibility.js"
---

# ISS-20260429T135959086Z-RESOURCE-OWNER-FLOW-EXCEEDS-CHECKER--EE03E20E: Resource owner_flow exceeds checker responsibility source policy limit

## 概要

GitHub Actions Source policy regressions fail because owner_flow.rs has 693 lines while nodesrc/test_resource_checker_responsibility.js enforces a 620-line responsibility split limit. This indicates Resource IR owner flow transfer/summary/raw-address responsibilities have re-concentrated in one module.

## 対象

- `nepl-core/src/resource/owner_flow.rs, nodesrc/test_resource_checker_responsibility.js`

## 根拠

- 未記入

## 問題

GitHub Actions Source policy regressions fail because owner_flow.rs has 693 lines while nodesrc/test_resource_checker_responsibility.js enforces a 620-line responsibility split limit. This indicates Resource IR owner flow transfer/summary/raw-address responsibilities have re-concentrated in one module.

## 影響

CI stops before rust/std/doctest jobs, hiding real static-check regressions. It also violates the static check complexity reduction plan by letting the Resource IR owner checker grow a new large responsibility cluster.

## 修正方針

Split owner_flow.rs by responsibility instead of raising the limit: move raw address ownership classification, call return summary application, and low-level owner transfer helpers into dedicated modules with enum-based classification and exhaustive match dispatch.

## 検証

Run node nodesrc/test_resource_checker_responsibility.js and cargo test -p nepl-core --test resource_ir resource_ir_owner_check_ -- --nocapture.
