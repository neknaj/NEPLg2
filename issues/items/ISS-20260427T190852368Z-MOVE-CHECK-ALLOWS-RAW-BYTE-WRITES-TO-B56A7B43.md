---
id: ISS-20260427T190852368Z-MOVE-CHECK-ALLOWS-RAW-BYTE-WRITES-TO-B56A7B43
title: "move_check allows raw byte writes to overwrite live non-Copy payloads"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-04-27
updated: 2026-04-28
target: "nepl-core/src/passes/move_check.rs, tests/compiler/move_effect.n.md"
---

# ISS-20260427T190852368Z-MOVE-CHECK-ALLOWS-RAW-BYTE-WRITES-TO-B56A7B43: move_check allows raw byte writes to overwrite live non-Copy payloads

## 概要

move_check rejects non-Copy store<T> over a live raw place, but copy-valued raw byte writes such as store_i32, memset_u8, and fill_i32 are not classified as writes to tracked raw ranges. They can overwrite initialized non-Copy payloads without consuming or dropping them.

## 対象

- `nepl-core/src/passes/move_check.rs, tests/compiler/move_effect.n.md`

## 根拠

- `move_check` は non-Copy `store<T>` を raw place initialization として扱っていたが、copy-valued raw `store_i32` / `store_u8` / `store<i32>` は通常 call または copy store として通していた。
- `memset_u8` / `fill_i32` も destination byte range を `raw_place_states` と照合していなかった。
- 修正前の `tmp/raw-byte-store-overwrites-noncopy.nepl` では `store<LocalToken> p ...` の直後に `store_i32 p 0` を呼んでも compiler が成功していた。

## 問題

move_check rejects non-Copy store<T> over a live raw place, but copy-valued raw byte writes such as store_i32, memset_u8, and fill_i32 are not classified as writes to tracked raw ranges. They can overwrite initialized non-Copy payloads without consuming or dropping them.

## 影響

This corrupts owning values behind the type and move model, allowing later loads/drops to observe invalid payloads and invalidating Resource IR assumptions about initialized raw places.

## 修正方針

Classify raw byte write helpers in move_check, derive their destination byte ranges, and reject writes that overlap initialized or possibly moved non-Copy raw places. Keep writes to Copy/untracked storage and writes after payload consumption allowed.

## 検証

Add compile_fail regressions for store_i32 and bulk fill/memset over live non-Copy places, plus passing regressions after load consumes the payload and for Copy storage.

## 対応結果

- raw store call を copy 値でも raw write event として扱うようにし、non-Copy `store<T>` は従来通り initialized state を作り、Copy store は destination range overwrite 検査を通すようにした。
- `store_i32` / `store_u8` の byte size を helper 名から求め、generic `store<T>` は `T` の storage size を使うようにした。
- `memset_u8` / `fill_u8` / `fill_i32` / `mem_fill` を raw byte write helper として分類し、destination range が live non-Copy raw place と重なる場合は `D3100` で拒否するようにした。
- direct intrinsic `#intrinsic "store"` の Copy value でも、live non-Copy raw place の上書きを拒否するようにした。

## 実施した検証

- `cargo fmt --check`: pass
- `cargo check -p nepl-core`: pass
- `trunk build`: pass
- `node nodesrc/tests.js -i tests/compiler/move_effect.n.md --no-tree -o tmp/raw-byte-write-live-noncopy-node.json -j 1`: `total=63`, `passed=63`
- 修正前再現ファイル `tmp/raw-byte-store-overwrites-noncopy.nepl` は修正後 `D3100` で拒否されることを確認した。

## 2026-04-28 MemPtr overload 追加対応

raw address 版の byte write は拒否済みだったが、`store_i32(MemPtr<i32>, i32)` などの typed `MemPtr` overload が raw write/copy 分類から漏れていた問題を `ISS-20260427T212724800Z-MOVE-CHECK-ALLOWS-MEMPTR-BYTE-WRITES-9D19BC9D` として分離し、修正した。`MemPtr` 経由の byte write / bulk copy も raw place provenance に接続され、live non-Copy payload と重なる場合は `D3100` になる。
