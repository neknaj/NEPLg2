---
id: ISS-20260517T090028643Z-COMPILER-MEMORY-FIELD-PROOF-IS-GRANT-4EB3DABF
title: "Compiler memory field proof is granted by generic owner field evidence"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-17
updated: 2026-05-17
target: "nepl-core/src/source_capability/proof_builder.rs, nepl-core/src/source_capability/proof.rs, nepl-core/src/source_map.rs, nepl-core/src/typecheck/field_access.rs, nepl-core/src/loader.rs, nepl-core/tests/resource_ir.rs, nodesrc/test_static_check_boundary_responsibility.js, doc/neplg2/static_check_complexity_reduction_plan.md"
---

# ISS-20260517T090028643Z-COMPILER-MEMORY-FIELD-PROOF-IS-GRANT-4EB3DABF: Compiler memory field proof is granted by generic owner field evidence

## 概要

OwnerAggregateFieldBoundary and CompilerMemoryFieldBoundary are represented as separate SourceCapabilityUseSite variants, but owner aggregate FieldAccessor evidence currently inserts both. As a result a compiler-owned stdlib use of core/field can prove compiler-memory field authority without proving that the use site is a direct MemPtr/RegionToken representation-field access.

## 対象

- `nepl-core/src/source_capability/proof_builder.rs, nepl-core/src/source_capability/proof.rs, nepl-core/src/source_map.rs, nepl-core/src/typecheck/field_access.rs, nepl-core/src/loader.rs, nepl-core/tests/resource_ir.rs, nodesrc/test_static_check_boundary_responsibility.js, doc/neplg2/static_check_complexity_reduction_plan.md`

## 根拠

- `SourceCapabilityUseSite` は `OwnerAggregateFieldBoundary` と `CompilerMemoryFieldBoundary` を分離していたが、`proof_builder.rs` の `OwnerAggregateCapabilityEvidence::FieldAccessor` arm が両方を同時に挿入していた。
- `typecheck/field_access.rs` は projected field type が `MemPtr<T>` の場合にも、同じ span の `CompilerMemoryFieldBoundary` を消費していた。そのため compiler-owned stdlib source では `PtrHolder.ptr` のような普通の aggregate field から `MemPtr<T>` を抜く use site が compiler memory field proof を満たし得た。
- `loader.rs` / `resource_ir.rs` には「owner aggregate field proof は compiler memory field proof ではない」ことを compiler-owned source で固定する回帰テストが不足していた。

## 問題

OwnerAggregateFieldBoundary and CompilerMemoryFieldBoundary are represented as separate SourceCapabilityUseSite variants, but owner aggregate FieldAccessor evidence currently inserts both. As a result a compiler-owned stdlib use of core/field can prove compiler-memory field authority without proving that the use site is a direct MemPtr/RegionToken representation-field access.

## 影響

This weakens the static-check proof model: a use site that only proves ordinary owner aggregate field access can satisfy the raw pointer field gate. It also makes the static-check program harder to audit because two distinct capabilities are derived from one evidence enum arm instead of an exhaustive capability-specific proof.

## 修正方針

Split compiler-memory field proof from owner-aggregate field proof. Attach CompilerMemoryFieldBoundary only to field accessor use sites whose selector is a compiler-memory representation field, and make typecheck consume that proof only for direct compiler-memory base field access rather than aggregate fields whose payload type is MemPtr.

## 検証

Add compiler-owned regression for PtrHolder.ptr: MemPtr extraction to remain rejected, add source capability tests for owner field evidence not granting compiler-memory field authority, run focused nepl-core tests, static-check boundary policy, issues check, cargo check, trunk build, and relevant NMD tests.

## 対応

- `CompilerMemoryField` enum を追加し、`CompilerMemoryFieldBoundary` を exact span だけでなく `Raw` / `Size` の field-specific proof artifact にした。
- `owner_aggregate` の `FieldAccessor` evidence から `CompilerMemoryFieldBoundary` を挿入する経路を削除した。
- `compiler_memory_field` source capability domain を追加し、`core/field` 由来または field intrinsic 由来の `get` / `get_ref` が compiler memory representation field selector (`raw` / `size`) を使う場合だけ compiler memory field proof を発行するようにした。
- typecheck の raw pointer field gate は direct `MemPtr.raw` など、base type 自身が compiler memory type で field selector が proof と一致する場合だけ `CompilerMemoryFieldBoundary` を消費するようにした。
- aggregate field の projected type が `MemPtr<T>` の場合は、owner aggregate field proof や compiler memory field proofで通さず、raw pointer payload extraction として拒否を維持するようにした。
- source policy に、owner aggregate evidence arm が compiler memory field proof を発行しないこと、compiler memory field proof が別 typed domain であることを追加した。

## 検証結果

- `cargo fmt -p nepl-core --check`
- `cargo check -p nepl-core`
- `node nodesrc/test_static_check_boundary_responsibility.js`
- `cargo test -p nepl-core compiler_memory_field_boundary_requires_representation_field_selector --lib -- --nocapture`
- `cargo test -p nepl-core source_map::tests --lib -- --nocapture`
- `cargo test -p nepl-core owner_aggregate_boundary --lib -- --nocapture`
- `cargo test -p nepl-core --test resource_ir typecheck_rejects_compiler_owned_aggregate_mem_ptr_payload_field_access -- --exact --nocapture`
- `cargo test -p nepl-core --test resource_ir typecheck_allows_region_token_field_access_with_owner_field_boundary -- --exact --nocapture`
- `cargo test -p nepl-core --test resource_ir typecheck_rejects_mem_ptr_field_access_outside_compiler_memory_field_boundary -- --exact --nocapture`
- `trunk build`
- `node nodesrc/tests.js -i stdlib/core/mem/internal.nepl --no-tree -o tmp/agent1-compiler-memory-field-core-mem-internal.json -j 1` (`total=4`, `passed=4`, `failed=0`, `errored=0`)
- `node nodesrc/tests.js -i tests/compiler/move_effect.n.md --no-tree -o tmp/agent1-compiler-memory-field-move-effect.json -j 1` (`total=115`, `passed=115`, `failed=0`, `errored=0`)
