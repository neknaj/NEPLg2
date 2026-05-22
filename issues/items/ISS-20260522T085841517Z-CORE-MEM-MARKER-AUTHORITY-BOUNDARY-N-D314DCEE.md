---
id: ISS-20260522T085841517Z-CORE-MEM-MARKER-AUTHORITY-BOUNDARY-N-D314DCEE
title: "core/mem marker authority boundary needs typed negative regression"
area: core
status: fixed
resolved: true
priority: P1
type: test
created: 2026-05-22
updated: 2026-05-22
target: "nepl-core/src/resource/**, nodesrc/test_stdlib_core_mem_boundary.js"
---

# ISS-20260522T085841517Z-CORE-MEM-MARKER-AUTHORITY-BOUNDARY-N-D314DCEE: core/mem marker authority boundary needs typed negative regression

## 概要

Review of ISS-20260522T081343069Z found that public marker authority is currently guarded mainly by source policy structure checks. There is no typed negative regression proving a public helper or public wrapper cannot expose collection_slot_storage_relocate authority.

## 対象

- `nepl-core/src/resource/**, nodesrc/test_stdlib_core_mem_boundary.js`

## 根拠

- 未記入

## 問題

Review of ISS-20260522T081343069Z found that public marker authority is currently guarded mainly by source policy structure checks. There is no typed negative regression proving a public helper or public wrapper cannot expose collection_slot_storage_relocate authority.

## 影響

A future refactor could accidentally move lifecycle marker authority into a public memory API surface while still passing unrelated realloc lifecycle tests, weakening the no-public-marker-authority design for memory safety.

## 修正方針

Add compiler/source capability or typed Resource IR negative tests that reject public exposure of collection_slot_storage_relocate, and prefer structural enum-based checks over long regex-only source policy where possible.

## 検証

A focused test must fail if realloc_region_bytes_keep_relocating is made pub or if public realloc_region_bytes_keep emits collection_slot_storage_relocate directly.

## 2026-05-22 Agent 1 解決メモ

`nepl-core/tests/effects.rs` に `collection_slot_storage_relocate` 専用の public authority negative regression を追加した。`pub fn` が direct に `collection_slot_storage_relocate` を発行する場合と、private helper を public wrapper から到達可能にする場合の両方で `type.collection_slot_lifecycle_boundary_restricted` を要求する。

これにより `nodesrc/test_stdlib_core_mem_boundary.js` の source policy regex だけに依存せず、compiler の SourceCapability / typed diagnostic 経路で public marker authority を拒否することを固定した。`realloc_region_bytes_keep_relocating` を public 化した場合や public `realloc_region_bytes_keep` に marker を移した場合は、この public surface negative regression と既存 source policy の双方で検出できる。

focused verification:

- `cargo test -p nepl-core --test effects collection_slot_storage_relocate_rejects_public -- --test-threads=1 --nocapture`: passed
- `cargo test -p nepl-core --test effects collection_slot_lifecycle_intrinsic_rejects_public -- --test-threads=1 --nocapture`: passed
