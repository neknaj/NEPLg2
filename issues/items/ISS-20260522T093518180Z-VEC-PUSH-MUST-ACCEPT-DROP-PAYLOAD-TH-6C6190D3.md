---
id: ISS-20260522T093518180Z-VEC-PUSH-MUST-ACCEPT-DROP-PAYLOAD-TH-6C6190D3
title: "Vec push must accept Drop payload through storage invariant and Resource IR slot proof"
area: stdlib
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-22
updated: 2026-05-22
target: "stdlib/alloc/collections/vec/mutation/push.nepl, nodesrc/test_stdlib_vec_no_unsafe_unwraps.js, nodesrc/test_stdlib_collection_cleanup_contract.js, nepl-core/tests/resource_ir.rs"
---

# ISS-20260522T093518180Z-VEC-PUSH-MUST-ACCEPT-DROP-PAYLOAD-TH-6C6190D3: Vec push must accept Drop payload through storage invariant and Resource IR slot proof

## 概要

Vec.push success path remains Copy-only even after VecStorageInvariant, owner-preserving failure payload, core/mem storage relocate, and Drop cleanup proof are available. Keeping push Copy-only blocks self-host owning payload vectors; removing Copy without structural proof would allow raw slot writes or item owner loss.

## 対象

- `stdlib/alloc/collections/vec/mutation/push.nepl, nodesrc/test_stdlib_vec_no_unsafe_unwraps.js, nodesrc/test_stdlib_collection_cleanup_contract.js, nepl-core/tests/resource_ir.rs`

## 根拠

- 親 issue [ISS-20260520T152218366Z-NON-COPY-COLLECTION-PAYLOAD-SUPPORT--A6A88543](./ISS-20260520T152218366Z-NON-COPY-COLLECTION-PAYLOAD-SUPPORT--A6A88543.md) は、non-Copy collection payload を stdlib allowlist ではなく compiler-issued owner / InitializedCell / Resource IR proof へ接続することを要求している。
- [NEPLg2 静的検査の複雑化解消計画 Stage 6](../../doc/neplg2/static_check_complexity_reduction_plan.md#stage-6-resource-ir-と-stdlib-境界の完成) は、collection slot lifecycle、storage relocation、drop traversal を typed enum / match / proof boundary で検査することを完了条件にしている。
- 直前の [ISS-20260522T090905300Z-VEC-PUSH-FAILURE-MUST-RETURN-REJECTE-21E0522B](./ISS-20260522T090905300Z-VEC-PUSH-FAILURE-MUST-RETURN-REJECTE-21E0522B.md) により、失敗時の `Vec<T>` owner と rejected `item: T` は `VecPushRejected<T>` として API 型へ残るようになった。
- [ISS-20260522T073914640Z-VEC-NON-COPY-LIFECYCLE-NEEDS-STORAGE-645ED85D](./ISS-20260522T073914640Z-VEC-NON-COPY-LIFECYCLE-NEEDS-STORAGE-645ED85D.md) で、payload 非依存の `VecStorageInvariant` が `VecCopyInvariant` から分離済みである。

## 問題

Vec.push success path remains Copy-only even after VecStorageInvariant, owner-preserving failure payload, core/mem storage relocate, and Drop cleanup proof are available. Keeping push Copy-only blocks self-host owning payload vectors; removing Copy without structural proof would allow raw slot writes or item owner loss.

## 影響

Self-host AST/HIR/diagnostic collections cannot safely store Drop payloads. A naive relaxation would reintroduce shallow-copy or leaked item owner paths on allocation/grow failure.

## 修正方針

Move push implementation behind a private payload-independent VecStorageInvariant helper, expose Copy and Drop overloads that delegate to it, require private slot initialize proof and owner-preserving VecPushRejected failure payload, and add source/Resource IR regressions that forbid VecCopyInvariant or stdlib allowlists on the append path.

## 検証

Run Vec source policy, collection cleanup contract policy, focused Drop payload Vec.push Resource IR lifecycle tests, issues check/index, and git diff --check.

## 2026-05-22 修正

`Vec.push` の実装本体を private `vec_push_storage_checked<T>` へ移し、public surface は `push<T: Copy>` と `push<T: Drop>` の overload が同じ helper へ委譲する形にした。helper は `VecStorageInvariant` を match して `len` / `initialized_len` / `cap` / storage extent の相関を payload 非依存で確認し、`VecCopyInvariant` / `vec_buffer_current_copy_invariant` は使わない。

success path の raw slot write は引き続き private `vec_push_slot_store_initialized<T>` に閉じ、`store<T>` と `collection_slot_initialize_empty` marker を同じ implementation boundary に置く。public wrapper は raw pointer operation や lifecycle marker を open-code しない。failure path は `VecPushRejected<T>` を通して、消費した `Vec<T>` と storage に入らなかった `item: T` を同時に返す。

`nodesrc/test_stdlib_collection_cleanup_contract.js` は、Drop-capable owner update を関数名 allowlist ではなく構造で分類するように更新した。条件は「public Drop overload は raw store/marker を持たず private helper へ委譲する」「private helper は `VecStorageInvariant`、owner-preserving `VecPushRejected<T>`、private slot initialize helper の組み合わせで証明される」「Copy raw-access proof や public marker authority を使わない」である。

`nepl-core/tests/resource_ir.rs` に `Vec<DropPayload>.new -> push -> free` と `with_capacity -> push -> push(grow) -> free` の回帰を追加した。前者は `InitializeEmpty` と actual `Drop::drop` / `CollectionSlotDropTraversal` の接続、後者は core/mem private realloc boundary の `CollectionStorageRelocate` を確認する。どちらも Drop payload 経路で `vec_buffer_current_copy_invariant` が monomorphize されないことを固定した。

検証:

- `node nodesrc/test_stdlib_vec_no_unsafe_unwraps.js`
- `node nodesrc/test_stdlib_collection_cleanup_contract.js`
- `cargo test -p nepl-core --test resource_ir resource_ir_initialized_check_vec_drop_push_free_closes_stdlib_lifecycle -- --test-threads=1 --exact --nocapture`
- `cargo test -p nepl-core --test resource_ir resource_ir_initialized_check_vec_drop_push_grow_relocates_stdlib_lifecycle -- --test-threads=1 --exact --nocapture`
- `cargo test -p nepl-core --test resource_ir resource_ir_initialized_check_vec_grow_relocates_stdlib_lifecycle -- --test-threads=1 --exact --nocapture`
