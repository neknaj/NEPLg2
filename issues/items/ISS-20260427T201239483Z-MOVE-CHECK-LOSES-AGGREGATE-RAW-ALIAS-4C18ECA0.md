---
id: ISS-20260427T201239483Z-MOVE-CHECK-LOSES-AGGREGATE-RAW-ALIAS-4C18ECA0
title: "move_check loses aggregate raw aliases stored in enum payloads"
area: core
status: verified
resolved: true
priority: P1
type: bug
created: 2026-04-27
updated: 2026-04-28
target: nepl-core/src/passes/move_check.rs
---

# ISS-20260427T201239483Z-MOVE-CHECK-LOSES-AGGREGATE-RAW-ALIAS-4C18ECA0: move_check loses aggregate raw aliases stored in enum payloads

## 概要

Aggregate field raw aliases are now tracked for struct/tuple variables, but when an aggregate containing a MemPtr/RegionToken is stored as an enum payload and later recovered by match binding, the aggregate field alias map is not restored for the bind local.

## 対象

- `nepl-core/src/passes/move_check.rs`
- `tests/compiler/move_effect.n.md`

## 根拠

- 修正前の `tmp/enum-aggregate-field-memptr-alias-double-load.nepl` では、`Result<PtrHolder,str>::Ok holder` を `match` して得た `h` から `field::get h "ptr"` で `MemPtr` を取り出すと、元の `p` と同じ raw place であることを `move_check` が復元できず、二重 `load<LocalToken>` が exit 0 で受理された。
- 既存の enum payload alias tracking は direct `MemPtr` / `RegionToken` payload の raw alias だけを variant 名に保存しており、payload が aggregate の場合の field offset -> raw place map を保存していなかった。
- branch merge でも enum payload 内 aggregate alias map を扱っていなかったため、全 branch で同じ `Result::Ok holder` を代入した後でも provenance が失われていた。

## 問題

Aggregate field raw aliases are now tracked for struct/tuple variables, but when an aggregate containing a MemPtr/RegionToken is stored as an enum payload and later recovered by match binding, the aggregate field alias map is not restored for the bind local.

## 影響

Result/Option wrappers can hide owner/storage pointers kept in records. Self-host APIs that return aggregate handles in Result can bypass raw memory ownership checks and double-load non-Copy values from the same raw place.

## 修正方針

Add enum-payload aggregate field raw alias tracking alongside direct enum payload raw aliases. Save aggregate alias maps on let/set from enum construction, merge them across branches only when identical, and restore them on match bind.

## 検証

- `cargo fmt --check`
- `cargo check -p nepl-core`
- `trunk build`
- `node nodesrc/tests.js -i tests/compiler/move_effect.n.md --no-tree -o tmp/enum-aggregate-field-raw-alias-node.json -j 1`: total 79 / passed 79
- 修正後の `tmp/enum-aggregate-field-memptr-alias-double-load.nepl` は D3100 で拒否されることを確認した。
