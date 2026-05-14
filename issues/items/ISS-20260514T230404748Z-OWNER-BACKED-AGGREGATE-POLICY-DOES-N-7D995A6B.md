---
id: ISS-20260514T230404748Z-OWNER-BACKED-AGGREGATE-POLICY-DOES-N-7D995A6B
title: "Owner-backed aggregate policy does not propagate through nested owner fields"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-14
updated: 2026-05-15
target: "nepl-core/src/typecheck/copy_capability.rs, nepl-core/tests/resource_ir.rs"
---

# ISS-20260514T230404748Z-OWNER-BACKED-AGGREGATE-POLICY-DOES-N-7D995A6B: Owner-backed aggregate policy does not propagate through nested owner fields

## 概要

The owner-backed aggregate constructor policy currently marks only structs whose direct field type is a compiler owner token. Aggregates that contain Vec, ByteBuf, HashMapStorage, or another owner-backed aggregate remain public constructors even though constructing them can forge invalid owner/state combinations outside the compiler-owned stdlib boundary.

## 対象

- `nepl-core/src/typecheck/copy_capability.rs, nepl-core/tests/resource_ir.rs`

## 根拠

- `nepl-core/src/typecheck/copy_capability.rs` の owner-backed aggregate policy は、field 型が direct compiler owner token である場合だけ `OwnerBackedAggregateBoundaryOnly` にしていた。
- `Vec<T>` は `RegionToken<T>` を直接 field に持つため制限されるが、`struct Wrapper { items: Vec<i32> }` のような二段目 aggregate は public constructor のまま残っていた。
- `HashMapStorage<K,V>` は `Vec<...>` owner を field に持ち、`HashMap<K,V,H>` は `HashMapStorage<K,V>` を field に持つため、transitive policy がないと collection storage state を user source から再構築できる。

## 問題

The owner-backed aggregate constructor policy currently marks only structs whose direct field type is a compiler owner token. Aggregates that contain Vec, ByteBuf, HashMapStorage, or another owner-backed aggregate remain public constructors even though constructing them can forge invalid owner/state combinations outside the compiler-owned stdlib boundary.

## 影響

User source can wrap or reconstruct owner-backed storage through a nested aggregate shape and bypass the MemPtr = non-owning pointer / RegionToken = free obligation owner separation required by Stage 6. This weakens memory safety because collection/storage invariants can be forged without Resource IR having a trustworthy compiler-issued owner boundary.

## 修正方針

Derive OwnerBackedAggregateBoundaryOnly transitively from struct field types: a struct is owner-backed if any field is a compiler owner token or another owner-backed aggregate, including generic applications. The rule must be structural rather than a stdlib name allowlist and must keep user structs named like memory types unaffected.

## 検証

Add compile-fail regressions for a user wrapper around Vec and for HashMap/HashMapStorage reconstruction through the public facade. Run focused resource/typecheck tests and the static check boundary source policy.

## 2026-05-15 修正

`mark_owner_backed_aggregate_constructor_policies` を fixed-point の構造判定に変更した。struct は field 型が compiler owner token そのものを含む場合だけでなく、すでに owner-backed と判定された aggregate を含む場合にも `OwnerBackedAggregateBoundaryOnly` になる。

この判定は `Vec` / `HashMap` などの stdlib 名 allowlist ではなく、`StructConstructorPolicy::RawMemoryBoundaryOnly(OwnerToken)` と struct field 型から導出する。generic application は base type へ戻して判定し、user source が同名の `RegionToken` / `MemPtr` を定義した場合は既存どおり compiler memory type として扱わない。

追加した regression:

- `typecheck_rejects_nested_owner_backed_aggregate_constructor_outside_boundary`: user-defined `VecBox { items: Vec<i32> }` の constructor を通常 source で拒否する。
- `typecheck_rejects_hashmap_owner_storage_reconstruction_outside_boundary`: `HashMapStorage<i32,i32>` を受け取って `HashMap<i32,i32,DefaultHash32>` を直接再構築する経路を拒否する。

検証:

- `cargo fmt -p nepl-core --check`
- `cargo test -p nepl-core --test resource_ir typecheck_rejects_nested_owner_backed_aggregate_constructor_outside_boundary -- --nocapture`
- `cargo test -p nepl-core --test resource_ir typecheck_rejects_hashmap_owner_storage_reconstruction_outside_boundary -- --nocapture`
- `cargo test -p nepl-core --test resource_ir typecheck_rejects_region_token_struct_constructor_outside_memory_boundary -- --nocapture`
- `cargo test -p nepl-core owner_aggregate -- --nocapture`
- `node nodesrc/test_static_check_boundary_responsibility.js`

関連:

- [NEPLg2 静的検査の複雑化解消計画](../../doc/neplg2/static_check_complexity_reduction_plan.md)
- [stdlib collection / mem / string static safety design](../../doc/neplg2/stdlib_collection_mem_string_static_safety_design.md)
- [stdlib raw-memory-backed APIs require staged effect migration](./ISS-20260427T204839136Z-STDLIB-RAW-MEMORY-BACKED-APIS-REQUIR-E503CD84.md)
