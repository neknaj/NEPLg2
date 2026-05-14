---
id: ISS-20260514T233136936Z-GENERIC-OWNER-BACKED-AGGREGATE-CONST-6E024598
title: "Generic owner-backed aggregate constructors bypass boundary after type application"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-14
updated: 2026-05-15
target: "nepl-core/src/typecheck/constructor_apply.rs, nepl-core/src/typecheck/copy_capability.rs, nepl-core/tests/resource_ir.rs, nodesrc/test_static_check_boundary_responsibility.js"
---

# ISS-20260514T233136936Z-GENERIC-OWNER-BACKED-AGGREGATE-CONST-6E024598: Generic owner-backed aggregate constructors bypass boundary after type application

## 概要

Owner-backed aggregate constructor policy is stored on the generic struct definition. A public generic wrapper with field type parameter T is not marked owner-backed at definition time, so constructing Wrapper<Vec<i32>> can bypass the owner aggregate constructor boundary even though the applied type contains a Vec owner.

## 対象

- `nepl-core/src/typecheck/constructor_apply.rs, nepl-core/src/typecheck/copy_capability.rs, nepl-core/tests/resource_ir.rs, nodesrc/test_static_check_boundary_responsibility.js`

## 根拠

- `StructConstructorPolicy::OwnerBackedAggregateBoundaryOnly` は struct definition に付くため、`struct OwnerBox<.T>: item <.T>` のような generic definition は `.T` 単体では owner-backed と判定されない。
- 一方で `OwnerBox<Vec<i32>>` の適用後型は `Vec<i32>` owner を field に持つため、constructor を boundary 外で許可すると owner-backed aggregate policy を generic wrapper で迂回できる。
- `field_access` 側は適用後の構造判定を見るようになったため、constructor 側も同じ applied type の証明に揃える必要がある。

## 問題

Owner-backed aggregate constructor policy is stored on the generic struct definition. A public generic wrapper with field type parameter T is not marked owner-backed at definition time, so constructing Wrapper<Vec<i32>> can bypass the owner aggregate constructor boundary even though the applied type contains a Vec owner.

## 影響

Safe source can wrap owner-backed storage inside public generic aggregates and then rely on the wrapper as an invariant-breaking owner container outside stdlib/compiler boundaries.

## 修正方針

After applying constructor type arguments, run the structural owner-backed aggregate predicate on the concrete result type and require OwnerAggregateConstructorBoundary when the applied type is owner-backed.

## 検証

Add resource_ir compile-fail regression for a generic owner wrapper instantiated with Vec<i32>, and source policy coverage that constructor_apply uses the structural predicate.

## 関連ドキュメント

- [NEPLg2 静的検査の複雑化解消計画 Stage 6](../../doc/neplg2/static_check_complexity_reduction_plan.md#stage-6-stdlib--self-host-buffer-%E7%A7%BB%E8%A1%8C)

## 2026-05-15 Agent 1 修正

`apply_struct_constructor` で type arguments を適用した concrete result type を作った後、`target_contains_owner_backed_aggregate` で owner-backed aggregate かを判定するようにした。definition-level policy が `Public` の generic struct でも、適用後の field が `Vec<T>` / `HashMap<T>` / owner token wrapper などを含む場合は `OwnerAggregateConstructorBoundary` capability がなければ `type.owner_aggregate.constructor_restricted` で拒否する。

raw memory boundary 専用 constructor は従来どおり先に `RawMemoryBoundaryOnly` policy で処理し、拒否する場合に不要な applied type を作らない。constructor / field projection の双方が同じ構造判定を使うため、generic wrapper だけで owner-backed aggregate 境界を迂回する経路を閉じた。

検証:

- `cargo fmt -p nepl-core --check`: pass
- `cargo test -p nepl-core --test resource_ir typecheck_rejects_generic_owner_backed_aggregate_constructor_after_application -- --nocapture`: pass
- `cargo test -p nepl-core --test resource_ir typecheck_rejects_nested_owner_backed_aggregate_constructor_outside_boundary -- --nocapture`: pass
- `cargo test -p nepl-core --test resource_ir typecheck_rejects_hashmap_owner_storage_reconstruction_outside_boundary -- --nocapture`: pass
- `cargo test -p nepl-core --test resource_ir typecheck_rejects_hashmap_owner_storage_field_projection_outside_boundary -- --nocapture`: pass
- `cargo test -p nepl-core owner_aggregate -- --nocapture`: pass
- `node nodesrc/test_static_check_boundary_responsibility.js`: pass
- `node nodesrc/issues.js check --dir issues`: pass
- `git diff --check`: pass
