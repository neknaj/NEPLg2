---
id: ISS-20260507T011907998Z-RESOURCE-INITIALIZED-ALIAS-FLOW-EXCE-E65684BD
title: "Resource initialized alias flow exceeds responsibility split policy"
area: core
status: open
resolved: false
priority: P1
type: architecture
created: 2026-05-07
updated: 2026-05-07
target: "nepl-core/src/resource/initialized_alias_flow.rs; nodesrc/test_resource_checker_responsibility.js"
---

# ISS-20260507T011907998Z-RESOURCE-INITIALIZED-ALIAS-FLOW-EXCE-E65684BD: Resource initialized alias flow exceeds responsibility split policy

## 概要

After splitting owner summary leaf responsibilities, the Resource checker policy reveals initialized_alias_flow.rs at 1034 lines against a 550-line limit, concentrating raw alias summary flow, call application, branch/return propagation, and scalar relation handling.

## 対象

- `nepl-core/src/resource/initialized_alias_flow.rs; nodesrc/test_resource_checker_responsibility.js`

## 根拠

- `ISS-20260507T011238860Z-RESOURCE-OWNER-SUMMARY-LEAF-EXCEEDS--EE0957DE` の分割後、`node nodesrc/test_resource_checker_responsibility.js` が次の未解決超過として検出した。
- `initialized_alias_flow.rs has 1034 lines; responsibility split limit is 550`
- `node nodesrc/run_source_policy_regressions.js --warn-only` でも同じ warning が残り、downstream CI は継続するが ResourceIR source policy debt として残っている。

## 問題

After splitting owner summary leaf responsibilities, the Resource checker policy reveals initialized_alias_flow.rs at 1034 lines against a 550-line limit, concentrating raw alias summary flow, call application, branch/return propagation, and scalar relation handling.

## 影響

Initialized raw alias flow can become another monolithic static-check module, making raw pointer provenance and initialized-cell proof propagation harder to audit and increasing the risk of memory-safety regressions hidden inside broad helper code.

## 修正方針

Split initialized_alias_flow.rs by raw alias return summary construction, direct/indirect call application, branch/return propagation, and scalar relation flow without raising the policy limit.

## 検証

Run node nodesrc/test_resource_checker_responsibility.js after the split and focused initialized alias/resource tests.
