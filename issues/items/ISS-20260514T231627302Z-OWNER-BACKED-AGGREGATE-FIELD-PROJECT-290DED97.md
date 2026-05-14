---
id: ISS-20260514T231627302Z-OWNER-BACKED-AGGREGATE-FIELD-PROJECT-290DED97
title: "Owner-backed aggregate field projection bypasses compiler boundary"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-14
updated: 2026-05-15
target: "nepl-core/src/typecheck/field_access.rs, nepl-core/src/typecheck/copy_capability.rs, nepl-core/tests/resource_ir.rs, nodesrc/test_static_check_boundary_responsibility.js"
---

# ISS-20260514T231627302Z-OWNER-BACKED-AGGREGATE-FIELD-PROJECT-290DED97: Owner-backed aggregate field projection bypasses compiler boundary

## 概要

Owner-backed aggregate constructor policy propagates through nested owner fields, but field projection still only restricts direct compiler owner token/raw pointer fields. User source can project a HashMap.storage or similar nested owner-backed aggregate and bypass the public API invariant boundary.

## 対象

- `nepl-core/src/typecheck/field_access.rs, nepl-core/src/typecheck/copy_capability.rs, nepl-core/tests/resource_ir.rs, nodesrc/test_static_check_boundary_responsibility.js`

## 根拠

- `typecheck/constructor_apply.rs` は `OwnerBackedAggregateBoundaryOnly` constructor を boundary 外で拒否していたが、`typecheck/field_access.rs` は direct `RegionToken` / `MemPtr` field だけを見ていた。
- そのため `HashMap<i32,i32,DefaultHash32>` から `field::get map "storage"` を行うと、`HashMapStorage` owner を public API の外へ取り出せる。
- 同じ構造で、`Vec<i32>` を field に持つ wrapper から `field::get box "items"` を行うと、constructor restriction と field projection restriction の authority が一致しない。

## 問題

Owner-backed aggregate constructor policy propagates through nested owner fields, but field projection still only restricts direct compiler owner token/raw pointer fields. User source can project a HashMap.storage or similar nested owner-backed aggregate and bypass the public API invariant boundary.

## 影響

Safe source can split collection/storage owner internals even though direct reconstruction is rejected, leaving Resource IR and typecheck with inconsistent authority for owner-backed aggregates.

## 修正方針

Use the same structural owner-backed aggregate predicate from typecheck field access, and require owner aggregate field boundary capability when either the base or projected field is owner-backed.

## 検証

Add resource_ir compile-fail regressions for HashMap.storage and nested Vec wrapper field projection, plus source policy coverage for the shared predicate.

## 関連ドキュメント

- [NEPLg2 静的検査の複雑化解消計画 Stage 6](../../doc/neplg2/static_check_complexity_reduction_plan.md#stage-6-stdlib--self-host-buffer-%E7%A7%BB%E8%A1%8C)

## 2026-05-15 Agent 1 修正

`target_contains_owner_backed_aggregate` を `copy_capability.rs` から field access policy へ共有し、base または projected field が owner-backed aggregate の場合は `OwnerAggregateFieldBoundary` capability を要求するようにした。

判定は direct owner token だけでなく、generic application の type parameter substitution、enum payload、tuple、box を再帰的に見る。これにより `HashMap.storage`、`HashMapStorage`、`Vec` wrapper、`Option<Vec<T>>` のような構造も型形状から owner-backed として扱える。

診断は新しい enum variant `TypeDiagnosticCode::OwnerAggregateFieldAccessRestricted` として追加し、旧 D 番号や文字列分岐ではなく `match` の網羅性で登録・表示する。

検証:

- `cargo test -p nepl-core --test resource_ir typecheck_rejects_hashmap_owner_storage_field_projection_outside_boundary -- --nocapture`: pass
- `cargo test -p nepl-core --test resource_ir typecheck_rejects_nested_owner_backed_aggregate_field_projection_outside_boundary -- --nocapture`: pass
- `cargo test -p nepl-core owner_aggregate -- --nocapture`: pass
- `cargo test -p nepl-core --test resource_ir typecheck_rejects_nested_owner_backed_aggregate_constructor_outside_boundary -- --nocapture`: pass
- `cargo test -p nepl-core --test resource_ir typecheck_rejects_region_token_field_access_outside_memory_boundary -- --nocapture`: pass
- `cargo test -p nepl-core diagnostic_codes_have_unique_serialized_names -- --nocapture`: pass
- `cargo fmt -p nepl-core --check`: pass
- `node nodesrc/test_static_check_boundary_responsibility.js`: pass
- `node nodesrc/issues.js check --dir issues`: pass
- `git diff --check`: pass
