---
id: ISS-20260521T062752580Z-COLLECTION-SLOT-LIFECYCLE-LOWERING-I-653BBF1A
title: "Collection slot lifecycle lowering is not counted by coverage gate"
area: core
status: fixed
resolved: true
priority: P1
type: test
created: 2026-05-21
updated: 2026-05-21
target: "nepl-core/src/resource/coverage*.rs, nepl-core/tests/resource_ir.rs"
---

# ISS-20260521T062752580Z-COLLECTION-SLOT-LIFECYCLE-LOWERING-I-653BBF1A: Collection slot lifecycle lowering is not counted by coverage gate

## 概要

Resource lowering coverage tracks calls, raw memory, borrows, moves, and unknown places, but CollectionSlotLifecycle and CollectionStorageRelocate operations are only place-covered and not counted as compiler-owned lifecycle producers. If lowering stops emitting those ResourceOps while argument place coverage remains balanced, compare_hir_resource_lowering can miss the producer loss.

## 対象

- `nepl-core/src/resource/coverage*.rs, nepl-core/tests/resource_ir.rs`

## 根拠

- `ResourceCoverageCounts` は raw memory、borrow、move などの HIR / Resource IR 対応数を比較していたが、`ResourceOp::CollectionSlotLifecycle` と `ResourceOp::CollectionStorageRelocate` は count 対象ではなかった。
- collection slot lifecycle は non-Copy collection payload の memory safety proof を Resource IR に渡す producer であるため、place coverage だけでは producer 欠落を検出する gate として不十分だった。
- static check の producer 欠落は stdlib module allowlist や手書き annotation への退行を誘発しやすいため、coverage gate も enum / match による typed count へ載せる必要がある。

## 問題

Resource lowering coverage tracks calls, raw memory, borrows, moves, and unknown places, but CollectionSlotLifecycle and CollectionStorageRelocate operations are only place-covered and not counted as compiler-owned lifecycle producers. If lowering stops emitting those ResourceOps while argument place coverage remains balanced, compare_hir_resource_lowering can miss the producer loss.

## 影響

Non-Copy collection payload support depends on generic Resource IR lifecycle producers. Missing coverage lets a future lowering regression remove collection slot proof input without an immediate coverage diagnostic, increasing the risk of stdlib-specific allowlists or silent static-check weakening.

## 修正方針

Add typed coverage counters and ResourceCoverageKind variants for CollectionSlotLifecycle and CollectionStorageRelocate. Count compiler-owned collection lifecycle intrinsics in HIR coverage, count the corresponding ResourceOps in resource coverage, and add regression tests that remove those ResourceOps from lowered Resource IR and expect CountMismatch diagnostics.

## 対応

- `ResourceCoverageCounts` に `collection_slot_lifecycle_ops` と `collection_storage_relocates` を追加した。
- `ResourceCoverageKind` に `CollectionSlotLifecycle` と `CollectionStorageRelocate` を追加し、`push_count_diagnostics` で HIR / Resource IR の差分を typed `CountMismatch` として報告するようにした。
- HIR coverage は `CollectionSlotLifecyclePrimitive::from_intrinsic_name` を通じて compiler-owned lifecycle intrinsic を enum として分類し、storage pair を要求する primitive だけを relocate count へ分けるようにした。
- Resource coverage は `ResourceOp::CollectionSlotLifecycle` と `ResourceOp::CollectionStorageRelocate` を直接 count するようにした。
- source-level regression で lifecycle op または relocate op を Resource IR から削った場合、coverage がそれぞれ `CollectionSlotLifecycle` / `CollectionStorageRelocate` の `CountMismatch` を出すことを固定した。

## 検証

- `cargo test -p nepl-core --test resource_ir resource_ir_lowering_coverage_guards_collection_slot_lifecycle -- --test-threads=1`: passed
- `cargo test -p nepl-core coverage -- --test-threads=1`: passed
- `cargo check -p nepl-core`: passed
