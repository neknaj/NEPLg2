---
id: ISS-20260521T132527293Z-SOURCE-LEVEL-COLLECTION-DROP-TRAVERS-24EF497F
title: "Source-level collection drop traversal lacks end-to-end storage release regression"
area: core
status: fixed
resolved: true
priority: P1
type: test
created: 2026-05-21
updated: 2026-05-21
target: nepl-core/tests/resource_ir.rs
---

# ISS-20260521T132527293Z-SOURCE-LEVEL-COLLECTION-DROP-TRAVERS-24EF497F: Source-level collection drop traversal lacks end-to-end storage release regression

## 概要

compiler-owned stdlib source はすでに `collection_slot_drop_traversal` を lowering し、手書き Resource IR は traversal 後の storage release を検証している。しかし、source-level compiler-owned stdlib code から raw store/load/drop、drop traversal、raw dealloc、`collection_slot_storage_dealloc` が一続きの generic proof path として接続される回帰がなかった。

## 対象

- `nepl-core/tests/resource_ir.rs`

## 根拠

- 親 issue [ISS-20260520T152218366Z-NON-COPY-COLLECTION-PAYLOAD-SUPPORT--A6A88543](./ISS-20260520T152218366Z-NON-COPY-COLLECTION-PAYLOAD-SUPPORT--A6A88543.md) は、non-Copy collection cleanup を stdlib module allowlist ではなく compiler-issued owner / Resource IR / generic proof boundary へ接続することを要求している。
- [ISS-20260521T114146610Z-COLLECTION-SLOT-DROP-TRAVERSAL-NEEDS-60837C0B](./ISS-20260521T114146610Z-COLLECTION-SLOT-DROP-TRAVERSAL-NEEDS-60837C0B.md) は `ResourceOp::CollectionSlotDropTraversal` と certified traversal summary を追加したが、positive source path と raw storage release を同時に固定していなかった。
- [ISS-20260521T131741789Z-COLLECTION-SLOT-DROP-TRAVERSAL-LOWER-691CB7BE](./ISS-20260521T131741789Z-COLLECTION-SLOT-DROP-TRAVERSAL-LOWER-691CB7BE.md) は source-level producer 欠落を coverage count で検出するが、traversal proof と storage dealloc proof の end-to-end success path そのものは別に必要である。

## 問題

isolated lowering coverage と手書き Resource IR だけでは、実際の compiler-owned stdlib source が次の証明を同時に満たすことを保証できない。

- raw `store<T>` が non-Copy payload を collection slot 初期化 proof に接続する。
- raw `load<T>` と actual drop が collection-wide drop traversal proof に接続する。
- raw `dealloc_raw` が storage release proof を発行し、`collection_slot_storage_dealloc` がその proof を消費する。
- 上記が stdlib module 名や関数名の allowlist ではなく、Resource IR の typed op と alias-aware proof boundary だけで通る。

## 影響

A future lowering or alias-summary change could keep isolated producer coverage green while breaking the actual non-Copy collection cleanup path needed by OwnedBuffer/Vec self-host data structures.

## 修正方針

Add a source-level compiler-owned stdlib regression that uses RegionToken storage, raw slot store/load/drop proofs, collection_slot_drop_traversal, raw dealloc release proof, and collection_slot_storage_dealloc without stdlib module allowlists.

## 検証

cargo test -p nepl-core --test resource_ir resource_ir_collection_slot_source_drop_traversal_storage_release -- --test-threads=1

## 対応

`resource_ir_collection_slot_source_drop_traversal_storage_release` を追加した。fixture は compiler-owned stdlib source として `&RegionToken<LocalOwner>` anchor を使い、2 slot 分の raw store / InitializeEmpty / raw load / actual drop / `collection_slot_drop_traversal` / `dealloc_raw` / `collection_slot_storage_dealloc` を同じ function 内で通す。

この回帰は、`ResourceOp::CollectionSlotDropTraversal`、`RawMemoryOp::Dealloc`、`CollectionSlotLifecycleEvent::StorageDealloc` が source lowering から typed op として残ることも確認する。`Vec` や特定 helper 名の allowlist は追加していない。
