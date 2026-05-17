---
id: ISS-20260517T025208836Z-RESOURCE-IR-RAW-ADDRESS-HELPER-PROOF-DAB3E3C8
title: "Resource IR raw address helper proof has overlapping lowering authorities"
area: core
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-17
updated: 2026-05-17
target: "nepl-core/src/resource_primitives.rs; nepl-core/src/resource/lower.rs; nepl-core/src/resource/lower_raw_address.rs; nepl-core/src/resource/lower_raw_address_return.rs; nepl-core/tests/resource_ir.rs"
---

# ISS-20260517T025208836Z-RESOURCE-IR-RAW-ADDRESS-HELPER-PROOF-DAB3E3C8: Resource IR raw address helper proof has overlapping lowering authorities

## 概要

MemoryHelperPrimitive calls such as mem_ptr_wrap, mem_ptr_add, region_new, region_ptr_at, and region_token_raw_ref have dedicated Resource IR lowering, but lower.rs still feeds the same calls through the generic named raw-address proof path and the transparent return projection path can also revisit helper bodies.

## 対象

- `nepl-core/src/resource_primitives.rs; nepl-core/src/resource/lower.rs; nepl-core/src/resource/lower_raw_address.rs; nepl-core/src/resource/lower_raw_address_return.rs; nepl-core/tests/resource_ir.rs`

## 根拠

- `MemoryHelperPrimitive` の `MemPtr` / `RegionToken` wrapper call は `push_core_mem_wrapper_semantics` で `ResourceOp::RawAddressAlias` または `ResourceOp::RawAddressView` に下げられる。
- 修正前の `push_direct_call_skeleton` は、この dedicated lowering を実行した直後に `push_named_raw_address_semantics` も実行していたため、同じ call-site に対して dedicated helper lowering と generic named helper proof が重なっていた。
- `lower_raw_address_return.rs` 側の transparent return projection も `has_dedicated_raw_address_lowering` に依存するため、この role classifier が一部 helper だけを表すと、helper 追加時に Resource IR lowering authority が再び分裂する。

## 問題

MemoryHelperPrimitive calls such as mem_ptr_wrap, mem_ptr_add, region_new, region_ptr_at, and region_token_raw_ref have dedicated Resource IR lowering, but lower.rs still feeds the same calls through the generic named raw-address proof path and the transparent return projection path can also revisit helper bodies.

## 影響

The same raw address fact can be emitted by multiple proof paths, making static-check behavior harder to audit and allowing future helper additions to depend on accidental duplicate or fallback proofs instead of a single enum-governed authority.

## 修正方針

Make dedicated MemoryHelperPrimitive lowering report whether it handled a call, skip generic named proof for those calls, and make transparent return projection skip every helper with dedicated call lowering. Add a regression that counts raw-address facts for representative helpers.

## 検証

- `cargo fmt -p nepl-core`: passed
- `node nodesrc/test_static_check_boundary_responsibility.js`: passed
- `cargo test -p nepl-core resource_ir_lowers_dedicated_memory_helpers_once_per_call --test resource_ir -- --nocapture`: passed
- `cargo test -p nepl-core memory_helper_primitive_marks_single_resource_lowering_authority -- --nocapture`: passed
- `cargo check -p nepl-core`: passed
