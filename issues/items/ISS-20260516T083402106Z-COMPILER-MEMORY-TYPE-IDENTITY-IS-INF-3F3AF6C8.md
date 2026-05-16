---
id: ISS-20260516T083402106Z-COMPILER-MEMORY-TYPE-IDENTITY-IS-INF-3F3AF6C8
title: "compiler memory type identity is inferred from names without source proof"
area: core
status: fixed
resolved: true
priority: P0
type: bug
created: 2026-05-16
updated: 2026-05-16
target: nepl-core/src/resource_primitives.rs
---

# ISS-20260516T083402106Z-COMPILER-MEMORY-TYPE-IDENTITY-IS-INF-3F3AF6C8: compiler memory type identity is inferred from names without source proof

## 概要

Resource IR classifies MemPtr and RegionToken by nominal type name through compiler_memory_type_of_type. A same-name struct that is not backed by SourceCapabilities can be treated as a compiler memory type by Resource IR, so static checks rely on a name convention rather than a proven compiler-owned type identity.

## 対象

- `nepl-core/src/resource_primitives.rs`
- `nepl-core/src/typecheck/driver.rs`
- `nepl-core/src/types.rs`

## 根拠

- `compiler_memory_type_of_type` が `TypeKind::Struct { name, .. }` の name から `MemPtr` / `RegionToken` を判定していた。
- Resource IR の raw pointer / owner token 判定が SourceCapabilities で証明された stdlib/compiler-owned 定義 identity ではなく、型名 convention に依存していた。
- 同名 struct を作っただけで compiler memory type として扱われる経路があり、静的検査の境界が source proof / type identity ではなく文字列に弱くなっていた。

## 問題

Resource IR classifies MemPtr and RegionToken by nominal type name through compiler_memory_type_of_type. A same-name struct that is not backed by SourceCapabilities can be treated as a compiler memory type by Resource IR, so static checks rely on a name convention rather than a proven compiler-owned type identity.

## 影響

Memory/resource proof can attach raw pointer or owner token semantics to types that were not proven as compiler-owned memory types. This weakens the static-check boundary and makes checker behavior depend on string names instead of source/type evidence.

## 修正方針

Record compiler memory type identity in TypeCtx only when typecheck registers a struct whose source file has CompilerMemoryTypeDefinition capability. Make resource_primitives query that typed identity instead of struct names, and add regressions/policy checks that same-name unmarked structs are not compiler memory types.

## 対応

- `TypeCtx` に `compiler_memory_types` identity registry を追加し、checkpoint / rollback / clone に含めた。
- typecheck の struct 登録時に、source map の `CompilerMemoryTypeDefinition` capability を持つ定義だけを `TypeCtx::mark_compiler_memory_type` で登録するようにした。
- `resource_primitives::compiler_memory_type_of_type` は struct 名を見ず、`TypeCtx::compiler_memory_type` の証明済み identity だけを読むようにした。
- `MemPtr` / `RegionToken` と同名でも、証明済み identity でない struct は raw pointer / owner token として扱わない regression を追加した。
- `nodesrc/test_resource_checker_responsibility.js` に、Resource IR が compiler memory type を型名から推論する実装へ戻ることを拒否する policy を追加した。

## 検証

- `cargo test -p nepl-core resource_primitives::tests::same_name_structs_are_not_memory_types_without_proven_identity -- --exact --nocapture`
- `cargo test -p nepl-core loader::tests::actual_core_mem_types_expose_both_compiler_memory_type_capabilities -- --exact --nocapture`
- `cargo test -p nepl-core loader::tests::imported_region_token_span_keeps_owner_token_capability -- --exact --nocapture`
- `cargo test -p nepl-core --test resource_ir typecheck_marks_imported_compiler_memory_types_in_type_context -- --exact --nocapture`
- `cargo test -p nepl-core --test resource_ir typecheck_rejects_region_token_struct_constructor_outside_memory_boundary -- --exact --nocapture`
- `cargo test -p nepl-core --test resource_ir typecheck_allows_user_struct_named_mem_ptr -- --exact --nocapture`
- `cargo test -p nepl-core --test resource_ir typecheck_allows_user_struct_named_region_token -- --exact --nocapture`
- `node nodesrc/test_resource_checker_responsibility.js`
- `cargo fmt -p nepl-core --check`
- `cargo check -p nepl-core`
- `node nodesrc/issues.js check --dir issues`
- `git diff --check`
