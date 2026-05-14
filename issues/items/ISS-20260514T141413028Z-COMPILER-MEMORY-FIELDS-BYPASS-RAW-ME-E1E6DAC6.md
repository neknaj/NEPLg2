---
id: ISS-20260514T141413028Z-COMPILER-MEMORY-FIELDS-BYPASS-RAW-ME-E1E6DAC6
title: "Compiler memory fields bypass raw-memory boundary"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-14
updated: 2026-05-14
target: "nepl-core/src/typecheck/field_access.rs, nepl-core/src/diagnostic_codes.rs, tests/stdlib/memory_safety.n.md, nepl-core/tests/resource_ir.rs, nodesrc/test_static_check_boundary_responsibility.js"
---

# ISS-20260514T141413028Z-COMPILER-MEMORY-FIELDS-BYPASS-RAW-ME-E1E6DAC6: Compiler memory fields bypass raw-memory boundary

## 概要

MemPtr and RegionToken direct constructors are restricted to the compiler raw-memory boundary, but field access still resolves compiler memory structs by normal struct shape. User source can project MemPtr.raw or RegionToken.ptr/size before the compiler applies the same capability boundary.

## 対象

- `nepl-core/src/typecheck/field_access.rs, nepl-core/src/diagnostic_codes.rs, tests/stdlib/memory_safety.n.md, nepl-core/tests/resource_ir.rs, nodesrc/test_static_check_boundary_responsibility.js`

## 根拠

- `StructConstructorPolicy::RawMemoryBoundaryOnly` は `MemPtr` / `RegionToken` の direct constructor を raw-memory boundary 外で拒否していた。
- 一方で `typecheck/field_access.rs` は struct field access を型の field shape だけで解決しており、同じ compiler memory struct の `raw` / `ptr` / `size` field を通常 struct field と同じ扱いにしていた。
- Resource IR owner checker は forged owner の使用時には拒否できるが、typed HIR に representation field projection を残すこと自体が compiler-issued capability boundary として弱い。
- `stdlib/core/mem/types.nepl` の representation helper は compiler memory type 定義モジュールの責務であり、user source に同じ field projection を許す理由にはならない。

## 問題

MemPtr and RegionToken direct constructors are restricted to the compiler raw-memory boundary, but field access still resolves compiler memory structs by normal struct shape. User source can project MemPtr.raw or RegionToken.ptr/size before the compiler applies the same capability boundary.

## 影響

Raw pointer identity and owner-token internals can leak through ordinary field projection diagnostics instead of compiler-owned boundary checks, weakening the type-level proof that safe source cannot inspect or reassemble raw memory representation.

## 修正方針

Gate field access for struct definitions whose constructor policy is RawMemoryBoundaryOnly. Emit distinct typed diagnostics for raw pointer field access and owner token field access outside the raw-memory boundary while leaving ordinary user structs with the same names unaffected.

## 検証

Add Rust and n.md compile_fail regressions for MemPtr.raw and RegionToken.ptr field projection outside the boundary, and extend the static-check responsibility policy so the field access path must inspect StructConstructorPolicy with exhaustive RestrictedStructConstructor matches.

## 対応

- `TypeDiagnosticCode` に `OwnerTokenFieldAccessRestricted` / `RawPointerFieldAccessRestricted` を追加し、`type.owner_token.field_access_restricted` / `type.raw_pointer.field_access_restricted` として階層化した。
- `typecheck/field_access.rs` で field access 対象の resolved struct definition を `StructConstructorPolicy` から判定し、`RawMemoryBoundaryOnly(OwnerToken|RawPointer)` の場合は raw-memory boundary 外の projection を拒否するようにした。
- `stdlib/core/mem/types.nepl` の representation helper は raw boundary ではなく `CompilerMemoryTypeDefinition(OwnerToken|RawPointer)` capability を持つ source として許可し、通常 user source が同じ intrinsic / field accessor を使って自動的に boundary 扱いになる抜け道は作らない形にした。
- `field_apply.rs` の `get` / `get_ref` / `put` 特殊化経路でも、overload failure に落とさず専用 diagnostic を出すようにした。
- Rust integration test と `tests/stdlib/memory_safety.n.md` に、compiler memory field projection 拒否と同名 user struct の field access 許可を追加した。
- `nodesrc/test_static_check_boundary_responsibility.js` で、field access 経路が constructor policy と definition capability を見て、`RestrictedStructConstructor` を明示的な `match` で分岐することを監視するようにした。

## 検証結果

- `cargo check -p nepl-core`
- `cargo fmt --package nepl-core --check`
- `cargo test -p nepl-core --test resource_ir field_access -- --nocapture`
- `cargo test -p nepl-core diagnostic_codes -- --nocapture`
- `node nodesrc/test_static_check_boundary_responsibility.js`
- `trunk build`
- `node nodesrc/tests.js -i tests/stdlib/memory_safety.n.md --no-tree -o tmp/agent1-field-boundary-memory-safety.json -j 1 --dist web/dist --assert-io`: total=32, passed=32

## 関連ドキュメント

- [NEPLg2 静的検査の複雑化解消計画](../../doc/neplg2/static_check_complexity_reduction_plan.md) Stage 4: Resource IR owner/provenance 分離。
