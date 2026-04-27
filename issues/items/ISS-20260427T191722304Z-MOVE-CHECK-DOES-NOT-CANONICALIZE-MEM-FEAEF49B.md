---
id: ISS-20260427T191722304Z-MOVE-CHECK-DOES-NOT-CANONICALIZE-MEM-FEAEF49B
title: "move_check does not canonicalize mem_ptr_add aliases"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-04-27
updated: 2026-04-28
target: "nepl-core/src/passes/move_check.rs, tests/compiler/move_effect.n.md"
---

# ISS-20260427T191722304Z-MOVE-CHECK-DOES-NOT-CANONICALIZE-MEM-FEAEF49B: move_check does not canonicalize mem_ptr_add aliases

## 概要

move_check canonicalizes mem_ptr_wrap, mem_ptr_addr, MemPtr copies, RegionToken projections, and raw i32 aliases, but mem_ptr_add is not part of the raw place model. mem_ptr_add p 0 can create a second MemPtr whose raw address is treated as an unrelated place, allowing duplicate non-Copy loads or cleanup checks to be bypassed.

## 対象

- `nepl-core/src/passes/move_check.rs, tests/compiler/move_effect.n.md`

## 根拠

- `raw_memory_place_key_from_mem_ptr` は `mem_ptr_wrap` / `region_ptr` / `get token "ptr"` / `MemPtr` construct を扱っていたが、`mem_ptr_add` を扱っていなかった。
- `raw_addr_alias_from_value` は `let q = mem_ptr_add p 0` に alias を設定できず、後続の `mem_ptr_addr q` は `$memptr:q` という別 place になっていた。
- 修正前の `tmp/memptr-add-alias-double-load.nepl` では、`p` と `q = mem_ptr_add p 0` から同じ `LocalToken` を二重 `load` しても compiler が成功していた。

## 問題

move_check canonicalizes mem_ptr_wrap, mem_ptr_addr, MemPtr copies, RegionToken projections, and raw i32 aliases, but mem_ptr_add is not part of the raw place model. mem_ptr_add p 0 can create a second MemPtr whose raw address is treated as an unrelated place, allowing duplicate non-Copy loads or cleanup checks to be bypassed.

## 影響

MemPtr pointer arithmetic can bypass raw ownership tracking, duplicating owning values or avoiding live-payload dealloc/realloc/write checks through an alias of the same storage.

## 修正方針

Teach move_check to derive raw place keys for mem_ptr_add by adding the offset to the base MemPtr raw place, and reuse that canonical key for raw loads, stores, dealloc/realloc/write checks, and copied aliases.

## 検証

Add compile_fail regressions for mem_ptr_add p 0 double non-Copy load and mem_ptr_add-based dealloc of a live non-Copy place, plus a passing offset case for disjoint storage.

## 対応結果

- `move_check` に `mem_ptr_add` helper 名の分類を追加した。
- `raw_memory_place_key_from_mem_ptr` が `mem_ptr_add base literal_offset` を `base raw place + offset` として正規化するようにした。
- `let q = mem_ptr_add p 0` の alias stack、`mem_ptr_addr q`、`dealloc_ptr q` などが同じ canonical raw place を参照するようになった。
- offset が literal ではなく raw place を確定できない場合は、既存通り未追跡に留める。
- `tests/compiler/move_effect.n.md` に `mem_ptr_add p 0` の二重 non-Copy load、same-place dealloc、disjoint offset 正常系を追加した。

## 実施した検証

- `cargo fmt --check`: pass
- `cargo check -p nepl-core`: pass
- `trunk build`: pass
- `node nodesrc/tests.js -i tests/compiler/move_effect.n.md --no-tree -o tmp/memptr-add-raw-alias-node.json -j 1`: `total=66`, `passed=66`
- 修正前再現ファイル `tmp/memptr-add-alias-double-load.nepl` は修正後 `D3100` で拒否されることを確認した。
