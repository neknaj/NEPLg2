---
id: ISS-20260522T210245685Z-VEC-POP-MUST-MOVE-OUT-REMOVED-SLOT-T-8E07FCEA
title: "Vec pop must move out removed slot through Resource IR proof"
area: stdlib
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-22
updated: 2026-05-22
target: "stdlib/alloc/collections/vec/mutation/pop.nepl, stdlib/alloc/collections/vec/types.nepl, nepl-core/tests/resource_ir.rs, nodesrc/test_stdlib_vec_no_unsafe_unwraps.js"
---

# ISS-20260522T210245685Z-VEC-POP-MUST-MOVE-OUT-REMOVED-SLOT-T-8E07FCEA: Vec pop must move out removed slot through Resource IR proof

## 概要

Vec.pop reads the tail payload and returns VecPop, but the removed tail slot is not closed with a collection_slot_move_out lifecycle proof. After push started creating initialized slot state, pop can leave stale initialized evidence outside the shortened initialized_len range, and future non-Copy pop cannot be enabled safely without the same generic MoveOut proof boundary.

## 対象

- `stdlib/alloc/collections/vec/mutation/pop.nepl, stdlib/alloc/collections/vec/types.nepl, nepl-core/tests/resource_ir.rs, nodesrc/test_stdlib_vec_no_unsafe_unwraps.js`

## 根拠

- 親 issue [ISS-20260520T152218366Z-NON-COPY-COLLECTION-PAYLOAD-SUPPORT--A6A88543](./ISS-20260520T152218366Z-NON-COPY-COLLECTION-PAYLOAD-SUPPORT--A6A88543.md) は、non-Copy collection payload を stdlib allowlist ではなく compiler-issued owner / InitializedCell / Resource IR proof へ接続することを要求している。
- [NEPLg2 静的検査の複雑化解消計画 Stage 6](../../doc/neplg2/static_check_complexity_reduction_plan.md#stage-6-resource-ir-と-stdlib-境界の完成) は、collection slot lifecycle を typed enum / match / proof boundary で検査することを完了条件にしている。
- `Vec.push` は `collection_slot_initialize_empty` により tail slot の initialized state を Resource IR へ登録する。したがって `Vec.pop` が tail slot を raw load して `initialized_len` を減らす場合は、同じ slot を `collection_slot_move_out` で閉じなければならない。

## 問題

Vec.pop reads the tail payload and returns VecPop, but the removed tail slot is not closed with a collection_slot_move_out lifecycle proof. After push started creating initialized slot state, pop can leave stale initialized evidence outside the shortened initialized_len range, and future non-Copy pop cannot be enabled safely without the same generic MoveOut proof boundary.

## 影響

Self-host code cannot rely on Vec.pop as an owner-preserving move-out operation. Without a typed MoveOut proof, storage cleanup may either reject valid pop/free sequences or tempt a stdlib-specific allowlist that hides initialized-cell bugs.

## 修正方針

Add private single-slot pop helpers that raw-load the tail element and emit collection_slot_move_out in the same source boundary, route Copy pop through VecStorageInvariant instead of VecCopyInvariant, keep non-Copy public accessors constrained until owner-return API is complete, and add Resource IR regressions for push->pop->free.

## 検証

Run Vec source policy, collection cleanup contract, focused Vec.pop Resource IR lifecycle tests, issues check/index, and git diff --check.

## 2026-05-22 Agent 1 修正

`Vec.pop<T: Copy>` の raw load を public body から private `vec_pop_copy_move_out_initialized_slot<T>` へ移し、helper 内で raw load と `collection_slot_move_out` marker を同じ source boundary に置いた。`pop` 本体は `VecStorageInvariant` を match し、metadata / storage extent が valid な場合だけ helper を呼ぶ。これにより public `pop` は raw pointer operation や lifecycle marker を open-code しない。

`VecCopyInvariant` / `VecDataView` は Copy raw-access authorization と raw storage view を同時に与えるため、removed slot lifecycle を表現するには責務が広すぎる。今回の `pop` は Copy-only のまま維持するが、slot state transition は payload-independent storage proof と single-slot MoveOut proof に分離した。non-Copy `pop` の public 解禁は、`VecPop<T>` から `Vec<T>` と `Option<T>` の両 owner を安全に取り出す consuming API を設計してから行う。

回帰テストとして `Vec<i32>.push -> push -> pop -> vec_pop_vec -> free` の Resource IR lifecycle を追加し、`vec_pop_copy_move_out_initialized_slot` が `CollectionSlotLifecycleEvent::MoveOut` を発行すること、`main` に initialized slot state が残らないことを固定した。source policy も、`pop` public body が raw load / `VecDataView` / `VecCopyInvariant` に戻らず、private helper を通ることを監視する。

検証:

- `node --check nodesrc/test_stdlib_vec_no_unsafe_unwraps.js`
- `node nodesrc/test_stdlib_vec_no_unsafe_unwraps.js`
- `node nodesrc/test_stdlib_collection_cleanup_contract.js`
- `cargo test -p nepl-core --test resource_ir resource_ir_initialized_check_vec_copy_pop_moves_out_tail_slot -- --test-threads=1 --exact --nocapture`
