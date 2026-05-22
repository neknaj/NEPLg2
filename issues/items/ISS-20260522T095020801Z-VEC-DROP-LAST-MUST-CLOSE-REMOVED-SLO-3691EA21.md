---
id: ISS-20260522T095020801Z-VEC-DROP-LAST-MUST-CLOSE-REMOVED-SLO-3691EA21
title: "Vec drop_last must close removed slot with Resource IR proof"
area: stdlib
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-22
updated: 2026-05-22
target: "stdlib/alloc/collections/vec/mutation/pop.nepl, nodesrc/test_stdlib_vec_no_unsafe_unwraps.js, nodesrc/test_stdlib_collection_cleanup_contract.js, nepl-core/tests/resource_ir.rs"
---

# ISS-20260522T095020801Z-VEC-DROP-LAST-MUST-CLOSE-REMOVED-SLO-3691EA21: Vec drop_last must close removed slot with Resource IR proof

## 概要

Vec.drop_last only decrements len/initialized_len under a Copy-only contract and emits no collection slot lifecycle proof for the removed tail slot. This leaves the API unusable for Drop payloads and risks metadata-only cleanup diverging from Resource IR initialized slot state.

## 対象

- `stdlib/alloc/collections/vec/mutation/pop.nepl, nodesrc/test_stdlib_vec_no_unsafe_unwraps.js, nodesrc/test_stdlib_collection_cleanup_contract.js, nepl-core/tests/resource_ir.rs`

## 根拠

- 親 issue [ISS-20260520T152218366Z-NON-COPY-COLLECTION-PAYLOAD-SUPPORT--A6A88543](./ISS-20260520T152218366Z-NON-COPY-COLLECTION-PAYLOAD-SUPPORT--A6A88543.md) は、non-Copy collection payload を stdlib allowlist ではなく compiler-issued owner / InitializedCell / Resource IR proof へ接続することを要求している。
- [NEPLg2 静的検査の複雑化解消計画 Stage 6](../../doc/neplg2/static_check_complexity_reduction_plan.md#stage-6-resource-ir-と-stdlib-境界の完成) は、collection slot lifecycle、storage relocation、drop traversal を typed enum / match / proof boundary で検査することを完了条件にしている。
- `Vec.drop_last<T: Drop>` は removed tail slot を `Drop::drop` した後に `collection_slot_drop_initialized` で閉じる必要がある。metadata の `len` / `initialized_len` だけを減らすと、owner storage 上の initialized slot state と source API contract が分離する。

## 問題

Vec.drop_last only decrements len/initialized_len under a Copy-only contract and emits no collection slot lifecycle proof for the removed tail slot. This leaves the API unusable for Drop payloads and risks metadata-only cleanup diverging from Resource IR initialized slot state.

## 影響

Self-host code cannot discard the last Drop payload element from Vec without clearing the whole collection. If the metadata-only pattern is extended to non-Copy payloads it would hide live slots from static cleanup proof.

## 修正方針

Add private single-slot drop helpers that pair raw load/drop evidence with collection_slot_drop_initialized, update drop_last Copy and Drop overloads to use VecStorageInvariant and those helpers, and add source/Resource IR regressions for Copy and Drop payloads.

## 検証

Run Vec source policy, collection cleanup contract policy, focused Vec.drop_last Resource IR lifecycle tests, issues check/index, and git diff --check.

## 2026-05-22 Agent 1 修正

`Vec.drop_last` を `Copy` / `Drop` overload に分け、public wrapper から private storage-checked helper へ委譲する構造にした。`drop_last<T: Copy>` は removed tail slot の値を返すため `vec_drop_last_copy_initialized_slot<T>` を使い、`drop_last<T: Drop>` は actual `Drop::drop` を実行してから `collection_slot_drop_initialized` を消費する `vec_drop_last_drop_initialized_slot<T>` を使う。どちらも `VecStorageInvariant` によって payload 非依存の storage metadata / extent を確認し、public API には raw pointer operation や lifecycle marker を出さない。

Resource IR 側の根本原因は、removed slot の precondition seed 自体ではなく、その後の `region_ptr` / `mem_ptr_add` / `mem_ptr_addr` などの non-owning `MemPtr` 経路を「non-Copy owner move」と同一視していたことだった。`MemPtr` は memory model 上 non-owning pointer であり、collection slot state を所有するのは `OwnedRegion` / `RegionToken` などの storage owner carrier である。そこで `type_carries_collection_slot_owner` を追加し、collection slot state の transfer / consumed-argument cleanup は owner obligation carrier に限って行うようにした。`MemPtr` や owner token への参照は slot state を移動・消去しない。

callee return path summary では、return path ごとの cell / collection slot / raw alias state を分離して replay し、path-insensitive な summary ops を先に適用して不可能 path の `DropInitialized` を混ぜないようにした。i32 scalar return facts も return path に持たせ、`initialized_len - 1` で得た tail index が `0 <= index < initialized_count` を満たす事実を caller 側へ伝播できるようにした。これにより `drop_last<T: Drop>` が raw load 済み payload の actual drop と removed slot の initialized state を同じ proof boundary で閉じる。

回帰テストとして `Vec<DropPayload>.push -> drop_last -> free` と `Vec<i32>.push -> drop_last -> free` の Resource IR lifecycle を追加し、`Drop` payload 経路で `CollectionSlotLifecycleEvent::DropInitialized`、actual loaded-value drop proof、cleanup traversal が閉じることを固定した。owner-carrier predicate については `MemPtr` が non-owning、`RegionToken` を含む構造体が owning carrier、`&RegionToken` が non-owning であることをユニットテストで固定した。

検証:

- `cargo fmt`
- `cargo test -p nepl-core resource::collection_slot_owner_carrier_tests -- --nocapture`
- `cargo test -p nepl-core --test resource_ir resource_ir_initialized_check_vec_drop_drop_last_closes_tail_slot -- --test-threads=1 --exact --nocapture`
- `cargo test -p nepl-core --test resource_ir resource_ir_initialized_check_vec_copy_drop_last_closes_tail_slot -- --test-threads=1 --exact --nocapture`
- `cargo test -p nepl-core --test resource_ir resource_ir_collection_slot_source_drop_initialized_accepts_actual_loaded_value_drop -- --test-threads=1 --exact --nocapture`
- `cargo test -p nepl-core --test resource_ir resource_ir_initialized_check_vec_push_free_closes_stdlib_lifecycle -- --test-threads=1 --exact --nocapture`
- `cargo test -p nepl-core resource::initialized_alias_tests::i32_ -- --nocapture`
- `cargo test -p nepl-core resource::initialized_alias_i32_condition_tests:: -- --nocapture`
- `node nodesrc/test_stdlib_vec_no_unsafe_unwraps.js`
- `node nodesrc/test_stdlib_collection_cleanup_contract.js`
- `node nodesrc/issues.js check --dir issues`
- `git diff --check`
