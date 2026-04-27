---
id: ISS-20260427T201910047Z-MOVE-CHECK-LOSES-RAW-ALIASES-RETURNE-85E61EC1
title: "move_check loses raw aliases returned from functions"
area: core
status: verified
resolved: true
priority: P1
type: bug
created: 2026-04-27
updated: 2026-04-28
target: nepl-core/src/passes/move_check.rs
---

# ISS-20260427T201910047Z-MOVE-CHECK-LOSES-RAW-ALIASES-RETURNE-85E61EC1: move_check loses raw aliases returned from functions

## 概要

Raw alias tracking is local to expressions and variables. When a function returns a MemPtr/RegionToken or an aggregate containing one, the caller does not recover the returned provenance unless the callee is one of several hard-coded core/mem helper names.

## 対象

- `nepl-core/src/passes/move_check.rs`
- `tests/compiler/move_effect.n.md`

## 根拠

- 修正前の `tmp/function-return-memptr-alias-double-load.nepl` では、`id_ptr p` が返した `q` と元の `p` が同じ `MemPtr<LocalToken>` 由来であるにもかかわらず、`move_check` が関数戻り値の raw alias を復元できず、同じ raw place の二重 `load<LocalToken>` が exit 0 で受理された。
- `raw_memory_place_key_from_mem_ptr` / `raw_memory_place_key_from_region_token` は `mem_ptr_wrap`、`mem_ptr_add`、`region_ptr` など既知の core/mem helper だけを式構造から特別扱いしており、通常の user function call では provenance を保持していなかった。
- aggregate field alias と enum payload alias は caller の local state には保存されるが、callee の戻り値がそれらを含む場合に caller で再インスタンス化する仕組みがなかった。

## 問題

Raw alias tracking is local to expressions and variables. When a function returns a MemPtr/RegionToken or an aggregate containing one, the caller does not recover the returned provenance unless the callee is one of several hard-coded core/mem helper names.

## 影響

Self-host and stdlib helper APIs can hide owner/storage pointers behind ordinary functions. Non-Copy raw memory can then be loaded twice through the original pointer and a function-returned alias, bypassing move_check raw ownership state.

## 修正方針

Add conservative interprocedural raw alias summaries for monomorphized functions. Summaries should describe raw return aliases, aggregate-field aliases, enum payload aliases, and enum-payload aggregate aliases in terms of function parameters, then instantiate them at call sites.

## 検証

- `cargo fmt --check`
- `cargo check -p nepl-core`
- `trunk build`
- `node nodesrc/tests.js -i tests/compiler/move_effect.n.md --no-tree -o tmp/function-return-raw-alias-node.json -j 1`: total 85 / passed 85
- 修正後の `tmp/function-return-memptr-alias-double-load.nepl` は D3100 で拒否されることを確認した。
