---
id: ISS-20260520T153639309Z-REGIONTOKEN-CONSTRUCTION-SHARES-GENE-C5BF72D0
title: "RegionToken construction shares generic raw-address alias capability"
area: compiler
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-20
updated: 2026-05-20
target: "nepl-core/src/source_map.rs, nepl-core/src/source_capability/**, nepl-core/src/resource/model.rs, nepl-core/src/resource/lower_raw_address.rs"
---

# ISS-20260520T153639309Z-REGIONTOKEN-CONSTRUCTION-SHARES-GENE-C5BF72D0: RegionToken construction shares generic raw-address alias capability

## 概要

region_new currently proves the same RawAddressAliasBoundary as mem_ptr_wrap. That collapses owner-token construction and non-owning raw pointer wrapping into one capability, so the static checker cannot exhaustively distinguish compiler-issued owner token issuance from ordinary raw-address aliasing.

## 対象

- `nepl-core/src/source_map.rs, nepl-core/src/source_capability/**, nepl-core/src/resource/model.rs, nepl-core/src/resource/lower_raw_address.rs`

## 根拠

- `region_new` は `RegionToken<T>` の free obligation owner を構築する境界であり、`mem_ptr_wrap` のような non-owning `MemPtr<T>` projection とは性質が異なる。
- 修正前は `MemoryHelperPrimitive::RegionNew` が `RawAddressAliasBoundary` evidence として扱われ、source proof と Resource IR effect diagnostic の両方で generic raw-address alias と同じ経路を通っていた。
- non-Copy collection payload support では compiler-issued owner token と initialized/drop state を接続するため、owner token construction を generic alias capability に混ぜたままにすると、後続の proof 境界が `RegionToken` 生成だけを網羅的に検査できない。

## 問題

region_new currently proves the same RawAddressAliasBoundary as mem_ptr_wrap. That collapses owner-token construction and non-owning raw pointer wrapping into one capability, so the static checker cannot exhaustively distinguish compiler-issued owner token issuance from ordinary raw-address aliasing.

## 影響

Non-Copy collection payload support needs compiler-issued owner tokens. If owner token construction remains a generic raw-address alias, future Resource IR and source policy checks can accidentally permit owner construction through the wrong boundary or fail to detect proof-program mistakes.

## 修正方針

Introduce a typed owner-token construction source capability and Resource IR alias kind for region_new, keep mem_ptr_wrap on the generic raw-address alias path, and make effect-boundary filtering use the owner-token construction use-site explicitly.

## 検証

Focused Rust tests for source capability and effect-boundary filtering must show region_new no longer grants generic RawAddressAliasBoundary while valid stdlib owner-token construction remains allowed.

## 2026-05-20 Agent 1 修正

`region_new` 用に `OwnerTokenConstructBoundary` を追加し、`mem_ptr_wrap` の `RawAddressAliasBoundary` から分離した。source capability proof fact、use-site capability、loader の test extension、raw builtin evidence gate は owner-token construction を専用 enum variant として扱う。

Resource IR では `RawAddressAliasKind::OwnerTokenConstruct` を追加し、`region_new` lowering だけがこの kind を出すようにした。effect gate は `OwnerTokenConstruct` を generic raw alias capability では許可せず、`SourceMap::owner_token_construct_boundary_allowed_at` の proof だけで通す。`mem_ptr_wrap` は従来どおり generic raw-address alias boundary に残る。

この修正は `RegionToken` を完全な compiler-issued owner token に置き換えるものではない。次段階で `OwnedRegion` / `OwnedBuffer` / initialized cell state へ進むために、owner-token construction の入口を generic alias から切り離した compiler-core 側の前提整備である。

関連設計:

- [NEPLg2 静的検査の複雑化解消計画](https://github.com/neknaj/NEPLg2/blob/main/doc/neplg2/static_check_complexity_reduction_plan.md)
- [stdlib collection/mem/string と静的検査の安全設計](https://github.com/neknaj/NEPLg2/blob/main/doc/neplg2/stdlib_collection_mem_string_static_safety_design.md)
- [Non-Copy collection payload support issue](https://github.com/neknaj/NEPLg2/blob/main/issues/items/ISS-20260520T152218366Z-NON-COPY-COLLECTION-PAYLOAD-SUPPORT--A6A88543.md)

focused verification:

- `cargo test -p nepl-core source_map::tests --lib`: 3/3 passed
- `cargo test -p nepl-core resource_primitives::tests --lib`: 7/7 passed
- `cargo test -p nepl-core loader::tests::raw_memory_boundary --lib`: 27/27 passed
- `cargo test -p nepl-core resource_effect_gate_keeps_owner_token_construct_separate_from_raw_alias --lib`: 1/1 passed
- `cargo test -p nepl-core --test resource_ir region_new`: 2/2 passed
- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_rejects_region_token_forged_from -- --test-threads=1`: 5/5 passed
- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_accepts_returned_allocated_region_token`: 1/1 passed
