---
id: ISS-20260516T103203631Z-OWNER-AGGREGATE-PROOF-MISSES-NESTED--F0DD4C3F
title: "Owner aggregate proof misses nested non-generic constructors"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-16
updated: 2026-05-16
target: nepl-core/src/source_capability/constructor_position.rs
---

# ISS-20260516T103203631Z-OWNER-AGGREGATE-PROOF-MISSES-NESTED--F0DD4C3F: Owner aggregate proof misses nested non-generic constructors

## 概要

SourceCapability explicit constructor evidence currently requires explicit type arguments, so compiler-owned stdlib sources such as adjacency_matrix/api/create.nepl cannot prove AdjacencyMatrix construction when a non-generic owner aggregate constructor is nested as a Result payload.

## 対象

- `nepl-core/src/source_capability/constructor_position.rs`

## 根拠

- `stdlib/alloc/collections/adjacency_matrix/api/create.nepl::doctest#1` は修正前、`ok<AdjacencyMatrix, Diag> AdjacencyMatrix nverts nbytes bits` の `AdjacencyMatrix` に `OwnerAggregateConstructorBoundary("AdjacencyMatrix")` が付かず、`type.owner_aggregate.constructor_restricted` で失敗していた。
- `nepl-core/src/source_capability/constructor_position.rs` は、nested explicit constructor evidence を `type_args.is_empty()` で落としていたため、非 generic owner aggregate constructor を Result payload などの引数位置で証明できなかった。

## 問題

SourceCapability explicit constructor evidence currently requires explicit type arguments, so compiler-owned stdlib sources such as adjacency_matrix/api/create.nepl cannot prove AdjacencyMatrix construction when a non-generic owner aggregate constructor is nested as a Result payload.

## 影響

Valid compiler-owned owner aggregate implementation sources are rejected by type.owner_aggregate.constructor_restricted, while the intended source-based proof model remains incomplete for non-generic aggregates.

## 修正方針

Model nested constructor evidence structurally from constructor-like symbols with following payload, regardless of generic type args, while retaining shadowing, qualified enum-variant rejection, same-module enum rejection, and user-source capability rejection.

## 対応内容

- `explicit_constructor_symbol` は、payload を持つ identifier を type args の有無に関係なく nested constructor evidence として observer へ渡すようにした。
- owner aggregate evidence 側の shadowing、qualified enum variant rejection、same-module enum rejection、user source capability rejection は変更していない。
- `owner_aggregate_boundary_accepts_nested_nongeneric_constructor_evidence` を追加し、`AdjacencyMatrix` のような非 generic constructor が Result payload に入る場合の source proof を固定した。
- focused adjacency_matrix doctest は `type.owner_aggregate.constructor_restricted` を出さなくなった。次の blocker として `RegionToken` field projection capability の不備を [ISS-20260516T103911431Z-OWNER-TOKEN-FIELD-PROJECTION-IGNORES-A0BA1412](./ISS-20260516T103911431Z-OWNER-TOKEN-FIELD-PROJECTION-IGNORES-A0BA1412.md) に分離した。

## 検証

- `cargo fmt -p nepl-core --check`: passed
- `cargo test -p nepl-core owner_aggregate_boundary_accepts_nested_nongeneric_constructor_evidence -- --nocapture`: passed
- `cargo check -p nepl-core`: passed
- `node nodesrc/test_static_check_boundary_responsibility.js`: passed
- `trunk build`: passed
- `node nodesrc/run_doctest.js -i stdlib/alloc/collections/adjacency_matrix/api/create.nepl -n 1 --dist web/dist`: `type.owner_aggregate.constructor_restricted` は消え、次の `type.owner_token.field_access_restricted` に到達
