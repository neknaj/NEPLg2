---
id: ISS-20260516T095206737Z-OWNER-BACKED-AGGREGATE-ROOT-STILL-US-13A94EB8
title: "owner-backed aggregate root still uses constructor policy instead of proven owner-token identity"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-16
updated: 2026-05-16
target: "nepl-core/src/typecheck/copy_capability.rs; nodesrc/test_static_check_boundary_responsibility.js; doc/neplg2/static_check_complexity_reduction_plan.md"
---

# ISS-20260516T095206737Z-OWNER-BACKED-AGGREGATE-ROOT-STILL-US-13A94EB8: owner-backed aggregate root still uses constructor policy instead of proven owner-token identity

## 概要

Owner-backed aggregate detection uses StructConstructorPolicy::RawMemoryBoundaryOnly(OwnerToken) through StructInfo to recognize the compiler owner token root. After compiler memory type identity moved to SourceCapability-backed TypeCtx registration, this leaves one structural typecheck proof rooted in policy metadata instead of the proven TypeCtx identity.

## 対象

- `nepl-core/src/typecheck/copy_capability.rs; nodesrc/test_static_check_boundary_responsibility.js; doc/neplg2/static_check_complexity_reduction_plan.md`

## 根拠

- `nepl-core/src/typecheck/copy_capability.rs` の `target_is_compiler_owner_token` は、修正前に `StructInfo.constructor_policy` が `StructConstructorPolicy::RawMemoryBoundaryOnly(RestrictedStructConstructor::OwnerToken)` かどうかを見ていた。
- `ISS-20260516T083402106Z-COMPILER-MEMORY-TYPE-IDENTITY-IS-INF-3F3AF6C8` 以後、compiler memory type の semantic identity は SourceCapability-backed `TypeCtx::compiler_memory_type` に置く方針になっている。
- owner-backed aggregate 判定は `Vec<T>` などの stdlib 名 allowlist ではなく field 型から fixed-point で導く設計だが、その根だけが policy metadata を読んでいると、Resource IR / field access と proof boundary がずれる。

## 問題

Owner-backed aggregate detection uses StructConstructorPolicy::RawMemoryBoundaryOnly(OwnerToken) through StructInfo to recognize the compiler owner token root. After compiler memory type identity moved to SourceCapability-backed TypeCtx registration, this leaves one structural typecheck proof rooted in policy metadata instead of the proven TypeCtx identity.

## 影響

Owner-backed aggregate constructor and field-projection restrictions can drift from the compiler memory identity model. Future changes could keep Resource IR and field access on proven TypeCtx identity while aggregate propagation still trusts constructor policy/name metadata, weakening memory-safety auditability.

## 修正方針

Make target_is_compiler_owner_token query the central TypeCtx compiler owner-token identity through resource_primitives, and keep StructConstructorPolicy only for aggregate constructor policy propagation. Add source policy to reject RestrictedStructConstructor-based owner-token root classification in copy_capability.rs.

## 検証

Run focused owner-backed aggregate constructor/field regressions, same-name user RegionToken regression, cargo check, and static-check boundary policy.

## 対応

2026-05-16 に修正した。`copy_capability.rs` の `target_is_compiler_owner_token` は `resource_primitives::type_is_owner_token` を使い、TypeCtx の証明済み owner-token identity だけを owner-backed aggregate の semantic root とする。`StructConstructorPolicy` は owner-backed aggregate constructor policy の fixed-point propagation には残すが、compiler owner token 自体の分類には使わない。

`typecheck/driver.rs` の Copy impl target rejection も新しい `target_is_compiler_owner_token(&ctx, target_ty)` signature に合わせ、同名 user `RegionToken` は引き続き Copy impl 可能、compiler-owned `RegionToken<T>` は Copy impl 不可のまま維持した。

## 回帰テスト

- `cargo fmt -p nepl-core --check`: passed
- `cargo test -p nepl-core --test resource_ir typecheck_rejects_nested_owner_backed_aggregate_constructor_outside_boundary -- --exact --nocapture`: passed
- `cargo test -p nepl-core --test resource_ir typecheck_rejects_generic_owner_backed_aggregate_constructor_after_application -- --exact --nocapture`: passed
- `cargo test -p nepl-core --test resource_ir typecheck_allows_user_struct_named_region_token -- --exact --nocapture`: passed
- `cargo test -p nepl-core --test neplg2 copy_impl_rejects_compiler_owner_token_target -- --exact --nocapture`: passed
- `cargo test -p nepl-core --test neplg2 copy_impl_allows_user_struct_named_region_token -- --exact --nocapture`: passed
- `cargo check -p nepl-core`: passed
- `node nodesrc/test_static_check_boundary_responsibility.js`: passed
- `node nodesrc/issues.js check --dir issues`: passed
- `git diff --check`: passed
