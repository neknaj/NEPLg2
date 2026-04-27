---
id: ISS-20260427T212724800Z-MOVE-CHECK-ALLOWS-MEMPTR-BYTE-WRITES-9D19BC9D
title: "move_check allows MemPtr byte writes to overwrite live non-Copy payloads"
area: core
status: verified
resolved: true
priority: P1
type: bug
created: 2026-04-27
updated: 2026-04-28
target: "nepl-core/src/passes/move_check.rs, tests/compiler/move_effect.n.md"
---

# ISS-20260427T212724800Z-MOVE-CHECK-ALLOWS-MEMPTR-BYTE-WRITES-9D19BC9D: move_check allows MemPtr byte writes to overwrite live non-Copy payloads

## 概要

raw byte write tracking rejects store_i32 on i32 raw addresses, but the MemPtr<i32> overload of store_i32 is not classified as a byte write. A MemPtr created from the same raw address can overwrite a live non-Copy raw place without D3100.

## 対象

- `nepl-core/src/passes/move_check.rs, tests/compiler/move_effect.n.md`

## 根拠

- `raw_memory_call_kind` は `store_i32` / `store_u8` / `memset_u8` / `fill_i32` / `mem_copy` / `mem_move` の raw write/copy 分類を i32 raw address 引数に限定していた。
- `stdlib/core/mem.nepl` には `MemPtr<i32>` / `MemPtr<u8>` overload の `store_i32` / `store_u8` / `fill_i32` / `memset_u8` と、`MemPtr<T>` overload の `mem_copy` / `mem_move` がある。
- 修正前の `node nodesrc/tests.js -i tests/compiler/move_effect.n.md --no-tree -o tmp/memptr-byte-write-regression-before.json -j 1` では、追加した `MemPtr<i32>` byte write regression が `expected compile_fail, but compiled successfully` になった。

## 問題

raw byte write tracking rejects store_i32 on i32 raw addresses, but the MemPtr<i32> overload of store_i32 is not classified as a byte write. A MemPtr created from the same raw address can overwrite a live non-Copy raw place without D3100.

## 影響

Typed MemPtr copy stores can corrupt initialized non-Copy payloads and bypass the compiler-owned raw memory ownership state.

## 修正方針

Classify MemPtr-backed store_i32/store_u8, memset/fill, and typed mem_copy/mem_move overloads as raw memory writes/copies by deriving their place keys through raw_dealloc_place_key/raw_memory_place_key_from_mem_ptr, then keep Copy storage and consumed payload cases valid.

## 解決内容

- raw memory call classification で i32 raw address だけでなく `MemPtr<T>` も raw place 引数として扱うようにした。
- `Store` / `ByteWrite` / `BulkCopy` の place key 取得を `raw_dealloc_place_key` に揃え、`MemPtr<T>` の provenance を `raw_memory_place_key_from_mem_ptr` 経由で使うようにした。
- typed `mem_copy<T>` / `mem_move<T>` overload の `count` は element count なので、`MemPtr<T>` の element type から byte size を計算するようにした。
- `MemPtr<i32>` の `store_i32` と `mem_copy<i32>` が live non-Copy raw place を上書きできない regression を追加した。

## 検証

- `node nodesrc/tests.js -i tests/compiler/move_effect.n.md --no-tree -o tmp/memptr-byte-write-regression-before.json -j 1`: 修正前 86 件中 1 件失敗
- `cargo fmt --check`: pass
- `cargo test -p nepl-core --test move_check`: 51/51 passed
- `trunk build`: pass
- `node nodesrc/tests.js -i tests/compiler/move_effect.n.md --no-tree -o tmp/memptr-byte-write-regression-after.json -j 1`: 87/87 passed
- `cargo check -p nepl-core --tests`: pass
