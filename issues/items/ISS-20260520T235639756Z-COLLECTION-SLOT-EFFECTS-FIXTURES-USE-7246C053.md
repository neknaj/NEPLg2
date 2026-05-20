---
id: ISS-20260520T235639756Z-COLLECTION-SLOT-EFFECTS-FIXTURES-USE-7246C053
title: "Collection slot effects fixtures used non-canonical compiler memory types"
area: core
status: fixed
resolved: true
priority: P1
type: test
created: 2026-05-20
updated: 2026-05-21
target: nepl-core/tests/effects.rs
---

# ISS-20260520T235639756Z-COLLECTION-SLOT-EFFECTS-FIXTURES-USE-7246C053: Collection slot effects fixtures used non-canonical compiler memory types

## 概要

collection slot lifecycle effects tests defined local same-shape `MemPtr` / `RegionToken` inside arbitrary stdlib paths, so canonical compiler-memory definition filtering correctly rejected them after owner-token authority was bound to `core/mem/types.nepl`.

## 対象

- `nepl-core/tests/effects.rs`

## 根拠

- [ISS-20260520T224333208Z-COMPILER-MEMORY-OWNER-TOKEN-AUTHORIT-7FCBF376](./ISS-20260520T224333208Z-COMPILER-MEMORY-OWNER-TOKEN-AUTHORIT-7FCBF376.md) で compiler-memory type definition capability は canonical `core/mem/types.nepl` に限定された。
- その後 `cargo test -p nepl-core collection_slot_lifecycle --test effects -- --test-threads=1` を実行すると、local same-shape `MemPtr` を使う acceptance fixture が `IntrinsicArgTypeMismatch` で失敗した。
- fixture の意図は collection slot intrinsic の anchor validation であり、fake compiler-memory type を許可することではない。

## 問題

collection slot lifecycle effects tests defined local same-shape `MemPtr` / `RegionToken` inside arbitrary stdlib paths, so canonical compiler-memory definition filtering correctly rejected them after owner-token authority was bound to `core/mem/types.nepl`.

## 影響

The tests no longer exercised the intended collection intrinsic anchor checks and one acceptance test failed after the compiler was made stricter; leaving this fixture would hide whether collection slot intrinsics are validated against canonical compiler memory types.

## 修正方針

Use temporary stdlib fixtures with canonical `core/mem/types.nepl` and import those types from the collection boundary source, then remove the obsolete non-canonical helper.

## 検証

- `cargo test -p nepl-core collection_slot --test effects -- --test-threads=1`
- `cargo check -p nepl-core`
- `cargo fmt --check -p nepl-core`
- `node nodesrc/issues.js check --dir issues`

## 2026-05-21 修正

- effects test fixture に temporary stdlib root を作成し、その中の canonical `core/mem/types.nepl` に `MemPtr` / `RegionToken` を定義する helper を追加した。
- collection slot lifecycle / storage dealloc / storage relocate の tests は、local same-shape type ではなく `#import "core/mem/types" as *` で canonical compiler-memory types を使うようにした。
- 未使用になった旧 `check_source_as_stdlib_path` helper は削除した。
