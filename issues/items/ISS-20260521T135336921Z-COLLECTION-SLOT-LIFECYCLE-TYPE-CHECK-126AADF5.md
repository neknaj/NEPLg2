---
id: ISS-20260521T135336921Z-COLLECTION-SLOT-LIFECYCLE-TYPE-CHECK-126AADF5
title: "Collection slot lifecycle type checks conflate slot identity and payload TypeId"
area: compiler-core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-21
updated: 2026-05-21
target: "nepl-core/src/resource/collection_slot_lifecycle.rs, nepl-core/src/resource/collection_slot_state_table.rs, nepl-core/tests/resource_ir.rs"
---

# ISS-20260521T135336921Z-COLLECTION-SLOT-LIFECYCLE-TYPE-CHECK-126AADF5: Collection slot lifecycle type checks conflate slot identity and payload TypeId

## 概要

`CollectionSlotStateTable` は `BorrowRead` / `MoveOut` / `Replace` / `DropInitialized` を `apply_collection_slot_lifecycle_event` へ流すが、この lifecycle boundary は payload 型を exact `TypeId` 比較で判定している。また slot entry の key も `Place` 全体一致に依存しており、`Place.ty` が slot identity に混ざる。

collection slot summary と compiler-owned generic helper は type variable を generic proof pattern として持つため、caller replay では `TypeCtx` / `type_pattern_matches` と同じ型照合が必要になる。さらに `ReplaceInitialized<Old, New>` のような state transition では、slot の物理 identity は storage + offset であり、state が保持する payload type とは分離されていなければならない。

## 対象

- `nepl-core/src/resource/collection_slot_lifecycle.rs`
- `nepl-core/src/resource/collection_slot_state_table.rs`
- `nepl-core/tests/resource_ir.rs`

## 根拠

- `collection_slot_owner_transfer_proof.rs` と `collection_slot_drop_proof.rs` は `type_pattern_matches` を使い、summary-certified proof の generic 型と caller 側の実型を照合している。
- `collection_slot_lifecycle.rs` の state transition だけは `actual == expected_ty` で分岐しており、generic lifecycle event が同じ proof model から外れている。
- `collection_slot_lifecycle_tests.rs` / `lower_collection_slot_tests.rs` は `ReplaceInitialized` が old type と new type を別々に持つ設計を示している一方、state table の slot lookup は `Place.ty` まで含む exact equality に依存している。

## 問題

- generic collection helper summary が `.T` を expected type として持つ場合、caller 側の concrete payload slot と exact `TypeId` が一致せず、valid proof が `TypeMismatch` になる可能性がある。
- slot key が `Place.ty` を含むため、replace 後に state が `Initialized(new_ty)` へ進んでも後続の `new_ty` target lookup が同じ slot として扱われない可能性がある。
- これを stdlib helper 名や API 名の個別許可で回避すると、Stage 6 の generic Resource IR proof boundary 方針に反する。

## 影響

generic collection cleanup / replace / move helper summary が false negative になり、non-Copy collection payload support を public stdlib API へつなぐ段階で helper ごとの個別 proof や allowlist を要求する圧力になる。これは self-host 実装で必要な owning payload collection の memory safety / type safety 基盤を弱める。

## 修正方針

1. `TypeCtx` を collection slot lifecycle transition / precondition path へ渡し、exact `TypeId` 比較を `type_pattern_matches` に基づく generic-aware 型照合へ置き換える。
2. `CollectionSlotStateTable` の slot lookup / update は storage + projection identity で行い、payload `TypeId` は `CollectionSlotState` 側だけで保持する。
3. mismatched concrete type は従来どおり typed `TypeMismatch` refutation として残す。
4. generic expected slot type、type-changing replace state key、source-level `ReplaceReturnOld` の回帰を追加する。

## 検証

- `cargo test -p nepl-core --lib collection_slot -- --test-threads=1`
- `cargo test -p nepl-core --test resource_ir resource_ir_collection_slot_source_replace_return_old_accepts_load_and_store_proofs -- --test-threads=1`
- `cargo test -p nepl-core --test resource_ir resource_ir_collection_slot_non_copy_replace_return_old_accepts_raw_load_and_store_proofs -- --test-threads=1`
- `cargo check -p nepl-core`
- `cargo fmt --check -p nepl-core`
- `node nodesrc/issues.js check --dir issues`

## 対応

`apply_collection_slot_lifecycle_event` が `TypeCtx` を受け取り、initialized slot の expected / actual payload 型を `type_pattern_matches` で照合するようにした。owner-transfer proof / drop proof と同じ generic-aware type-pattern boundary に揃えたため、generic helper summary の type variable と caller 側 concrete payload を exact `TypeId` に依存せず検査できる。

`CollectionSlotStateTable` の slot lookup / update は root + projections を identity として扱うようにした。`Place.ty` は state entry の key ではなく payload state の型として扱うため、`ReplaceInitialized<Old, New>` 後も同じ storage + offset の slot を new payload target から参照できる。

source-level `collection_slot_replace_return_old` の regression を追加し、old raw load proof と new raw store proofが、stdlib helper 名や collection module 名の allowlist なしで `ReplaceReturnOld` lifecycle event を証明することを固定した。
