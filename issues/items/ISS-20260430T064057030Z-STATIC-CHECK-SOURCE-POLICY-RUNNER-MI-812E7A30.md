---
id: ISS-20260430T064057030Z-STATIC-CHECK-SOURCE-POLICY-RUNNER-MI-812E7A30
title: "Static-check source policy runner misses Resource IR and self-host safety policies"
area: ci
status: verified
resolved: true
priority: P1
type: bug
created: 2026-04-30
updated: 2026-04-30
target: "nodesrc/run_source_policy_regressions.js, nodesrc/test_resource_ir_test_harness_policy.js, nodesrc/test_selfhost_*"
---

# ISS-20260430T064057030Z-STATIC-CHECK-SOURCE-POLICY-RUNNER-MI-812E7A30: Static-check source policy runner misses Resource IR and self-host safety policies

## 概要

Several source-policy scripts that guard static-check correctness are not included in nodesrc/run_source_policy_regressions.js. In particular, Resource IR test harness policy and self-host diagnostic/owner/boundary policies can pass locally when run directly but are not covered by the CI source-policy aggregate step.

## 対象

- `nodesrc/run_source_policy_regressions.js, nodesrc/test_resource_ir_test_harness_policy.js, nodesrc/test_selfhost_*`

## 根拠

- `nodesrc/test_resource_ir_test_harness_policy.js` は Resource IR tests が `lower_hir_module_skeleton` に戻らないことを監視するが、aggregate source-policy runner に含まれていなかった。
- self-host の `SelfhostOutcome` raw result cell 禁止、diagnostic code enum 化、CLI args/driver/file_io/reporter boundary、source text line map、string helper boundary の policy が aggregate runner に含まれていなかった。
- `nodesrc/test_stdlib_nm_no_raw_aggregate_detours.js` と `nodesrc/test_stdlib_byte_scanner_helpers_boundary.js` も、stdlib/self-host が raw aggregate decomposition や scanner facade へ戻らないための source policy だが aggregate runner から漏れていた。

## 問題

Several source-policy scripts that guard static-check correctness are not included in nodesrc/run_source_policy_regressions.js. In particular, Resource IR test harness policy and self-host diagnostic/owner/boundary policies can pass locally when run directly but are not covered by the CI source-policy aggregate step.

## 影響

A regression can reintroduce skeleton Resource IR lowering in tests, raw Result cells in self-host outcomes, raw string diagnostic codes, or self-host boundary violations without failing the standard source-policy run. That weakens the static-check design contract and makes type/memory safety regressions easier to miss.

## 修正方針

Add the static-check and self-host safety source-policy scripts to the aggregate source-policy runner, keeping functional or slow standalone runner tests out of this aggregate unless they are pure source-policy checks.

## 検証

Run the newly added policy scripts directly, node nodesrc/run_source_policy_regressions.js, node nodesrc/issues.js check, and git diff --check.

実行済み:

- 追加した 12 個の source-policy script を直接実行: passed
- `node nodesrc/run_source_policy_regressions.js`: passed
