---
id: ISS-20260528T073034861Z-RESOURCE-CHECKER-RESIDUAL-LARGE-MODU-55E5FDF0
title: "Resource checker residual large modules need second responsibility split"
area: core
status: open
resolved: false
priority: P2
type: architecture
created: 2026-05-28
updated: 2026-05-28
target: "nepl-core/src/resource/*; nodesrc/test_resource_checker_responsibility.js"
---

# ISS-20260528T073034861Z-RESOURCE-CHECKER-RESIDUAL-LARGE-MODU-55E5FDF0: Resource checker residual large modules need second responsibility split

## 概要

After the current resource checker responsibility recovery, several already-large resource modules still require baseline synchronization to keep the monitor runnable: collection_slot_summary_build_ops.rs, collection_slot_summary_build_ops_tests.rs, collection_slot_summary_build_drop_traversal.rs, collection_slot_summary_return_path_value.rs, collection_slot_state_table.rs, lower_collection_slot.rs, initialized_alias_i32_condition.rs, initialized_collection_slot_dispatch.rs, and initialized_control.rs.

## 対象

- `nepl-core/src/resource/*; nodesrc/test_resource_checker_responsibility.js`

## 根拠

- 未記入

## 問題

After the current resource checker responsibility recovery, several already-large resource modules still require baseline synchronization to keep the monitor runnable: collection_slot_summary_build_ops.rs, collection_slot_summary_build_ops_tests.rs, collection_slot_summary_build_drop_traversal.rs, collection_slot_summary_return_path_value.rs, collection_slot_state_table.rs, lower_collection_slot.rs, initialized_alias_i32_condition.rs, initialized_collection_slot_dispatch.rs, and initialized_control.rs.

## 影響

The resource checker responsibility gate can pass after the monitor coverage recovery, but these modules remain close to or above their old responsibility budgets. Future static-check work may continue to accumulate implementation code in broad orchestration files unless the residual split work is planned explicitly.

## 修正方針

Split the residual large modules by proof domain and orchestration boundary instead of raising budgets further. In particular, separate collection-slot summary op construction from replay/proof helpers, split initialized_control.rs by branch/loop/match path merge responsibilities, and keep path-test modules separate from production proof logic.

## 検証

Run node nodesrc/test_resource_checker_responsibility.js, cargo check -p nepl-core, targeted resource unit tests for each split module, node nodesrc/issues.js check --dir issues, and git diff --check.
