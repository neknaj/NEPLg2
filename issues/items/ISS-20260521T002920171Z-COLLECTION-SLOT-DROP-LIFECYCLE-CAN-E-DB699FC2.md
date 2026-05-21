---
id: ISS-20260521T002920171Z-COLLECTION-SLOT-DROP-LIFECYCLE-CAN-E-DB699FC2
title: "Collection slot drop lifecycle can erase droppable payload without drop elaboration"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-21
updated: 2026-05-21
target: "nepl-core/src/resource/initialized_collection_slot.rs, nepl-core/src/resource/collection_slot_lifecycle.rs, nepl-core/tests/resource_ir.rs"
source: "doc/neplg2/static_check_complexity_reduction_plan.md#stage-6-self-host-compiler-準備"
---

# ISS-20260521T002920171Z-COLLECTION-SLOT-DROP-LIFECYCLE-CAN-E-DB699FC2: Collection slot drop lifecycle can erase droppable payload without drop elaboration

## 概要

CollectionSlotLifecycleEvent::DropInitialized and ReplaceInitialized(DropOldOwner) can currently advance slot state to Dropped/Initialized by state transition alone. For payload types that require actual Drop elaboration, storage dealloc can then trust Dropped state without proving destructor insertion.

## 対象

- `nepl-core/src/resource/initialized_collection_slot.rs, nepl-core/src/resource/collection_slot_lifecycle.rs, nepl-core/tests/resource_ir.rs`

## 根拠

- [NEPLg2 静的検査の複雑化解消計画](../../doc/neplg2/static_check_complexity_reduction_plan.md) Stage 6 は、Resource IR が move / borrow / lifetime / initialized / drop / raw provenance を共有状態として検査し、`MemPtr` / owner token / initialized cell を分離することを完了条件にしている。
- [stdlib collection/mem/string と静的検査の安全設計](../../doc/neplg2/stdlib_collection_mem_string_static_safety_design.md) は、non-Copy payload collection の safe move/drop を `CollectionSlotLifecycleEvent::DropInitialized` / `ReplaceDropOld` などの typed lifecycle event として compiler-core の generic proof boundary へ載せる方針を明記している。
- 親 issue [ISS-20260520T152218366Z-NON-COPY-COLLECTION-PAYLOAD-SUPPORT--A6A88543](./ISS-20260520T152218366Z-NON-COPY-COLLECTION-PAYLOAD-SUPPORT--A6A88543.md) は、`OwnedBuffer<T>` の moved slot / initialized cell state と drop traversal が未完であることを追跡している。

## 問題

CollectionSlotLifecycleEvent::DropInitialized and ReplaceInitialized(DropOldOwner) can currently advance slot state to Dropped/Initialized by state transition alone. For payload types that require actual Drop elaboration, storage dealloc can then trust Dropped state without proving destructor insertion.

## 影響

Once non-Copy collection payload APIs start using these intrinsics, a Vec-like collection could mark a live droppable payload as dropped and release backing storage without running the payload destructor. This violates the generic Resource IR proof boundary and memory-safety policy.

## 修正方針

Make collection slot drop-producing events typed proof obligations. If the payload type is StateOnly, the existing state transition remains valid. If the payload type needs Drop code, the compiler must either produce a typed drop-elaboration path or reject the lifecycle event. This issue implements the safe compiler-side rejection and regression coverage; the parent non-Copy collection issue remains open for full slot-drop lowering and public API work.

## 修正内容

- `CollectionSlotLifecycleRefutation::DropRequiresElaboration` と対応する `resource.collection_slot.drop_requires_elaboration` diagnostic を追加した。
- `ResourceCheckEngine::apply_collection_slot_lifecycle` で `DropInitialized` と `ReplaceInitialized(DropOldOwner)` を drop-producing event として enum / match で分類するようにした。
- 対象 payload type が `ResourceDropRequirement::StateOnly` ではない場合、slot state を `Dropped` / replacement 後 state に進めず、typed refutation を出すようにした。
- `ReplaceInitialized(ReturnOldOwner)` は drop-producing event ではなく ownership transfer として扱い、drop diagnostic を出さないことを回帰テストで固定した。

## 検証

Add Resource IR regressions for DropInitialized and ReplaceDropOld on a type with Drop impl. Verify they emit typed collection-slot diagnostics and do not let storage dealloc succeed by state-only cleanup. Also verify ReplaceReturnOld remains an ownership-transfer event and does not produce the drop diagnostic.

実施:

- `cargo test -p nepl-core resource_ir_collection_slot_drop_initialized_requires_drop_elaboration_for_droppable_payload --test resource_ir -- --test-threads=1`
- `cargo test -p nepl-core resource_ir_collection_slot_replace_ --test resource_ir -- --test-threads=1`
- `cargo test -p nepl-core collection_slot --lib -- --test-threads=1`
