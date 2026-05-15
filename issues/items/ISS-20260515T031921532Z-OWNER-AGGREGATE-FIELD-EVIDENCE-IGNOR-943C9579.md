---
id: ISS-20260515T031921532Z-OWNER-AGGREGATE-FIELD-EVIDENCE-IGNOR-943C9579
title: "Owner aggregate field evidence ignores get_field_ref intrinsics"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-15
updated: 2026-05-15
target: "nepl-core/src/source_capability/owner_aggregate.rs, nepl-core/src/loader.rs, nodesrc/test_static_check_boundary_responsibility.js"
---

# ISS-20260515T031921532Z-OWNER-AGGREGATE-FIELD-EVIDENCE-IGNOR-943C9579: Owner aggregate field evidence ignores get_field_ref intrinsics

## 概要

owner aggregate field source capability scanner recurses through intrinsic arguments but does not classify get_field_ref/get_field intrinsics themselves as field accessor evidence, so compiler-owned stdlib modules using intrinsic field references fail owner aggregate field checks after a fresh trunk build.

## 対象

- `nepl-core/src/source_capability/owner_aggregate.rs, nepl-core/src/loader.rs, nodesrc/test_static_check_boundary_responsibility.js`

## 根拠

- `owner_aggregate` source evidence walker は `PrefixItem::Intrinsic` の引数だけを再帰し、`#intrinsic "get_field_ref"` / `#intrinsic "get_field"` 自体を field accessor evidence として分類していなかった。
- 同じ source に `struct MemPtr` / `struct RegionToken` などの type definition が存在する場合、scope shadow 判定が type definition まで value shadow として扱い、同名 constructor evidence を消していた。
- focused doctest の前段階で `type.owner_aggregate.field_access_restricted` が発生し、compiler-owned `core/mem/types.nepl` の正当な field accessor 実装が拒否された。

## 問題

owner aggregate field source capability scanner recurses through intrinsic arguments but does not classify get_field_ref/get_field intrinsics themselves as field accessor evidence, so compiler-owned stdlib modules using intrinsic field references fail owner aggregate field checks after a fresh trunk build.

## 影響

Stage 6 owner-backed aggregate field projection gate rejects valid compiler-owned stdlib implementation modules such as core/mem/types.nepl, causing focused mem/Vec doctests to fail and hiding the intended boundary between stdlib implementation authority and user source.

## 修正方針

Treat get_field/get_field_ref intrinsics as enum field-accessor evidence while preserving constructor-name evidence and user-source denial; add loader and source-policy regressions.

## 解決内容

- `PrefixItem::Intrinsic` で builtin owner aggregate field accessor evidence を収集してから引数を再帰するようにした。
- `SourceCapabilityScope` は top-level `struct` / `enum` / `trait` を value shadow として扱わず、`fn` / `fn alias` / local binding / parameter / match binding だけを shadow source にする。
- loader regression と source policy regression に、compiler-owned `get_field_ref` intrinsic と same-module struct constructor evidence の両方を追加した。

## 検証

- `cargo test -p nepl-core owner_aggregate_boundary --lib -- --nocapture`
- `node nodesrc/test_static_check_boundary_responsibility.js`
- `trunk build`
- `node nodesrc/tests.js -i stdlib/core/mem.nepl -i stdlib/core/mem/types.nepl -i stdlib/core/mem/internal.nepl -i stdlib/core/mem/pointer/alloc.nepl -i stdlib/core/mem/pointer/region.nepl -i stdlib/core/mem/pointer/scalar.nepl -i stdlib/alloc/collections/vec/storage/api.nepl -i stdlib/alloc/collections/vec/storage/view.nepl -i stdlib/alloc/collections/vec/storage/cleanup.nepl --no-tree -o tmp/agent1-raw-operation-specific-capability-doctests-after-identity-origin.json -j 1 --dist web/dist --assert-io`
