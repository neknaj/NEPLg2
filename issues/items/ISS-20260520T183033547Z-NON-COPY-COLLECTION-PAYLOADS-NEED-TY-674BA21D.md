---
id: ISS-20260520T183033547Z-NON-COPY-COLLECTION-PAYLOADS-NEED-TY-674BA21D
title: "Non-Copy collection payloads need typed slot lifecycle proof boundary"
area: core
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-20
updated: 2026-05-21
target: "nepl-core/src/resource/**, nepl-core/tests/resource_ir.rs, doc/neplg2/stdlib_collection_mem_string_static_safety_design.md"
---

# ISS-20260520T183033547Z-NON-COPY-COLLECTION-PAYLOADS-NEED-TY-674BA21D: Non-Copy collection payloads need typed slot lifecycle proof boundary

## 概要

Non-Copy collection payload support cannot be implemented by relaxing raw mem_copy/mem_move or by stdlib module allowlists. Raw bulk move is byte-preserving memmove and the current public typed API correctly stays Copy-only, but compiler-core has no typed collection slot lifecycle fact for Initialize/BorrowRead/MoveOut/Replace/Drop/StorageDealloc.

## 対象

- `nepl-core/src/resource/**, nepl-core/tests/resource_ir.rs, doc/neplg2/stdlib_collection_mem_string_static_safety_design.md`

## 根拠

- `core/mem/pointer/bulk.nepl` は `mem_move<T>` を `.T: Copy` に限定し、source の所有権を未初期化へ変える操作ではないと明記している。
- `tests/compiler/move_effect.n.md` には raw `mem_move` が initialized non-Copy source を byte move できない compile_fail regression がある。
- [stdlib collection/mem/string と静的検査の安全設計](../../doc/neplg2/stdlib_collection_mem_string_static_safety_design.md) は、non-Copy payload collection を raw bulk operation ではなく initialized cell state / drop traversal / generic Resource IR proof boundary へ載せる方針を示している。
- 既存の [ISS-20260520T152218366Z-NON-COPY-COLLECTION-PAYLOAD-SUPPORT--A6A88543](./ISS-20260520T152218366Z-NON-COPY-COLLECTION-PAYLOAD-SUPPORT--A6A88543.md) は、stdlib 個別 proof ではなく compiler-issued owner token と generic proof boundary が必要であることを P1 として残している。

## 問題

Non-Copy collection payload support cannot be implemented by relaxing raw mem_copy/mem_move or by stdlib module allowlists. Raw bulk move is byte-preserving memmove and the current public typed API correctly stays Copy-only, but compiler-core has no typed collection slot lifecycle fact for Initialize/BorrowRead/MoveOut/Replace/Drop/StorageDealloc.

## 影響

Self-host collections would either stay Copy-only or reintroduce shallow copies, leaks, double drop, or module-specific proof exceptions. This blocks safe AST/HIR/diagnostic collections and conflicts with the generic Resource IR proof policy.

## 修正方針

Add a generic typed collection slot lifecycle model in compiler-core as the proof boundary for non-Copy payload storage. Keep raw bulk memory operations Copy-only; model ownership moves with slot operations instead of byte memmove.

## 対応

- `CollectionSlotState` を `Uninitialized | Initialized(TypeId) | Moved(TypeId) | Dropped(TypeId) | Released` として追加した。
- `CollectionSlotLifecycleEvent` を `InitializeEmpty`、`BorrowRead`、`MoveOut`、`ReplaceInitialized`、`DropInitialized`、`StorageDealloc` に分けた。
- `CollectionSlotLifecycleRefutation` は unavailable state、type mismatch、live overwrite、live slot dealloc を enum payload として保持する。
- `apply_collection_slot_lifecycle_event` を `resource` module の公開 proof boundary として re-export し、将来の lowering / checker が stdlib 名の allowlist ではなく同じ typed transition を使えるようにした。
- doc に、raw `mem_move` を ownership move として扱わず、non-Copy payload は slot lifecycle event へ載せる設計を追記した。

## 検証

- `cargo test -p nepl-core collection_slot_lifecycle -- --test-threads=1`
- `node nodesrc/test_resource_checker_responsibility.js`
