---
id: ISS-20260522T085841517Z-CORE-MEM-MARKER-AUTHORITY-BOUNDARY-N-D314DCEE
title: "core/mem marker authority boundary needs typed negative regression"
area: core
status: open
resolved: false
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
