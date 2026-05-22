---
id: ISS-20260522T012056297Z-RAW-REALLOC-REJECTS-COLLECTION-MANAG-95C9800C
title: "Raw realloc rejects collection-managed non-Copy payload relocation"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-22
updated: 2026-05-22
target: "nepl-core/src/resource/initialized_raw_memory.rs, nepl-core/src/resource/raw_cell_lifecycle.rs, nepl-core/src/resource/raw_realloc.rs, nepl-core/tests/resource_ir.rs"
source: doc/neplg2/static_check_complexity_reduction_plan.md#stage-6-stdlib--self-host-準備
---

# ISS-20260522T012056297Z-RAW-REALLOC-REJECTS-COLLECTION-MANAG-95C9800C: Raw realloc rejects collection-managed non-Copy payload relocation

## 概要

RawMemoryOp::Realloc calls the generic live non-Copy raw-cell release gate before a raw relocation proof can be certified. Even when the same raw cell is tracked as an initialized collection slot, realloc is rejected, so Vec-like storage grow cannot relocate non-Copy payloads through the generic CollectionStorageRelocate proof boundary.

## 対象

- `nepl-core/src/resource/initialized_raw_memory.rs, nepl-core/src/resource/raw_cell_lifecycle.rs, nepl-core/src/resource/raw_realloc.rs, nepl-core/tests/resource_ir.rs`

## 根拠

- [NEPLg2 静的検査の複雑化解消計画](../../doc/neplg2/static_check_complexity_reduction_plan.md) Stage 6 は、non-Copy collection payload を stdlib module allowlist ではなく Resource IR の raw provenance / initialized cell / collection slot state の generic proof boundary で扱うことを要求している。
- [ISS-20260520T152218366Z-NON-COPY-COLLECTION-PAYLOAD-SUPPORT--A6A88543](./ISS-20260520T152218366Z-NON-COPY-COLLECTION-PAYLOAD-SUPPORT--A6A88543.md) は、`Vec<T>` の non-Copy `push` / `clear` / `free` / grow を compiler-owned Resource IR marker と owner-preserving API 型に接続することを残件にしている。
- [ISS-20260521T105928164Z-COLLECTION-STORAGE-RELOCATE-LACKS-RA-BE2DCD3E](./ISS-20260521T105928164Z-COLLECTION-STORAGE-RELOCATE-LACKS-RA-BE2DCD3E.md) で `CollectionStorageRelocate` は raw realloc success proof を要求するようになったが、その前段の `RawMemoryOp::Realloc` が collection-managed non-Copy raw cell を拒否していた。

## 問題

RawMemoryOp::Realloc calls the generic live non-Copy raw-cell release gate before a raw relocation proof can be certified. Even when the same raw cell is tracked as an initialized collection slot, realloc is rejected, so Vec-like storage grow cannot relocate non-Copy payloads through the generic CollectionStorageRelocate proof boundary.

## 影響

Non-Copy Vec grow and future self-host collection storage relocation remain blocked, or stdlib authors would be forced toward Vec-specific allowlists / helper conventions instead of source-derived Resource IR proof.

## 修正方針

Keep raw destructive operations strict by default, but for RawMemoryOp::Realloc only accept live non-Copy raw cells when CollectionSlotStateTable proves that the exact raw cell is an initialized or maybe-initialized collection slot. Store that certified raw-cell set in PendingRawRealloc and, on realloc success, rekey only those non-Copy raw cells together with copy raw cells. CollectionStorageRelocate must still consume the raw movement proof separately.

## 検証

Add Resource IR regression where raw store + CollectionSlotLifecycle::InitializeEmpty over a non-Copy payload is followed by realloc success, CollectionStorageRelocate, and storage dealloc. The expected diagnostic should be the live relocated collection slot, not RawMemoryReallocCell or missing raw movement proof. Keep existing destructive raw storage operation tests rejecting unmanaged live non-Copy cells.

## 対応結果

2026-05-22 に修正済み。

- `RawMemoryOp::Realloc` の live non-Copy raw-cell gate を、通常の破壊的 raw operation と分離した。
- unmanaged non-Copy raw cell は従来通り `RawMemoryReallocCell` で拒否する一方、同じ raw cell が `CollectionSlotStateTable` 上で `Initialized` / `MaybeInitialized` として追跡されている場合だけ、collection-managed raw cell として pending realloc proof に記録する。
- `PendingRawRealloc` は certified collection-managed non-Copy raw cell の一覧を保持し、realloc success path の `RawCellLifecycleEvent::ReallocSuccessTransfer` は Copy raw cell / raw byte range に加えて、その certified non-Copy raw cell だけを new storage へ rekey する。
- `CollectionStorageRelocate` はこれまで通り raw movement proof を別途消費するため、proofless relocate や stdlib helper 名 allowlist は追加していない。

## 回帰テスト

- `resource_ir_collection_storage_relocate_accepts_live_non_copy_payload_after_realloc`
- `resource_ir_realloc_rekeys_collection_managed_non_copy_raw_cell`
- `resource_ir_cell_check_reports_destructive_raw_storage_ops_over_live_cell`
