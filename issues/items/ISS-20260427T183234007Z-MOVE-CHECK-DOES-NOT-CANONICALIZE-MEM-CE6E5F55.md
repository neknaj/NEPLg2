---
id: ISS-20260427T183234007Z-MOVE-CHECK-DOES-NOT-CANONICALIZE-MEM-CE6E5F55
title: "move_check does not canonicalize MemPtr raw address aliases"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-04-27
updated: 2026-04-28
target: "nepl-core/src/passes/move_check.rs, tests/compiler/move_effect.n.md"
---

# ISS-20260427T183234007Z-MOVE-CHECK-DOES-NOT-CANONICALIZE-MEM-CE6E5F55: move_check does not canonicalize MemPtr raw address aliases

## 概要

move_check tracks i32 raw address aliases, but mem_ptr_addr(MemPtr<T>) results are not canonicalized to the same raw memory place. A program can call mem_ptr_addr on the same MemPtr twice and load a non-Copy value from both resulting i32 addresses without D3100.

## 対象

- `nepl-core/src/passes/move_check.rs, tests/compiler/move_effect.n.md`

## 根拠

- `nepl-core/src/passes/move_check.rs` の raw place tracking は i32 の `Var` / literal / `add` から place key を作っていた。
- `MemPtr<T>` は stdlib 上では `raw <i32>` を持つ typed address だが、`mem_ptr_wrap` / `mem_ptr_addr` の HIR call は raw place key へ反映されていなかった。
- そのため、同じ `MemPtr<T>` から `mem_ptr_addr` を複数回呼んだ場合でも、得られた i32 binding が別々の raw place として扱われた。

## 問題

move_check tracks i32 raw address aliases, but mem_ptr_addr(MemPtr<T>) results are not canonicalized to the same raw memory place. A program can call mem_ptr_addr on the same MemPtr twice and load a non-Copy value from both resulting i32 addresses without D3100.

## 影響

MemPtr is intended to be a typed raw pointer, but aliasing through mem_ptr_addr bypasses the existing raw place move tracking and can create multiple owners for a non-Copy value stored in the same memory cell.

## 修正方針

Teach raw place tracking to derive stable place keys through MemPtr values and mem_ptr_addr/mem_ptr_wrap calls, then add compile_fail regressions for repeated non-Copy loads through MemPtr-derived addresses.

## 検証

tests/compiler/move_effect.n.md rejects repeated non-Copy loads through mem_ptr_addr aliases with D3100 while existing raw i32 and MemPtr Copy field cases continue to pass.

## 対応結果

- `move_check` の raw place key 生成を `mem_ptr_addr` call に対応させた。
- `mem_ptr_wrap` call、`MemPtr` struct construct、`MemPtr` 変数コピーを同じ raw place alias に畳み込むようにした。
- 未知の `MemPtr` 変数は `$memptr:<binding>` という安定 key を使い、同じ binding 由来の `mem_ptr_addr` が同じ raw place として合流するようにした。
- `tests/compiler/move_effect.n.md` に同じ `MemPtr` と copy した `MemPtr` 由来 address の二重 non-Copy load compile_fail を追加した。

## 実施した検証

- `cargo fmt --check`: pass
- `cargo check -p nepl-core`: pass
- `trunk build`: pass
- `node nodesrc/tests.js -i tests/compiler/move_effect.n.md --no-tree -o tmp/memptr-raw-alias-move-node.json -j 1`: `total=44`, `passed=44`
