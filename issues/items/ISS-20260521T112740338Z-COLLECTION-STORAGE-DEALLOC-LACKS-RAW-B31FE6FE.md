---
id: ISS-20260521T112740338Z-COLLECTION-STORAGE-DEALLOC-LACKS-RAW-B31FE6FE
title: "Collection storage dealloc lacks raw release proof"
area: compiler
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-21
updated: 2026-05-21
target: "nepl-core/src/resource/**"
---

# ISS-20260521T112740338Z-COLLECTION-STORAGE-DEALLOC-LACKS-RAW-B31FE6FE: Collection storage dealloc lacks raw release proof

## 概要

CollectionSlotLifecycleEvent::StorageDealloc can currently release collection slot state without evidence that the backing raw storage was actually deallocated. This separates the high-level collection lifecycle proof from the raw free obligation and can hide missing storage release in compiler-owned collection cleanup lowering.

## 対象

- `nepl-core/src/resource/**`

## 根拠

- `RawMemoryOp::Dealloc` は raw storage の release obligation を閉じるが、修正前の `CollectionSlotLifecycleEvent::StorageDealloc` はその成功 fact を要求せず、collection slot state だけを released にできた。
- `StorageDealloc` は `CollectionSlotStateTable` 上で live slot が残っていないことを確認していたが、backing raw storage 自体が解放されたことを Resource IR の generic proof として結び付けていなかった。
- [static_check_complexity_reduction_plan.md](../../doc/neplg2/static_check_complexity_reduction_plan.md) の Stage 6 / Resource IR 方針では、stdlib helper 名や module 固有 allowlist ではなく、source-derived fact / obligation / evidence / refutation で storage owner の release を証明する必要がある。

## 問題

CollectionSlotLifecycleEvent::StorageDealloc can currently release collection slot state without evidence that the backing raw storage was actually deallocated. This separates the high-level collection lifecycle proof from the raw free obligation and can hide missing storage release in compiler-owned collection cleanup lowering.

## 影響

Non-Copy collection payload support would be able to mark a container as storage-released after slot drops even if raw storage release did not happen. That is not a source-derived generic proof and leaves the Resource IR model unable to catch missing free obligations at the collection lifecycle boundary.

## 修正方針

Record a certified raw storage release fact when RawMemoryOp::Dealloc succeeds, require that fact before CollectionSlotLifecycleEvent::StorageDealloc mutates collection slot state, consume it atomically, and carry certified release proof through collection slot summaries.

## 対応内容

- `PendingRawReallocs` に certified raw storage release proof を追加し、`RawMemoryOp::Dealloc` 成功時だけ canonical raw address / owner-cell canonical address の release fact を発行するようにした。
- `CollectionSlotStorageReleaseProof` を追加し、`StorageDealloc` だけが raw release proof obligation を持つことを typed enum / exhaustive match で表現した。
- `CollectionSlotStateTable::storage_release_precondition` で state mutation 前に live slot 診断を確定し、その後に raw release proof を消費してから storage release state へ commit する順序にした。
- call summary は certified raw release proof を持つ `StorageDealloc` だけを replay し、callee 内の proofless storage release を caller に伝播しないようにした。

## 検証

- `cargo check -p nepl-core`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_collection_storage_dealloc -- --test-threads=1`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_collection_storage_relocate -- --test-threads=1`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_collection_slot_call_summary -- --test-threads=1`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_collection_slot_return_summary -- --test-threads=1`: passed
- `cargo test -p nepl-core collection_slot_state_table --lib -- --test-threads=1`: passed
- `cargo test -p nepl-core initialized_collection_slot --lib -- --test-threads=1`: passed
