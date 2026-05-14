---
id: ISS-20260514T164856024Z-OWNER-BACKED-AGGREGATE-CONSTRUCTORS--61400B84
title: "Owner-backed aggregate constructors are forgeable outside compiler boundary"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-14
updated: 2026-05-15
target: "nepl-core/src/typecheck, nepl-core/src/source_map.rs, tests/stdlib/memory_safety.n.md"
---

# ISS-20260514T164856024Z-OWNER-BACKED-AGGREGATE-CONSTRUCTORS--61400B84: Owner-backed aggregate constructors are forgeable outside compiler boundary

## 概要

Structs that directly contain compiler owner tokens such as RegionToken<T> can still be constructed or have their owner-token field projected from user source. This allows source code to forge or extract free-obligation owners from an aggregate shape instead of going through compiler-owned memory boundaries.

## 対象

- `nepl-core/src/typecheck, nepl-core/src/source_map.rs, tests/stdlib/memory_safety.n.md`

## 根拠

- `Vec<T>` が `RegionToken<T>` を直接 field に持つ形へ移行した後も、通常 user source から `Vec<T> len cap state region` の direct struct constructor を呼べた。
- `field::get_ref &v "region"` のような field projection で、aggregate の内部 owner token を user source へ借用できた。
- `RegionToken<T>` 自体の direct constructor / field access は compiler boundary で制限済みだったが、それを field に持つ aggregate には同じ owner 境界が伝播していなかった。
- `doc/neplg2/static_check_complexity_reduction_plan.md` Stage 6 は `MemPtr = non-owning pointer`、`RegionToken/OwnedRegion = free obligation owner` の分離を要求しており、aggregate 経由の owner token forge/projection を許すとこの分離が成立しない。

## 問題

Structs that directly contain compiler owner tokens such as RegionToken<T> can still be constructed or have their owner-token field projected from user source. This allows source code to forge or extract free-obligation owners from an aggregate shape instead of going through compiler-owned memory boundaries.

## 影響

User code can bypass the MemPtr = non-owning pointer / RegionToken = free obligation owner separation required by the static-check complexity reduction plan. That weakens memory-safety guarantees for Vec and future OwnedBuffer-style collection APIs.

## 修正方針

Derive an owner-backed aggregate constructor policy from the struct field types, add a distinct owner aggregate source capability for compiler-owned stdlib implementation files, reject direct constructors and owner-token field projection outside that boundary, and add compile_fail regressions.

## 検証

Run nepl-core cargo checks, static-check source policy tests, trunk build, memory_safety doctests, focused Vec tests, and issue index validation.

## 解決内容

compiler typecheck が struct field 型を見て、direct field に compiler owner token を含む public aggregate を `OwnerBackedAggregateBoundaryOnly` として分類するようにした。これは `Vec` など特定 stdlib 名の allowlist ではなく、`RegionToken<T>` の constructor policy と型形状から導出する。

direct constructor は `owner_aggregate_boundary_allowed` を持つ source に限定し、user source では `type.owner_aggregate.constructor_restricted` を出す。owner token field の projection も、通常の `RegionToken` field access restriction に加えて、aggregate field の型を解決して owner token なら boundary 外で `type.owner_token.field_access_restricted` にする。

`OwnerAggregateBoundary` は `RawMemoryBoundary` と分離した source capability にした。configured stdlib root 配下でも無条件には付与せず、parsed source に aggregate constructor / field accessor の構造化 evidence がある場合だけ付与する。実際に owner-backed aggregate かどうかは typecheck が型形状から判定するため、stdlib の特定 module / 関数名 allowlist にはしない。

stdlib 実装 module は owner aggregate の move/reconstruct/projection が必要だが、raw memory operation authority とは別の権限であるため、raw boundary を広げずに owner aggregate 内部操作だけを許可する。

## 関連ドキュメント

- [NEPLg2 静的検査の複雑化解消計画](../../doc/neplg2/static_check_complexity_reduction_plan.md)

## 検証結果

- `cargo fmt -p nepl-core --check`: passed
- `cargo check -p nepl-core --tests`: passed
- `cargo test -p nepl-core owner_aggregate_boundary -- --nocapture`: 4 passed
- `node nodesrc/test_static_check_boundary_responsibility.js`: passed
- `trunk build`: passed
- `node nodesrc/tests.js -i tests/stdlib/memory_safety.n.md --no-tree -o tmp/agent1-owner-aggregate-fixtures-final.json -j 1 --dist web/dist --assert-io`: total=34, passed=34
- `node nodesrc/tests.js -i stdlib/alloc/collections/vec/access/header.nepl -i stdlib/alloc/collections/vec/access/data.nepl -i stdlib/alloc/collections/vec/storage/alloc.nepl -i stdlib/tests/vec.n.md --no-tree -o tmp/agent1-owner-aggregate-vec-final.json -j 1 --dist web/dist --assert-io`: total=15, passed=15
- `node nodesrc/test_stdlib_vec_no_unsafe_unwraps.js`: passed
- `node nodesrc/test_stdlib_vec_borrowed_observers.js`: passed
- `node nodesrc/test_resource_gate_order.js`: passed
