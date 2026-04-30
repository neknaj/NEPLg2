---
id: ISS-20260430T062912063Z-RESOURCE-CHECKER-RESPONSIBILITY-POLI-CC55287A
title: "Resource checker responsibility policy misses initialized summary variant builder"
area: ci
status: verified
resolved: true
priority: P2
type: bug
created: 2026-04-30
updated: 2026-04-30
target: "nodesrc/test_resource_checker_responsibility.js, nepl-core/src/resource/initialized_summary_variant_build.rs"
---

# ISS-20260430T062912063Z-RESOURCE-CHECKER-RESPONSIBILITY-POLI-CC55287A: Resource checker responsibility policy misses initialized summary variant builder

## 概要

After splitting variant-gated initialized summary construction into initialized_summary_variant_build.rs, nodesrc/test_resource_checker_responsibility.js still checks initialized_summary_build.rs but does not require or line-limit the new variant builder module. The strict policy can pass even if the new module grows into another concentrated Resource IR checker.

## 対象

- `nodesrc/test_resource_checker_responsibility.js, nepl-core/src/resource/initialized_summary_variant_build.rs`

## 根拠

- `initialized_summary_variant_build.rs` は `initialized_summary_build.rs` から分離された Resource IR checker module だが、responsibility policy の必須 module 一覧と行数制限に入っていなかった。
- そのため、variant-gated initialized summary の責務が再び大きくなっても `nodesrc/test_resource_checker_responsibility.js` が検出できない状態だった。

## 問題

After splitting variant-gated initialized summary construction into initialized_summary_variant_build.rs, nodesrc/test_resource_checker_responsibility.js still checks initialized_summary_build.rs but does not require or line-limit the new variant builder module. The strict policy can pass even if the new module grows into another concentrated Resource IR checker.

## 影響

The source policy no longer fully monitors the Resource IR responsibility boundary it is meant to enforce. Static-check complexity can regress without a policy failure, especially around variant-gated initialized-cell summaries.

## 修正方針

Add initialized_summary_variant_build.rs to the required resource module list, require mod initialized_summary_variant_build in resource/mod.rs, and add an explicit line-count limit for the new module.

## 検証

Run node nodesrc/test_resource_checker_responsibility.js, node nodesrc/run_source_policy_regressions.js, node nodesrc/issues.js check, and git diff --check.

実行済み:

- `node nodesrc/test_resource_checker_responsibility.js`: passed
- `node nodesrc/run_source_policy_regressions.js`: passed
- `node nodesrc/issues.js check`: passed
