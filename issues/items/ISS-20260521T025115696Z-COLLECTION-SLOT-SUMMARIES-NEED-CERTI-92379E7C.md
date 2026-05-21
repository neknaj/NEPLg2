---
id: ISS-20260521T025115696Z-COLLECTION-SLOT-SUMMARIES-NEED-CERTI-92379E7C
title: "Collection slot summaries need certified owner-transfer value-flow proof"
area: core
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-21
updated: 2026-05-21
target: "nepl-core/src/resource/collection_slot_summary_*.rs, nepl-core/src/resource/initialized_collection_slot.rs"
---

# ISS-20260521T025115696Z-COLLECTION-SLOT-SUMMARIES-NEED-CERTI-92379E7C: Collection slot summaries need certified owner-transfer value-flow proof

## 概要

Collection slot lifecycle summaries replay non-Copy owner-transfer events in the caller, but the local raw StoreValue/MoveOutLoadedCell proof is consumed inside the callee and is not represented in the summary. Replaying the event without certified proof either rejects safe callee-proven lifecycle effects or tempts caller-local allowlists.

## 対象

- `nepl-core/src/resource/collection_slot_summary_*.rs, nepl-core/src/resource/initialized_collection_slot.rs`

## 根拠

- 親 issue [ISS-20260520T152218366Z-NON-COPY-COLLECTION-PAYLOAD-SUPPORT--A6A88543](./ISS-20260520T152218366Z-NON-COPY-COLLECTION-PAYLOAD-SUPPORT--A6A88543.md) は、non-Copy collection payload support を stdlib 個別 allowlist ではなく Resource IR の generic proof boundary へ載せることを要求している。
- [ISS-20260521T020307778Z-COLLECTION-SLOT-OWNER-TRANSFER-NEEDS-403A919A](./ISS-20260521T020307778Z-COLLECTION-SLOT-OWNER-TRANSFER-NEEDS-403A919A.md) で同一関数内の raw `StoreValue` / `MoveOutLoadedCell` proof は実装されたが、callee で消費済みの proof を caller summary replay へ伝える表現がまだなかった。
- [NEPLg2 静的検査の複雑化解消計画](../../doc/neplg2/static_check_complexity_reduction_plan.md) Stage 6 は、summary を含む Resource IR proof を enum / match で扱い、stdlib module 名に依存しないことを完了条件にしている。

## 問題

Collection slot lifecycle summaries replay non-Copy owner-transfer events in the caller, but the local raw StoreValue/MoveOutLoadedCell proof is consumed inside the callee and is not represented in the summary. Replaying the event without certified proof either rejects safe callee-proven lifecycle effects or tempts caller-local allowlists.

## 影響

Non-Copy collection helpers cannot safely cross function boundaries, blocking self-host collection payloads unless stdlib-specific exceptions or shallow owner transfer are reintroduced.

## 修正方針

Represent callee-proven owner-transfer evidence as a typed summary enum, build it only from Resource IR raw value-flow facts available at the lifecycle event, and make summary replay accept only matching certified proof for non-Copy owner-transfer events.

## 検証

Add Resource IR regressions for a callee that raw-stores then initializes a non-Copy slot, raw-loads then moves out a non-Copy slot, and a missing-proof summary that remains rejected; run focused collection slot resource tests and issue validation.

## 2026-05-21 修正内容

- `CollectionSlotLifecycleSummaryEventProof` を追加し、summary event を `StateOnly` と `OwnerTransferValueFlow(CollectionSlotOwnerTransferObligation)` に分けた。
- summary build は、callee 内の lifecycle event 時点で raw `StoreValue` / `MoveOutLoadedCell` proof が `CellTable` に存在する場合だけ `OwnerTransferValueFlow` を summary に載せる。proof がない non-Copy owner-transfer event は summary として信用しない。
- summary replay は `OwnerTransferValueFlow` を `SummaryCertified` proof として扱い、caller 側に同じ raw fact を再要求しない。証明種別や型が event の obligation と一致しない場合は従来どおり `OwnerTransferRequiresValueProof` で拒否する。
- `ReplaceReturnOld` は old raw load と new raw store の両方が callee 内で証明された場合だけ certified summary proof になる。
- direct Resource IR event は従来どおり `LocalRawValueFlow` proof を消費するため、callee summary proof と caller-local proof の責務を分離した。

## 2026-05-21 検証

- `cargo check -p nepl-core`: pass
- `cargo test -p nepl-core resource_ir_collection_slot --test resource_ir -- --test-threads=1`: pass
- `cargo test -p nepl-core collection_slot --lib -- --test-threads=1`: pass
- `cargo fmt --check -p nepl-core`: pass
- `git diff --check`: pass
