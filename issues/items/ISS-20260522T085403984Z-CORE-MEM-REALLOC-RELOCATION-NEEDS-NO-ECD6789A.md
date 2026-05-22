---
id: ISS-20260522T085403984Z-CORE-MEM-REALLOC-RELOCATION-NEEDS-NO-ECD6789A
title: "core/mem realloc relocation needs non-Copy initialized slot regression"
area: core
status: fixed
resolved: true
priority: P1
type: test
created: 2026-05-22
updated: 2026-05-22
target: nepl-core/tests/resource_ir.rs
---

# ISS-20260522T085403984Z-CORE-MEM-REALLOC-RELOCATION-NEEDS-NO-ECD6789A: core/mem realloc relocation needs non-Copy initialized slot regression

## 概要

ISS-20260522T081343069Z fixed the core/mem realloc relocation proof, but review found the regression suite only covers Copy Vec grow and storage-only DropPayload realloc. It does not prove a non-Copy initialized collection slot is rekeyed from old storage to new storage through realloc_region_bytes_keep<T>.

## 対象

- `nepl-core/tests/resource_ir.rs`

## 根拠

- 未記入

## 問題

ISS-20260522T081343069Z fixed the core/mem realloc relocation proof, but review found the regression suite only covers Copy Vec grow and storage-only DropPayload realloc. It does not prove a non-Copy initialized collection slot is rekeyed from old storage to new storage through realloc_region_bytes_keep<T>.

## 影響

A future change could keep storage-only realloc passing while dropping initialized non-Copy collection slot state across reallocation, weakening memory safety coverage for the self-hosting prerequisites.

## 修正方針

Add a source-level Resource IR regression that allocates RegionToken<DropPayload>, stores a DropPayload into a collection-managed slot, emits collection_slot_initialize_empty, calls public realloc_region_bytes_keep<DropPayload>, then drops/traverses/deallocates via the grown token.

## 検証

Focused nepl-core resource_ir initialized check must pass without CollectionSlotRefuted diagnostics and must show core/mem private realloc emits CollectionStorageRelocate.

## 2026-05-22 Agent 1 解決メモ

`resource_ir_initialized_check_realloc_region_rekeys_noncopy_initialized_slot` を追加した。source-level fixture は stdlib collection 実装境界として読み込み、`RegionToken<DropPayload>` を確保して `DropPayload` を raw store し、`collection_slot_initialize_empty` で initialized slot state を作る。その後 public `realloc_region_bytes_keep<DropPayload>` を通し、success branch の grown storage に対して actual `Drop::drop` と `collection_slot_drop_traversal` を実行してから `dealloc_region` する。

このテストにより、storage-only realloc ではなく「non-Copy initialized collection slot state が old storage から new storage へ rekey されること」を Resource IR initialized check で固定した。`realloc_region_bytes_keep_relocating` に `CollectionStorageRelocate` が存在すること、cleanup 側に `CollectionSlotDropTraversal` が存在することも typed enum の ResourceOp で確認している。

focused verification:

- `cargo test -p nepl-core --test resource_ir resource_ir_initialized_check_realloc_region_rekeys_noncopy_initialized_slot -- --test-threads=1 --exact --nocapture`: passed
