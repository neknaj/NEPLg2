---
id: ISS-20260512T202946482Z-TYPECHECK-CONSTRUCTOR-CAPABILITY-BOU-14965EAB
title: "Typecheck constructor capability boundary is not guarded by source policy"
area: core
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-12
updated: 2026-05-13
target: "nodesrc/test_static_check_boundary_responsibility.js; nepl-core/src/typecheck/model.rs; nepl-core/src/typecheck/driver.rs; nepl-core/src/typecheck/constructor_apply.rs"
---

# ISS-20260512T202946482Z-TYPECHECK-CONSTRUCTOR-CAPABILITY-BOU-14965EAB: Typecheck constructor capability boundary is not guarded by source policy

## 概要

MemPtr and RegionToken direct constructors are compiler capability boundaries: MemPtr must remain a raw pointer wrapper restricted to the memory boundary and RegionToken must remain an owner-token constructor restricted to the memory boundary. Rust focused tests cover current behavior, but the source policy does not assert the enum-first StructConstructorPolicy model, the raw-memory-boundary classification, or the exhaustive RestrictedStructConstructor diagnostic match.

## 対象

- `nodesrc/test_static_check_boundary_responsibility.js; nepl-core/src/typecheck/model.rs; nepl-core/src/typecheck/driver.rs; nepl-core/src/typecheck/constructor_apply.rs`

## 根拠

- `typecheck/model.rs` は `StructConstructorPolicy` / `RestrictedStructConstructor` enum で constructor capability を表す。
- `typecheck/driver.rs` は raw memory boundary 内で定義された `MemPtr` を `RawPointer`、`RegionToken` を `OwnerToken` として分類する。
- `typecheck/constructor_apply.rs` は direct constructor 呼び出し時に `raw_memory_boundary_allowed(span)` を確認し、許可外なら `type.raw_pointer.constructor_restricted` / `type.owner_token.constructor_restricted` を出す。
- ただし既存 source policy は、この boundary が enum-first かつ明示分岐で維持されていることを確認していなかった。

## 問題

MemPtr and RegionToken direct constructors are compiler capability boundaries: MemPtr must remain a raw pointer wrapper restricted to the memory boundary and RegionToken must remain an owner-token constructor restricted to the memory boundary. Rust focused tests cover current behavior, but the source policy does not assert the enum-first StructConstructorPolicy model, the raw-memory-boundary classification, or the exhaustive RestrictedStructConstructor diagnostic match.

## 影響

A later typecheck refactor could collapse the constructor policy back into name-based strings or a generic diagnostic path, allowing forged raw pointer or owner-token constructors to reappear without being caught by source policy.

## 修正方針

Extend the static-check boundary policy to require StructConstructorPolicy and RestrictedStructConstructor enums, raw-memory-boundary-only mapping for MemPtr/RegionToken, exhaustive diagnostic matching for OwnerToken and RawPointer, and no wildcard constructor-policy match arms.

## 検証

node nodesrc/test_static_check_boundary_responsibility.js; cargo test -p nepl-core constructor_restricted; node nodesrc/issues.js check --dir issues

## 2026-05-13 修正

`nodesrc/test_static_check_boundary_responsibility.js` に typecheck constructor capability boundary の source policy を追加した。

- `StructConstructorPolicy` と `RestrictedStructConstructor` enum の存在を監視する。
- `StructInfo` が `constructor_policy: StructConstructorPolicy` を保持することを監視する。
- `struct_constructor_policy` が `raw_memory_boundary_allowed(span.file_id)` に基づき、`MemPtr` を `RawPointer`、`RegionToken` を `OwnerToken` として分類することを監視する。
- `apply_struct_constructor` が `StructConstructorPolicy::RawMemoryBoundaryOnly(restricted)` で `raw_memory_boundary_allowed(span)` を確認することを監視する。
- `OwnerToken` と `RawPointer` が別々の diagnostic code に落ちることを監視する。

これにより、compiler-issued owner token / raw pointer capability の入口を通常 struct constructor に戻す refactor を source policy で検出できる。
