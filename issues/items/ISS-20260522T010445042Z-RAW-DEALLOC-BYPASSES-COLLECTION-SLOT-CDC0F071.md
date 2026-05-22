---
id: ISS-20260522T010445042Z-RAW-DEALLOC-BYPASSES-COLLECTION-SLOT-CDC0F071
title: "Raw dealloc bypasses collection slot state proof"
area: compiler/resource-ir
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-22
updated: 2026-05-22
target: "nepl-core/src/resource/initialized_raw_memory.rs; nepl-core/src/resource/collection_slot_state_release_alias.rs; nepl-core/tests/resource_ir.rs"
---

# ISS-20260522T010445042Z-RAW-DEALLOC-BYPASSES-COLLECTION-SLOT-CDC0F071: Raw dealloc bypasses collection slot state proof

## 概要

RawMemoryOp::Dealloc certifies raw storage release and clears raw cells without consulting CollectionSlotStateTable. A collection slot can remain Initialized after the raw cell was loaded/dropped, letting storage be released without the generic collection-slot StorageDealloc proof.

## 対象

- `nepl-core/src/resource/initialized_raw_memory.rs; nepl-core/src/resource/collection_slot_state_release_alias.rs; nepl-core/tests/resource_ir.rs`

## 根拠

- `RawMemoryOp::Dealloc` は raw cell の live non-Copy 検査後に `PendingRawReallocs::certify_release` を発行していたが、同じ storage 配下の `CollectionSlotStateTable` を参照していなかった。
- Resource IR 上では raw cell を `Load` して `Drop` すると raw cell 側の live state は消える一方、`CollectionSlotLifecycle::MoveOut` / `DropInitialized` を通していない collection slot は `Initialized` のまま残る。
- この状態で raw dealloc が成立すると、non-Copy collection slot が live のまま storage release proof だけが成立してしまう。

## 問題

RawMemoryOp::Dealloc certifies raw storage release and clears raw cells without consulting CollectionSlotStateTable. A collection slot can remain Initialized after the raw cell was loaded/dropped, letting storage be released without the generic collection-slot StorageDealloc proof.

## 影響

Non-Copy collection storage can be freed while Resource IR still proves initialized collection slots exist. This undermines memory safety and would make later Vec non-Copy support depend on stdlib discipline instead of compiler-owned proof.

## 修正方針

Make raw dealloc release consult the generic collection slot state table using raw address aliases, reject live or maybe-live slots, and only certify raw release when both raw cells and collection slots satisfy release preconditions. Add regression coverage for a moved raw cell with still-initialized collection slot.

## 検証

cargo test -p nepl-core --test resource_ir raw_dealloc -- --nocapture; cargo check -p nepl-core

## 対応

- raw dealloc 時に `CollectionSlotStateTable` を alias-aware に参照し、同じ storage 配下に collection slot state が存在する場合は storage release precondition を必ず適用するようにした。
- live / maybe-live / range proof required の collection slot が残っている場合、raw release proof は発行せず `CollectionSlotRefuted` として拒否する。
- slot 側で `DropInitialized` などの汎用 Resource IR 証明が済んでいる場合だけ raw dealloc で collection slot state を retired にする。
- 明示的な `CollectionSlotLifecycle::StorageDealloc` と raw dealloc の両方が同じ refutation を発見する経路で、完全同一 Resource 診断を重複表示しないようにした。

## 実施した検証

- `cargo fmt --check -p nepl-core`
- `cargo test -p nepl-core --test resource_ir resource_ir_raw_dealloc -- --nocapture`
- `cargo test -p nepl-core --test resource_ir resource_ir_collection_storage_dealloc -- --nocapture`
- `cargo check -p nepl-core`
