---
id: ISS-20260512T122547229Z-RESOURCE-SUPPORT-MODULES-ARE-MISSING-8F8975DB
title: "Resource support modules are missing responsibility policy coverage"
area: core
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-12
updated: 2026-05-12
target: "nodesrc/test_resource_checker_responsibility.js, nepl-core/src/resource/*.rs"
---

# ISS-20260512T122547229Z-RESOURCE-SUPPORT-MODULES-ARE-MISSING-8F8975DB: Resource support modules are missing responsibility policy coverage

## 概要

Several large Resource IR support modules such as cell_state.rs, owner_control.rs, initialized_control.rs, initialized_variant.rs, owner_state.rs, borrow_state.rs, and storage_origin.rs are not registered in the resource responsibility policy. They can grow or regain mixed responsibilities without source-policy signal.

## 対象

- `nodesrc/test_resource_checker_responsibility.js, nepl-core/src/resource/*.rs`

## 根拠

- `cell_state.rs`、`owner_control.rs`、`initialized_control.rs`、`initialized_variant.rs`、`owner_state.rs`、`borrow_state.rs`、`storage_origin.rs` など、Resource IR の cell / owner / borrow / storage-origin state を扱う大きな module が責務分割 policy に未登録だった。
- `nodesrc/test_resource_checker_responsibility.js` は主要 checker / summary module の存在と行数を監視しているが、support state / control-flow module の一部が抜けていた。
- Resource IR は現在の authoritative static-check path であり、support module の肥大化や責務混在も memory-safety review の難度へ直結する。

## 問題

Several large Resource IR support modules such as cell_state.rs, owner_control.rs, initialized_control.rs, initialized_variant.rs, owner_state.rs, borrow_state.rs, and storage_origin.rs are not registered in the resource responsibility policy. They can grow or regain mixed responsibilities without source-policy signal.

## 影響

Resource IR is the authoritative static-check path. Missing responsibility coverage makes it easier to recreate monolithic checker debt around cell, owner, borrow, and storage-origin state, increasing review risk for type and memory-safety fixes.

## 修正方針

Add the currently missing large Resource IR support modules to the responsibility policy with explicit line limits. This does not change checker semantics; it hardens the guardrail around future static-check edits.

## 検証

Run node nodesrc/test_resource_checker_responsibility.js and node nodesrc/issues.js check --dir issues.

## 対応結果

2026-05-12 に修正済み。

- `nodesrc/test_resource_checker_responsibility.js` の必須 module 一覧に、未監視だった大きい Resource IR support module を追加した。
- 追加対象は `borrow_state.rs`、`cell_state.rs`、`dump.rs`、`initialized_control.rs`、`initialized_variant.rs`、`model.rs`、`owner_control.rs`、`owner_drop_scope.rs`、`owner_state.rs`、`place_utils.rs`、`storage_origin.rs`。
- 各 module に明示的な行数上限を設定し、今後の static-check 修正で support module が無制限に増える回帰を source policy で検出できるようにした。
- 現時点で 220 行超かつ policy 未登録の `nepl-core/src/resource/*.rs` は 0 件であることを確認した。

検証:

- `node nodesrc/test_resource_checker_responsibility.js`: passed
- `node nodesrc/issues.js check --dir issues`: passed
