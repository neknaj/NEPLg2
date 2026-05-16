---
id: ISS-20260516T103911431Z-OWNER-TOKEN-FIELD-PROJECTION-IGNORES-A0BA1412
title: "Owner token field projection ignores proven field boundary source"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-16
updated: 2026-05-16
target: nepl-core/src/typecheck/field_access.rs
---

# ISS-20260516T103911431Z-OWNER-TOKEN-FIELD-PROJECTION-IGNORES-A0BA1412: Owner token field projection ignores proven field boundary source

## 概要

After nested constructor proof is fixed, stdlib core/mem/pointer/region.nepl and alloc/string/storage.nepl still fail with type.owner_token.field_access_restricted even though the compiler-owned source imports core/field and contains explicit field accessor evidence.

## 対象

- `nepl-core/src/typecheck/field_access.rs`

## 根拠

- `ISS-20260516T103203631Z-OWNER-AGGREGATE-PROOF-MISSES-NESTED--F0DD4C3F` の修正後、`stdlib/alloc/collections/adjacency_matrix/api/create.nepl::doctest#1` は owner aggregate constructor restriction を越えたが、次に `/stdlib/core/mem/pointer/region.nepl` の `get region "size"` / `get region "raw"` と `/stdlib/alloc/string/storage.nepl` の `get region "raw"` が `type.owner_token.field_access_restricted` で失敗した。
- 対象 source は `#import "core/field" as *` を持ち、`OwnerAggregateFieldBoundary` source evidence 自体は構文上存在する。
- 現在の `restricted_struct_field_access_allowed` は direct `RegionToken` base の field projection では `OwnerAggregateFieldBoundary` を見ず、同じ capability を owner-backed aggregate の field 型 projection 側にしか適用していない。

## 問題

After nested constructor proof is fixed, stdlib core/mem/pointer/region.nepl and alloc/string/storage.nepl still fail with type.owner_token.field_access_restricted even though the compiler-owned source imports core/field and contains explicit field accessor evidence.

## 影響

Valid compiler-owned memory boundary helpers cannot project RegionToken raw/size fields, so stdlib doctests and downstream collection APIs fail before Resource IR can prove the actual ownership flow.

## 修正方針

Allow direct OwnerToken field projection when the source file has OwnerAggregateFieldBoundary evidence, while keeping RawPointer field access restricted to raw structural/compiler memory definition evidence and keeping user source without capability rejected.

## 対応内容

- `restricted_struct_field_access_allowed` の `OwnerToken` branch で `OwnerAggregateFieldBoundary` を認めるようにした。
- `RawPointer` branch は従来どおり raw structural boundary / compiler memory type definition capability のみで許可し、owner field source proof では許可しない。
- user source は `SourceCapabilities::none` のままなので、`#import "core/field"` を書いただけでは direct `RegionToken` field projection を許可しない。
- focused `region.nepl` doctest は通過した。`adjacency_matrix/api/create.nepl` は compiler boundary 系の error を越え、次の stale doctest assertion issue [ISS-20260516T104604371Z-ADJACENCYMATRIX-CREATE-DOCTEST-STILL-9ACAECC9](./ISS-20260516T104604371Z-ADJACENCYMATRIX-CREATE-DOCTEST-STILL-9ACAECC9.md) に進んだ。

## 検証

- `cargo fmt -p nepl-core --check`: passed
- `cargo test -p nepl-core --test resource_ir typecheck_allows_region_token_field_access_with_owner_field_boundary -- --exact --nocapture`: passed
- `cargo test -p nepl-core --test resource_ir typecheck_rejects_mem_ptr_field_access_with_owner_field_boundary -- --exact --nocapture`: passed
- `cargo test -p nepl-core --test resource_ir typecheck_rejects_region_token_field_access_outside_memory_boundary -- --exact --nocapture`: passed
- `cargo check -p nepl-core`: passed
- `node nodesrc/test_static_check_boundary_responsibility.js`: passed
- `trunk build`: passed
- `node nodesrc/run_doctest.js -i stdlib/core/mem/pointer/region.nepl -n 1 --dist web/dist`: passed
- `node nodesrc/run_doctest.js -i stdlib/alloc/collections/adjacency_matrix/api/create.nepl -n 1 --dist web/dist`: compiler boundary errors are gone; stale `eq` doctest issue remains
