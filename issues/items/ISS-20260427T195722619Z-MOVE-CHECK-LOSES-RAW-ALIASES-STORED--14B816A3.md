---
id: ISS-20260427T195722619Z-MOVE-CHECK-LOSES-RAW-ALIASES-STORED--14B816A3
title: "move_check loses raw aliases stored in aggregate fields"
area: core
status: verified
resolved: true
priority: P1
type: bug
created: 2026-04-27
updated: 2026-04-28
target: nepl-core/src/passes/move_check.rs
---

# ISS-20260427T195722619Z-MOVE-CHECK-LOSES-RAW-ALIASES-STORED--14B816A3: move_check loses raw aliases stored in aggregate fields

## 概要

MemPtr/RegionToken raw-place aliases are preserved for direct variables and enum payloads, but aliases stored inside ordinary aggregate fields are not persisted. A MemPtr placed in a struct field and later retrieved with core/field::get is treated as an unrelated MemPtr.

## 対象

- `nepl-core/src/passes/move_check.rs`
- `tests/compiler/move_effect.n.md`

## 根拠

- 修正前の `tmp/struct-field-memptr-alias-double-load.nepl` では、`PtrHolder` の `ptr` field から取り出した `MemPtr` と元の `MemPtr` が同じ raw place を指すにもかかわらず、二重 `load<LocalToken>` が exit 0 で受理された。
- monomorphize 後の HIR では `core/field::get` が `Intrinsic { name: "load", ... }` に下がっており、既存の call 名ベースの `get` 判定では aggregate field projection を復元できなかった。
- さらに `field_move_path_from_addr` が generic 適用後の field type を `TypeId` 直比較していたため、`MemPtr<LocalToken>` field が同一型として認識されていなかった。

## 問題

MemPtr/RegionToken raw-place aliases are preserved for direct variables and enum payloads, but aliases stored inside ordinary aggregate fields are not persisted. A MemPtr placed in a struct field and later retrieved with core/field::get is treated as an unrelated MemPtr.

## 影響

Non-Copy values stored in raw memory can be loaded twice through the original pointer and the field-retrieved pointer, bypassing compiler-owned raw memory ownership checks. Self-host data structures that keep owner/storage pointers in records are especially affected.

## 修正方針

Track raw aliases stored in aggregate fields in MoveCheckContext, persist them through scope/snapshot/restore/branch merge, save them on let/set from struct/tuple construction, and restore them for field::get projection before raw memory checks. Field path detection must use semantic type equality instead of raw TypeId equality so generic aggregate fields are recognized.

## 検証

- `cargo fmt --check`
- `cargo check -p nepl-core`
- `trunk build`
- `node nodesrc/tests.js -i tests/compiler/move_effect.n.md --no-tree -o tmp/aggregate-field-raw-alias-node.json -j 1`: total 77 / passed 77
- 修正後の `tmp/struct-field-memptr-alias-double-load.nepl` は D3100 で拒否されることを確認した。
