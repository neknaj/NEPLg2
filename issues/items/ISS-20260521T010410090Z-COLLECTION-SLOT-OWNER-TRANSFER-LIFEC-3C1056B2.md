---
id: ISS-20260521T010410090Z-COLLECTION-SLOT-OWNER-TRANSFER-LIFEC-3C1056B2
title: "Collection slot owner-transfer lifecycle lacks payload value-flow proof"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-21
updated: 2026-05-21
target: "nepl-core/src/resource/initialized_collection_slot.rs, nepl-core/src/resource/collection_slot_summary_build.rs, nepl-core/src/resource/collection_slot_summary_apply.rs, nepl-core/src/resource/collection_slot_lifecycle.rs"
---

# ISS-20260521T010410090Z-COLLECTION-SLOT-OWNER-TRANSFER-LIFEC-3C1056B2: Collection slot owner-transfer lifecycle lacks payload value-flow proof

## 概要

CollectionSlotLifecycle primitives such as InitializeEmpty, MoveOut, and ReplaceReturnOld changed initialized/moved/replaced slot state using only storage anchor, offset, and type arguments. They did not carry typed payload source or destination evidence, so Resource IR could assert a non-Copy owner transfer without proving that a payload value was consumed, returned, or materialized.

## 対象

- `nepl-core/src/resource/initialized_collection_slot.rs`
- `nepl-core/src/resource/collection_slot_lifecycle.rs`
- `nepl-core/src/resource/collection_slot_summary_build.rs`
- `nepl-core/src/resource/collection_slot_summary_apply.rs`

## 根拠

- `CollectionSlotLifecycleEvent::InitializeEmpty` は value type argument だけで slot を initialized にしていた。
- `CollectionSlotLifecycleEvent::MoveOut` は output place を持たないため、non-Copy payload の materialize 先を証明できなかった。
- `CollectionSlotLifecycleEvent::ReplaceInitialized(ReturnOldOwner)` は old owner return と new owner consume のどちらも typed payload place proof を持たなかった。
- 追加検証中に、collection slot target を raw value origin まで含む `RawCellAddressAliases::canonicalize` で正規化すると、value move 後の `StorageDealloc` が古い origin へ戻り、移動先 live slot を見落とし得ることも確認した。

## 問題

CollectionSlotLifecycle primitives such as InitializeEmpty, MoveOut, and ReplaceReturnOld changed initialized/moved/replaced slot state using only storage anchor, offset, and type arguments. Copy payload では state-only marker として扱えるが、non-Copy payload では owner consume / materialize / return の証明なしに slot state を進めることになり、memory safety の証明として不十分だった。

## 影響

Future non-Copy collection APIs could satisfy slot lifecycle state transitions with annotations while the actual owner value flow is missing or mismatched, weakening memory-safety and type-safety guarantees. また、raw value origin による canonicalization が collection slot owner place に混入すると、value transfer 後の移動先 storage release が live slot を検出できない。

## 修正方針

`ResourceCheckEngine` が `InitializeEmpty` / `MoveOut` / `ReplaceInitialized(ReturnOldOwner)` / `ReplaceInitialized(DropOldOwner)` を enum match で分類し、non-Copy payload を含む owner-transfer event は `OwnerTransferRequiresValueProof` で拒否する。これにより payload place proof が実装されるまで、state-only annotation で non-Copy owner state を生成・移動・置換できない。

collection slot の place 正規化は raw value origin ではなく `canonicalize_owner_cell_address` を使う。raw address alias の同一性は引き続き扱うが、value move 後の stable origin を owner slot place へ逆流させない。

完全な non-Copy collection support は親 issue [ISS-20260520T152218366Z-NON-COPY-COLLECTION-PAYLOAD-SUPPORT--A6A88543](./ISS-20260520T152218366Z-NON-COPY-COLLECTION-PAYLOAD-SUPPORT--A6A88543.md) の残件として、payload consume / materialize / return place proof と compiler-owned slot-drop lowering を追加する。

## 検証

- `cargo test -p nepl-core collection_slot --lib -- --test-threads=1`
- `cargo test -p nepl-core resource_ir_collection_slot --test resource_ir -- --test-threads=1`
- `cargo test -p nepl-core resource_ir_collection_storage --test resource_ir -- --test-threads=1`

追加した regression:

- non-Copy `InitializeEmpty` は value-flow proof なしに slot state を生成しない。
- proven slot state であっても non-Copy `MoveOut` / `ReplaceReturnOld` は value-flow proof なしに state を進めない。
- droppable `DropInitialized` / `ReplaceDropOld` の drop elaboration guard は維持される。
- value move / aggregate construct / return transfer 後の collection slot dealloc は owner-cell canonicalization により移動先の live slot を検出する。
