---
id: ISS-20260507T171425909Z-MEMPTR-STRUCT-CONSTRUCTOR-IS-FORGEAB-7EC211C1
title: "MemPtr struct constructor is forgeable outside compiler memory boundary"
area: core
status: fixed
resolved: true
priority: P1
type: security
created: 2026-05-07
updated: 2026-05-07
target: "nepl-core/src/typecheck/driver.rs, nepl-core/src/typecheck/constructor_apply.rs, nepl-core/src/diagnostic_codes.rs, nepl-core/tests/resource_ir.rs, tests/stdlib/memory_safety.n.md"
---

# ISS-20260507T171425909Z-MEMPTR-STRUCT-CONSTRUCTOR-IS-FORGEAB-7EC211C1: MemPtr struct constructor is forgeable outside compiler memory boundary

## 概要

MemPtr is intended to be the typed non-owning pointer side of the memory model, but its ordinary struct constructor remains callable from user source. That lets safe source manufacture raw-pointer-shaped values without going through the compiler-owned core/mem wrapper boundary.

## 対象

- `nepl-core/src/typecheck/driver.rs, nepl-core/src/typecheck/constructor_apply.rs, nepl-core/src/diagnostic_codes.rs, nepl-core/tests/resource_ir.rs, tests/stdlib/memory_safety.n.md`

## 根拠

- `MemPtr<T>` は non-owning pointer を表す wrapper だが、`MemPtr raw` の通常 struct constructor は user source から直接呼べた。
- direct struct constructor は `mem_ptr_wrap` などの compiler-known helper call を通らないため、raw pointer construction が型検査上の通常 aggregate construction と同じ経路に混ざる。
- `RegionToken` direct constructor は `ISS-20260507T170021735Z-REGIONTOKEN-STRUCT-CONSTRUCTOR-IS-FO-0CC2D37A` で制限済みであり、同じ raw memory boundary policy を raw pointer wrapper へ拡張する必要があった。

## 問題

MemPtr is intended to be the typed non-owning pointer side of the memory model, but its ordinary struct constructor remains callable from user source. That lets safe source manufacture raw-pointer-shaped values without going through the compiler-owned core/mem wrapper boundary.

## 影響

Raw address provenance can enter typed HIR through an unclassified struct constructor path. Even when Resource IR later treats MemPtr as non-owning, the public constructor keeps raw pointer construction as a type-level escape hatch and weakens the raw memory boundary.

## 修正方針

Extend struct constructor policy so core memory-boundary MemPtr direct construction is restricted to raw-memory-boundary files, with a dedicated type.raw_pointer.constructor_restricted diagnostic. Keep mem_ptr_wrap as the explicit boundary wrapper while parent raw-address-escape issues continue tracking its public API migration.

## 検証

Add focused Rust and doctest regressions for direct MemPtr construction, keep a same-name user struct constructor allowed, run diagnostic registry/source-policy checks, and verify issue index.

## 修正結果

- `StructConstructorPolicy::RawMemoryBoundaryOnly` に `RestrictedStructConstructor::{OwnerToken,RawPointer}` を持たせ、制限理由を enum で分けた。
- core memory boundary 内で定義された `MemPtr` direct constructor を `RestrictedStructConstructor::RawPointer` として分類した。
- constructor 適用時に policy を `match` し、raw-memory-boundary capability 外では `type.raw_pointer.constructor_restricted` を出すようにした。
- 同名の user-defined `MemPtr` は `Public` policy のままなので、core raw pointer wrapper の制限が通常の user struct constructor へ波及しない。
- `mem_ptr_wrap` は今回の scope では explicit boundary wrapper として維持し、public raw address escape API の移行は親 issue 側に残した。

## 回帰テスト

- `cargo fmt --check -p nepl-core`: passed
- `cargo test -p nepl-core --test resource_ir typecheck_rejects_mem_ptr_struct_constructor_outside_memory_boundary -- --nocapture`: passed
- `cargo test -p nepl-core --test resource_ir typecheck_allows_user_struct_named_mem_ptr -- --nocapture`: passed
- `cargo test -p nepl-core --test resource_ir typecheck_rejects_region_token_struct_constructor_outside_memory_boundary -- --nocapture`: passed
- `cargo test -p nepl-core --test resource_ir typecheck_allows_user_struct_named -- --nocapture`: passed
- `trunk build`: passed
- `node nodesrc/tests.js -i tests/stdlib/memory_safety.n.md --no-tree -o tmp/agent1-memptr-constructor-boundary.json -j 1 --dist web/dist`: total=19, passed=19
- `node nodesrc/test_diagnostic_code_first_boundary.js`: passed
- `node nodesrc/issues.js check`: passed
- `git diff --check`: passed
