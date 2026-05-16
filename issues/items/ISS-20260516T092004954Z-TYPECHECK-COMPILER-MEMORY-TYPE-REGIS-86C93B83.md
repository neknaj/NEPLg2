---
id: ISS-20260516T092004954Z-TYPECHECK-COMPILER-MEMORY-TYPE-REGIS-86C93B83
title: "typecheck compiler memory type registration trusts file capability without struct shape proof"
area: core
status: fixed
resolved: true
priority: P0
type: bug
created: 2026-05-16
updated: 2026-05-16
target: "nepl-core/src/typecheck/compiler_memory_type.rs, nepl-core/src/typecheck/driver.rs, nepl-core/tests/resource_ir.rs, nodesrc/test_static_check_boundary_responsibility.js"
---

# ISS-20260516T092004954Z-TYPECHECK-COMPILER-MEMORY-TYPE-REGIS-86C93B83: typecheck compiler memory type registration trusts file capability without struct shape proof

## 概要

After SourceCapabilities were moved to source-code evidence, typecheck still registers MemPtr / RegionToken compiler memory identities by combining only the struct name with a file-level CompilerMemoryTypeDefinition capability. A stale or manually injected SourceMap capability can therefore mark a same-name malformed user struct as compiler memory even though the struct definition itself does not prove the required shape at the typecheck boundary.

## 対象

- `nepl-core/src/typecheck/compiler_memory_type.rs`
- `nepl-core/src/typecheck/driver.rs`
- `nepl-core/tests/resource_ir.rs`
- `nodesrc/test_static_check_boundary_responsibility.js`

## 根拠

- `nepl-core/src/typecheck/driver.rs` は `TypeCtx::mark_compiler_memory_type` の前に、struct 名と `SourceMap::compiler_memory_type_definition_allowed` だけを組み合わせていた。
- `SourceCapabilities` は source proof から生成されるが、`SourceMap::add_with_capabilities` / `set_capabilities` により file capability は外部から注入可能であり、typecheck boundary 側でも現在の struct 定義そのものを検証しないと proof connection が弱い。
- `MemPtr` / `RegionToken` の semantic identity は Resource IR が後段で使うため、登録時の shape proof は warning や best-effort ではなく hard invariant にする必要がある。

## 問題

After SourceCapabilities were moved to source-code evidence, typecheck still registers MemPtr / RegionToken compiler memory identities by combining only the struct name with a file-level CompilerMemoryTypeDefinition capability. A stale or manually injected SourceMap capability can therefore mark a same-name malformed user struct as compiler memory even though the struct definition itself does not prove the required shape at the typecheck boundary.

## 影響

Static-check correctness depends on file capability state instead of the actual typed struct definition. This weakens the source/type proof connection and can make Resource IR treat malformed same-name structs as raw pointer or owner token identities.

## 修正方針

Make typecheck consume both the SourceMap capability and the current StructDef typed shape before calling TypeCtx::mark_compiler_memory_type. Keep constructor-name classification only as the primitive-kind lookup and require public one-parameter MemPtr(raw:i32) / RegionToken(raw:i32,size:i32) shape at the registration boundary.

## 検証

Add a regression that injects a RawPointer capability for a malformed MemPtr struct and verifies TypeCtx does not register it as compiler memory. Extend source policy so typecheck cannot regress to name+file-capability registration.

## 2026-05-16 Agent 1 修正

`compiler_memory_type_definition_allowed` を `typecheck/compiler_memory_type.rs` に分離し、登録条件を次の 2 段証明に変更した。

- `SourceMap` が該当 file に `CompilerMemoryTypeDefinition` capability を持つこと。
- 現在 typecheck 中の `StructDef` が typed shape として compiler memory type の契約を満たすこと。

`RawPointer` は `pub struct MemPtr<.T>: raw <i32>`、`OwnerToken` は `pub struct RegionToken<.T>: raw <i32>, size <i32>` の形だけを認める。構築子名の分類は primitive kind lookup に限定し、`TypeCtx::mark_compiler_memory_type` は source capability と typed struct shape が両方成立した場合だけ呼ぶ。

回帰テストとして、malformed `MemPtr` struct に `RawPointer` capability を手動注入しても typecheck が user struct として扱い、`TypeCtx::compiler_memory_type` に登録しないことを固定した。source policy も、driver が現在の `StructDef` / typed field / type params を registration helper に渡し、旧 `&s.name.name + span.file_id` だけの登録へ戻らないことを検査する。

検証:

- `cargo fmt -p nepl-core --check`
- `cargo test -p nepl-core --test resource_ir typecheck_requires_struct_shape_for_compiler_memory_type_registration -- --exact --nocapture`
- `cargo check -p nepl-core`
- `node nodesrc/test_static_check_boundary_responsibility.js`
- `node nodesrc/test_resource_checker_responsibility.js`
