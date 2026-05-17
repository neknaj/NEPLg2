---
id: ISS-20260517T071912004Z-COMPILER-MEMORY-TYPE-DEFINITION-SHAP-EE5DF0E2
title: "compiler memory type definition shape is duplicated across source proof and typecheck"
area: core
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-17
updated: 2026-05-17
target: "nepl-core/src/resource_primitives.rs, nepl-core/src/source_capability/memory_type_definition.rs, nepl-core/src/typecheck/compiler_memory_type.rs"
---

# ISS-20260517T071912004Z-COMPILER-MEMORY-TYPE-DEFINITION-SHAP-EE5DF0E2: compiler memory type definition shape is duplicated across source proof and typecheck

## 概要

MemPtr/RegionToken definition shape is checked twice: source_capability/memory_type_definition.rs matches AST field names/types, while typecheck/compiler_memory_type.rs separately matches TypeCtx field names/types. The raw/size field contract is therefore not owned by a single typed compiler-memory shape spec.

## 対象

- `nepl-core/src/resource_primitives.rs, nepl-core/src/source_capability/memory_type_definition.rs, nepl-core/src/typecheck/compiler_memory_type.rs`

## 根拠

- `source_capability/memory_type_definition.rs` は AST 上の `MemPtr` / `RegionToken` definition を `is_mem_ptr_definition` / `is_region_token_definition` で個別に判定していた。
- `typecheck/compiler_memory_type.rs` は typecheck 後の `field_names` / `TypeId` を別の `match CompilerMemoryType` で再判定していた。
- どちらも `raw` / `size` field 名と i32 field 型を別々に保持しており、compiler memory representation の変更時に source proof と typed proof が drift し得た。

## 問題

MemPtr/RegionToken definition shape is checked twice: source_capability/memory_type_definition.rs matches AST field names/types, while typecheck/compiler_memory_type.rs separately matches TypeCtx field names/types. The raw/size field contract is therefore not owned by a single typed compiler-memory shape spec.

## 影響

Source proof and typecheck can drift: a future change to compiler memory representation can update one checker while the other still accepts or rejects stale shapes. This weakens the memory-safety boundary around MemPtr = non-owning pointer and RegionToken = free obligation owner.

## 修正方針

Move field layout/name contracts onto a shared CompilerMemoryTypeShape/field spec in resource_primitives, and make both source capability AST proof and typecheck TypeId proof consume that spec with exhaustive CompilerMemoryType matching.

## 対応内容

- `CompilerMemoryFieldSpec::{RawI32, SizeI32}` と `compiler_memory_type_field_specs` を `resource_primitives.rs` に追加し、`MemPtr` / `RegionToken` の field 名・順序・型要求を shared typed spec に集約した。
- source capability AST proof は per-type checker を削除し、`compiler_memory_type_field_specs(memory_type)` を読んで field shape を検査する。
- typecheck 側の typed struct shape proof も同じ field spec を消費し、`raw` / `size` spelling の local duplicate を削除した。
- `source_capability.rs` の不要になった compiler memory primitive classifier re-export を削除した。
- source policy に、source proof / typecheck proof が local `raw` / `size` spelling と per-memory-type shape checker を再導入しない検査を追加した。

## 検証

cargo test -p nepl-core compiler_memory_type --lib -- --nocapture
cargo test -p nepl-core --test resource_ir typecheck_requires_struct_shape_for_compiler_memory_type_registration -- --exact --nocapture
node nodesrc/test_static_check_boundary_responsibility.js
node nodesrc/issues.js check --dir issues
