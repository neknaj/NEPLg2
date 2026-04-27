---
id: ISS-20260427T184214411Z-MOVE-CHECK-ALLOWS-RAW-DEALLOC-WITH-L-6543A0A2
title: "move_check allows raw dealloc with live non-Copy place"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-04-27
updated: 2026-04-28
target: "nepl-core/src/passes/move_check.rs, tests/compiler/move_effect.n.md"
---

# ISS-20260427T184214411Z-MOVE-CHECK-ALLOWS-RAW-DEALLOC-WITH-L-6543A0A2: move_check allows raw dealloc with live non-Copy place

## 概要

Raw memory ownership tracking records non-Copy store/load state, but dealloc_raw/dealloc/dealloc_ptr calls are not classified as raw memory operations. A live initialized raw place can therefore be returned to storage without consuming or dropping its payload.

## 対象

- `nepl-core/src/passes/move_check.rs, tests/compiler/move_effect.n.md`

## 根拠

- `nepl-core/src/passes/move_check.rs` は `RawMemoryCallKind` として `Load` / `Store` だけを分類しており、raw dealloc 系 call は通常 call として argument visit だけで終わっていた。
- `raw_place_states` には non-Copy `store<T>` による initialized state が残るが、`dealloc_raw` / `dealloc_ptr` はその state を参照していなかった。
- 修正前の `tmp/dealloc-raw-drops-noncopy.nepl` では `store<LocalToken> p ...` の直後に `dealloc_raw p size_of<LocalToken>` を呼んでも compiler が成功していた。

## 問題

Raw memory ownership tracking records non-Copy store/load state, but dealloc_raw/dealloc/dealloc_ptr calls are not classified as raw memory operations. A live initialized raw place can therefore be returned to storage without consuming or dropping its payload.

## 影響

This bypasses drop obligations at the compiler boundary: self-host code can leak owning values by freeing storage-only bytes, and future Resource IR cannot trust dealloc to mean the region is uninitialized.

## 修正方針

Classify raw dealloc APIs in move_check, canonicalize i32 and MemPtr addresses to raw places, and reject dealloc when the deallocated range still contains initialized or partially moved non-Copy raw places. Keep storage-only dealloc allowed after load/drop consumption.

## 検証

Add compile_fail regressions for dealloc_raw/dealloc_ptr on live non-Copy places and a passing regression for dealloc after load consumption.

## 対応結果

- `move_check` の raw memory call 分類に `Dealloc` を追加した。
- `dealloc_raw` / `dealloc` / `dealloc_ptr` / `__nepl_rt_dealloc` 系 call の第 1 引数を i32 raw address または `MemPtr<T>` raw place として正規化し、`raw_place_states` の live non-Copy state と照合するようにした。
- dealloc 対象範囲に `Initialized` または `PossiblyMoved` の non-Copy raw place が残っている場合は `D3100` を出すようにした。
- size が literal または direct `#intrinsic "size_of"` の場合は byte range で判定し、それ以外は同一 base の dealloc tail として安全側に判定する。
- `load<T>` で ownership を消費して `Moved` state になった raw place の storage-only dealloc は許可する。

## 実施した検証

- `cargo fmt --check`: pass
- `cargo check -p nepl-core`: pass
- `trunk build`: pass
- `node nodesrc/tests.js -i tests/compiler/move_effect.n.md --no-tree -o tmp/raw-dealloc-drop-obligation-node.json -j 1`: `total=47`, `passed=47`
- 修正前再現ファイル `tmp/dealloc-raw-drops-noncopy.nepl` は修正後 `D3100` で拒否されることを確認した。
