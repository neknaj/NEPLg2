---
id: ISS-20260516T041311764Z-STAGE-6-MEMORY-MODEL-DOC-STILL-DESCR-AEC8348B
title: "Stage 6 memory model doc still describes removed alloc_ptr owner path"
area: docs
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-16
updated: 2026-05-16
target: doc/neplg2/stdlib_collection_mem_string_static_safety_design.md
---

# ISS-20260516T041311764Z-STAGE-6-MEMORY-MODEL-DOC-STILL-DESCR-AEC8348B: Stage 6 memory model doc still describes removed alloc_ptr owner path

## 概要

stdlib collection/memory safety design document still states that core/mem exposes alloc_ptr/dealloc_ptr and that low-level scratch implementations should explicitly import core/mem/pointer/alloc, even though Stage 6 removed the direct MemPtr owner API. The document can mislead future work back toward a deleted owner-carrier design.

## 対象

- `doc/neplg2/stdlib_collection_mem_string_static_safety_design.md`

## 根拠

- `doc/neplg2/stdlib_collection_mem_string_static_safety_design.md` の進捗表で、`core/mem` が `alloc_ptr` / `dealloc_ptr` をまだ保持していると説明していた。
- 同文書の 2026-05-16 追記で、`alloc_ptr<T> -> Result<MemPtr<T>, str>` / `realloc_ptr<T>` / `dealloc_ptr<T>` が public API として残ることを「実装対象」として書いたままだった。
- 判定節で「低レベル scratch 実装は `core/mem/pointer/alloc` を明示 import する」と説明していたが、実装では `stdlib/core/mem/pointer/alloc.nepl` は削除済みである。
- 一方で `doc/neplg2/static_check_complexity_reduction_plan.md` と現行 stdlib は、direct import 可能な `MemPtr<T>` owner API を削除済みとして扱っている。

## 問題

stdlib collection/memory safety design document still states that core/mem exposes alloc_ptr/dealloc_ptr and that low-level scratch implementations should explicitly import core/mem/pointer/alloc, even though Stage 6 removed the direct MemPtr owner API. The document can mislead future work back toward a deleted owner-carrier design.

## 影響

Developers may reintroduce direct MemPtr allocation owner APIs or plan stdlib migrations against an obsolete boundary, weakening the MemPtr = non-owning pointer and RegionToken/OwnedBuffer = free obligation owner split.

## 修正方針

Update the design document to state that alloc_ptr/realloc_ptr/dealloc_ptr are removed from public and direct import surfaces, scratch storage must use RegionToken/owned region boundaries, and remaining work is compiler-issued OwnedBuffer/initialized cell/drop traversal rather than direct low-level import cleanup.

## 検証

Search doc/neplg2/stdlib_collection_mem_string_static_safety_design.md for stale core/mem/pointer/alloc or alloc_ptr public-owner wording; run node nodesrc/issues.js check --dir issues.

## 2026-05-16 Agent 1 修正

`doc/neplg2/stdlib_collection_mem_string_static_safety_design.md` を、2026-05-16 時点の Stage 6 実装へ同期した。`core/mem` の進捗表は、`MemPtr<T>` が non-owning projection、`RegionToken<T>` が free obligation owner、public / direct import 可能な `alloc_ptr` / `realloc_ptr` / `dealloc_ptr` は削除済みであることを明記した。

2026-05-16 追記と判定節から、低レベル scratch 実装が `core/mem/pointer/alloc` を明示 import するという古い方針を削除した。現在の方針は、scratch storage も `RegionToken` owner と `region_ptr` 由来の non-owning ABI view に分け、残る作業は forgeable `RegionToken` を `OwnedBytes` / `OwnedBuffer` / compiler-issued owner token へ置き換えることとして整理した。

検証:

- `rg -n '低レベル scratch 実装は `core/mem/pointer/alloc`|実装対象にする|`MemPtr<T>` / `RegionToken<T>` / `alloc_ptr`|raw `i32` API と typed wrapper が同じ module' doc/neplg2/stdlib_collection_mem_string_static_safety_design.md`: stale wording 0 件
- `rg -n 'direct import 可能な `core/mem/pointer/alloc` module 自体も削除済み|public / direct import 可能な `alloc_ptr` / `realloc_ptr` / `dealloc_ptr` は削除済み' doc/neplg2/stdlib_collection_mem_string_static_safety_design.md`: current wording 2 件
- `node nodesrc/issues.js check --dir issues`
