---
id: ISS-20260516T094111889Z-TYPECHECK-FIELD-ACCESS-USES-STRUCT-P-44D67554
title: "typecheck field access uses struct policy instead of proven compiler memory type identity"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-16
updated: 2026-05-16
target: "nepl-core/src/typecheck/field_access.rs; nodesrc/test_static_check_boundary_responsibility.js; doc/neplg2/static_check_complexity_reduction_plan.md"
---

# ISS-20260516T094111889Z-TYPECHECK-FIELD-ACCESS-USES-STRUCT-P-44D67554: typecheck field access uses struct policy instead of proven compiler memory type identity

## 概要

Compiler memory field access restriction still derives MemPtr / RegionToken authority from StructConstructorPolicy through the struct name map. After compiler memory identity moved to SourceCapability-backed TypeCtx registration, this leaves field access on a weaker semantic proof boundary than Resource IR construct/lowering paths.

## 対象

- `nepl-core/src/typecheck/field_access.rs; nodesrc/test_static_check_boundary_responsibility.js; doc/neplg2/static_check_complexity_reduction_plan.md`

## 根拠

- `nepl-core/src/typecheck/field_access.rs` の `restricted_struct_constructor_for_field_access` は、修正前に `TypeKind::Struct { name, .. } | TypeKind::Named(name)` から `self.structs.get(&name)?.constructor_policy` を読み、`StructConstructorPolicy::RawMemoryBoundaryOnly` を compiler memory field access restriction の根拠にしていた。
- `ISS-20260516T083402106Z-COMPILER-MEMORY-TYPE-IDENTITY-IS-INF-3F3AF6C8` と `ISS-20260516T092004954Z-TYPECHECK-COMPILER-MEMORY-TYPE-REGIS-86C93B83` で、compiler memory type の semantic identity は SourceCapability-backed TypeCtx registration へ移っている。
- focused test 中に、`RegionToken` への不正 field 名 access が `type.owner_token.field_access_restricted` より先に `FieldInvalidAccess` を返す経路も確認した。restricted compiler memory type では field 名の正否を通常 source に露出する前に境界検査する必要がある。

## 問題

Compiler memory field access restriction still derives MemPtr / RegionToken authority from StructConstructorPolicy through the struct name map. After compiler memory identity moved to SourceCapability-backed TypeCtx registration, this leaves field access on a weaker semantic proof boundary than Resource IR construct/lowering paths.

## 影響

Static checks can drift: future changes may keep Resource IR on proven TypeCtx identity while field access is still governed by name/policy metadata. That weakens auditability of memory safety checks and makes regressions harder to catch through enum/match-based proof boundaries.

## 修正方針

Make typecheck field-access restriction query the central compiler memory type identity from TypeCtx, then map CompilerMemoryType to RestrictedStructConstructor with an exhaustive match. Keep SourceMap capability as authority for definition modules, but use TypeCtx identity for semantic classification.

## 検証

Add/update source policy so field_access.rs must consume compiler_memory_type_of_type and must not reintroduce StructConstructorPolicy-based compiler-memory field classification. Run focused field-access regressions plus static-check boundary policy.

## 対応

2026-05-16 に修正した。`field_access.rs` は `resource_primitives::compiler_memory_type_of_type` を通して TypeCtx の証明済み compiler memory identity を読み、`CompilerMemoryType` から `RestrictedStructConstructor` へ enum `match` で変換する。`StructConstructorPolicy` と struct name map による compiler memory field classification は削除した。

さらに、restricted compiler memory type の base field access は field name validation より前に boundary gate を実行するようにした。これにより、`get token "ptr"` のような存在しない field 名でも `generic struct has no field` ではなく `type.owner_token.field_access_restricted` が先に出る。

## 回帰テスト

- `cargo fmt -p nepl-core --check`: passed
- `cargo test -p nepl-core --test resource_ir typecheck_rejects_region_token_field_access_outside_memory_boundary -- --exact --nocapture`: passed
- `cargo test -p nepl-core --test resource_ir typecheck_rejects_mem_ptr_field_access_outside_memory_boundary -- --exact --nocapture`: passed
- `cargo test -p nepl-core --test resource_ir typecheck_allows_user_struct_named_mem_ptr_field_access -- --exact --nocapture`: passed
- `cargo test -p nepl-core --test resource_ir typecheck_allows_user_struct_named_region_token_field_access -- --exact --nocapture`: passed
- `cargo check -p nepl-core`: passed
- `node nodesrc/test_static_check_boundary_responsibility.js`: passed
- `node nodesrc/issues.js check --dir issues`: passed
- `git diff --check`: passed
