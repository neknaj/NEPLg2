---
id: ISS-20260505T185456921Z-RESOURCE-INITIALIZED-ALIAS-MODULE-EX-06DA441F
title: "Resource initialized alias module exceeds responsibility split limit again"
area: core
status: open
resolved: false
priority: P1
type: architecture
created: 2026-05-05
updated: 2026-05-05
target: "nepl-core/src/resource/initialized_alias.rs, nepl-core/src/resource/initialized_alias_flow.rs, nodesrc/test_resource_checker_responsibility.js"
---

# ISS-20260505T185456921Z-RESOURCE-INITIALIZED-ALIAS-MODULE-EX-06DA441F: Resource initialized alias module exceeds responsibility split limit again

## 概要

After resolving the Resource IR lowering traversal split, the direct Resource checker responsibility policy now reaches initialized_alias.rs and reports 619 lines over the 550-line limit. Raw alias table logic has grown again after prior alias-flow extraction, so initialized alias canonicalization and summary support need another semantic split instead of raising the limit.

## 対象

- `nepl-core/src/resource/initialized_alias.rs, nepl-core/src/resource/initialized_alias_flow.rs, nodesrc/test_resource_checker_responsibility.js`

## 根拠

- `node nodesrc/test_resource_checker_responsibility.js` は `lower.rs` 分割後、次の未解決責務違反として `initialized_alias.rs has 619 lines; responsibility split limit is 550` を報告する。
- `ISS-20260428T180803802Z-RESOURCE-INITIALIZED-RAW-ALIAS-LOGIC-E8D87FFA` で raw alias flow は一度 `initialized_alias_flow.rs` へ分離済みだが、その後の unknown-offset alias / aggregate field alias / MemPtr raw view 対応により `initialized_alias.rs` 本体が再び上限を超えた。
- Stage 4 Resource check では initialized alias fact が CellState と owner check の根拠になるため、table API、canonicalization、projection/query helper、summary support が同じ module に戻ると memory-safety audit が難しくなる。

## 問題

After resolving the Resource IR lowering traversal split, the direct Resource checker responsibility policy now reaches initialized_alias.rs and reports 619 lines over the 550-line limit. Raw alias table logic has grown again after prior alias-flow extraction, so initialized alias canonicalization and summary support need another semantic split instead of raising the limit.

## 影響

Initialized raw alias facts feed CellState, owner checks, and raw memory safety diagnostics. If initialized_alias.rs keeps growing, changes to unknown-offset aliases, MemPtr raw views, and aggregate field aliases become harder to audit and can hide memory-safety regressions.

## 修正方針

Review initialized_alias.rs by responsibility and extract the next coherent boundary, such as raw alias canonicalization, alias query/projection helpers, or summary/call application helpers, while keeping initialized_alias.rs focused on the RawCellAddressAliases table API and preserving source-policy guards.

## 検証

Run node nodesrc/test_resource_checker_responsibility.js, node nodesrc/run_source_policy_regressions.js --warn-only, cargo fmt --check -p nepl-core, cargo check -p nepl-core --tests, and focused Resource IR raw alias/cell state tests.
