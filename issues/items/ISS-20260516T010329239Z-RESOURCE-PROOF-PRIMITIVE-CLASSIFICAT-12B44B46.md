---
id: ISS-20260516T010329239Z-RESOURCE-PROOF-PRIMITIVE-CLASSIFICAT-12B44B46
title: "Resource proof primitive classification is scattered across name-based checks"
area: core
status: open
resolved: false
priority: P1
type: architecture
created: 2026-05-16
updated: 2026-05-16
target: "nepl-core/src/resource/**, nepl-core/src/source_capability/**, nepl-core/src/effects.rs, nodesrc/test_static_check_boundary_responsibility.js"
---

# ISS-20260516T010329239Z-RESOURCE-PROOF-PRIMITIVE-CLASSIFICAT-12B44B46: Resource proof primitive classification is scattered across name-based checks

## 概要

Static check authority is moving to Resource IR, but memory primitive roles are still recognized by scattered string-name checks such as MemPtr, RegionToken, region_new, region_ptr, and mem_ptr_addr. This is not a stdlib module allowlist, but it is still not a sufficiently centralized generic prover boundary.

## 対象

- `nepl-core/src/resource/**, nepl-core/src/source_capability/**, nepl-core/src/effects.rs, nodesrc/test_static_check_boundary_responsibility.js`

## 根拠

- `nepl-core/src/loader.rs` は configured stdlib root 配下の parsed source に source evidence がある場合だけ `SourceCapability` を付与しており、旧 `RAW_MEMORY_BOUNDARY_STDLIB_PATHS` のような exact module allowlist は残っていない。
- `nepl-core/src/source_map.rs` の `SourceCapability` / `CompilerMemoryType` と、`nepl-core/src/effects.rs` の `RawMemoryOp` / `RawBodyMemoryOp` / `ExternalIoOp` / `NondetOp` は enum-first であり、operation-level gate としては妥当な方向である。
- `nepl-core/src/typecheck/copy_capability.rs` の owner-backed aggregate 判定は `Vec` / `HashMap` などの stdlib 名 allowlist ではなく、owner token policy と field 型から fixed-point で導出されている。
- 一方で `nepl-core/src/source_capability/memory_type_definition.rs` は `"MemPtr"` / `"RegionToken"` を直接 constructor name として分類している。
- `nepl-core/src/source_capability/raw_memory/evidence.rs` は `"mem_ptr_addr"` / `"region_new"` / `"region_ptr"` などを direct string match で raw-address boundary evidence にしている。
- `nepl-core/src/resource/lower_raw_address.rs`、`lower_raw_address_place.rs`、`lower_raw_address_return.rs`、`effect_return_escape.rs`、`effect_return_owner_type.rs`、`owner_flow.rs`、`owner_summary_leaf.rs`、`place_utils.rs` などに `MemPtr` / `RegionToken` / `region_new` / `region_ptr` / `mem_ptr_addr` の判定が分散している。
- `nodesrc/test_static_check_boundary_responsibility.js` は module allowlist 復活や capability split の退行を検出するが、primitive string 判定が registry 外へ増えることを十分に禁止していない。

## 問題

Static check authority is moving to Resource IR, but memory primitive roles are still recognized by scattered string-name checks such as MemPtr, RegionToken, region_new, region_ptr, and mem_ptr_addr. This is not a stdlib module allowlist, but it is still not a sufficiently centralized generic prover boundary.

## 影響

Adding OwnedBuffer, OwnedRegion, or later self-host memory primitives can require multiple ad hoc checker edits. Missing one edit may not be caught by Rust exhaustiveness because several sites still branch on strings, making the static-check program itself harder to statically verify.

## 修正方針

Introduce a central typed resource primitive registry that classifies compiler memory types and memory helper semantics once, preferably after type resolution using TypeId or definition identity. Resource IR lowering and owner/cell/effect checkers must consume typed enum variants through exhaustive match. SourceCapability should remain an authority/provenance gate only, not the semantic proof engine. Add source policy that rejects direct primitive string checks outside the registry.

この修正では stdlib module ごとの証明器を作らない。`Vec`、`HashMap`、`ByteBuilder` などの個別 collection / string module を compiler に登録して許可する設計は禁止する。compiler が持つべきなのは、型解決済みの definition identity から `NonOwningPointer`、`FreeObligationOwner`、`RawAddressProjection`、`OwnedStorageConstructor`、`StorageFree`、`RawLoadStore` のような typed primitive property を得る単一の registry / prover である。

実装順は次の通りとする。

1. `CompilerMemoryType` を `ResourcePrimitiveType` へ拡張し、`RawPointer` / `OwnerToken` だけでなく今後の `OwnedBuffer` / `OwnedBytes` / `OwnedRegion` を追加できる型付き分類にする。
2. core memory helper 名の string match を registry module へ集約し、Resource IR lowering は `MemoryHelperPrimitive` enum を受け取って `match` する。
3. Resource checker 内の `is_named_struct_type(..., "MemPtr")` / `name == "RegionToken"` などを registry query へ置換する。
4. source capability scanner は「この source に privileged primitive を実装する authority があるか」だけを判定し、semantic proof は typecheck 後の typed registry + Resource IR に移す。
5. source policy に registry 外の `MemPtr` / `RegionToken` / `region_*` / `mem_ptr_*` direct string 判定を禁止する監視を追加する。

この issue は `Vec` の `OwnedBuffer` 化より先に確認する。`OwnedBuffer` を追加するたびに Resource IR の複数箇所へ名前判定を足すと、静的検査大規模修正の目的である「汎用的で強力な証明器」から外れるためである。

## 検証

Run focused Resource IR/effect/typecheck tests, nodesrc static-check responsibility policy, and regressions for raw boundary, owner token construction, raw address projection, and no module-specific allowlist. Verify new primitive variants force compile-time updates through exhaustive matches.
