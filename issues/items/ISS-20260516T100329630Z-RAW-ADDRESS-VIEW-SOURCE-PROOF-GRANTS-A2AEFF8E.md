---
id: ISS-20260516T100329630Z-RAW-ADDRESS-VIEW-SOURCE-PROOF-GRANTS-A2AEFF8E
title: "raw address view source proof grants structural memory boundary"
area: core
status: fixed
resolved: true
priority: P0
type: architecture
created: 2026-05-16
updated: 2026-05-16
target: nepl-core/src/source_capability/proof.rs
---

# ISS-20260516T100329630Z-RAW-ADDRESS-VIEW-SOURCE-PROOF-GRANTS-A2AEFF8E: raw address view source proof grants structural memory boundary

## 概要

SourceCapabilityProof treats any MemoryHelperPrimitive call as RawMemoryStructuralBoundary evidence. A stdlib file that merely calls safe raw-address-view helpers such as mem_ptr_addr or region_ptr can then receive permission for compiler memory representation constructors and fields. This conflates address-view proof with direct representation proof and weakens the static checker.

## 対象

- `nepl-core/src/source_capability/proof.rs`

## 根拠

- `nepl-core/src/source_capability/raw_memory/evidence.rs` の旧 `RawMemoryBoundaryEvidence::from_symbol` は `MemoryHelperPrimitive::from_symbol(name).is_some()` を `RawMemoryStructuralBoundary` evidence として扱っていた。
- `nepl-core/src/source_capability/proof.rs` は structural evidence と raw operation helper body evidence を同じ `record_raw_memory_evidence` で function frame に記録していたため、address-view helper の使用も raw helper definition self-operation proof に混ざり得た。
- `nepl-core/src/compiler.rs` の `RawAddressViewOutsideBoundary` suppression も `raw_memory_structural_boundary_allowed` を見ており、representation 直操作と raw address view 利用が capability 上で分かれていなかった。

## 問題

SourceCapabilityProof treats any MemoryHelperPrimitive call as RawMemoryStructuralBoundary evidence. A stdlib file that merely calls safe raw-address-view helpers such as mem_ptr_addr or region_ptr can then receive permission for compiler memory representation constructors and fields. This conflates address-view proof with direct representation proof and weakens the static checker.

## 影響

Static memory boundary is broader than the source property proves. It becomes easier for future stdlib changes to accidentally forge MemPtr or RegionToken internals without the compiler distinguishing raw address view usage from representation access.

## 修正方針

Split source capabilities into raw memory representation structural boundary and raw address view boundary. Grant representation boundary only from restricted constructor evidence / compiler memory definition proof, and grant raw address view boundary from typed memory helper evidence. Typecheck constructors and fields must use the representation boundary; Resource IR raw-address-view diagnostics must use the address-view boundary.

## 検証

Focused loader/source_map tests prove mem_ptr_addr call evidence no longer grants structural boundary but does grant raw address view boundary; constructor evidence still grants structural boundary. Run cargo fmt/check and policy tests.

## 対応内容

- `SourceCapability::RawAddressViewBoundary` と `raw_address_view_boundary_allowed` を追加し、ResourceEffectBoundary の raw address view suppression をこの capability へ接続した。
- raw source evidence を `RawMemoryStructuralEvidence` と `RawAddressViewEvidence` に分割し、restricted constructor は structural boundary、`MemoryHelperPrimitive` は raw-address-view boundary だけを証明するようにした。
- `MemoryHelperPrimitive::is_raw_address_view_boundary_evidence` を exhaustive match として追加し、`RegionNew` のような owner-token helper call が address-view evidence に昇格しないことを型付き registry で固定した。
- raw helper definition self-operation proof を `function_has_raw_operation_evidence` に改め、actual raw operation / raw body evidence だけで更新するようにした。
- regression と source policy を追加し、address-view helper が structural boundary や raw operation capability に昇格しないことを固定した。

## 検証結果

- `cargo fmt -p nepl-core --check`: passed
- `cargo test -p nepl-core loader::tests::raw_memory_boundary_accepts_raw_address_helper_evidence -- --exact --nocapture`: passed
- `cargo test -p nepl-core loader::tests::raw_memory_boundary_does_not_promote_address_view_helper_to_operation_definition -- --exact --nocapture`: passed
- `cargo test -p nepl-core source_map::tests::source_capabilities_are_enum_keyed -- --exact --nocapture`: passed
- `cargo test -p nepl-core compiler::tests::resource_effect_gate_allows_raw_address_view_inside_raw_boundary -- --exact --nocapture`: passed
- `cargo test -p nepl-core loader::tests::raw_memory_boundary_rejects_owner_constructor_helper_as_address_view_evidence -- --exact --nocapture`: passed
- `cargo test -p nepl-core resource_primitives::tests::memory_helper_primitive_separates_address_view_boundary_evidence -- --exact --nocapture`: passed
- `node nodesrc/test_static_check_boundary_responsibility.js`: passed
