---
id: ISS-20260427T185656579Z-MOVE-CHECK-ALLOWS-REALLOCATING-RAW-S-45B12E2B
title: "move_check allows reallocating raw storage with live non-Copy payload"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-04-27
updated: 2026-04-28
target: "nepl-core/src/passes/move_check.rs, tests/compiler/move_effect.n.md"
---

# ISS-20260427T185656579Z-MOVE-CHECK-ALLOWS-REALLOCATING-RAW-S-45B12E2B: move_check allows reallocating raw storage with live non-Copy payload

## 概要

Raw realloc APIs move/copy bytes from an old raw region to a new one, but move_check only checks load/store/dealloc ownership events. A live initialized non-Copy raw place can be passed to realloc_raw/realloc/realloc_ptr and moved as bytes without consuming or dropping the payload.

## 対象

- `nepl-core/src/passes/move_check.rs, tests/compiler/move_effect.n.md`

## 根拠

- `move_check` の raw memory event は `Load` / `Store` / `Dealloc` までで、`realloc_raw` / `realloc` / `realloc_ptr` を old range の ownership event として扱っていなかった。
- `realloc_raw` は旧領域の bytes を新領域へ移し、旧領域を storage として解放するため、initialized non-Copy payload が残っている場合は typed move/drop model を迂回する。
- 修正前の `tmp/realloc-raw-duplicates-noncopy.nepl` では `store<LocalToken> p ...` の直後に `realloc_raw p size_of<LocalToken> 32` を呼んでも compiler が成功していた。

## 問題

Raw realloc APIs move/copy bytes from an old raw region to a new one, but move_check only checks load/store/dealloc ownership events. A live initialized non-Copy raw place can be passed to realloc_raw/realloc/realloc_ptr and moved as bytes without consuming or dropping the payload.

## 影響

This can duplicate or invalidate owning values behind the compiler's move model, causing leaks, double-drops, or use-after-free assumptions in self-host storage code.

## 修正方針

Classify raw realloc APIs in move_check, canonicalize i32/MemPtr/RegionToken source places, and reject realloc when the old range still contains initialized or possibly moved non-Copy raw places. Allow realloc after payload ownership has been consumed.

## 検証

Add compile_fail regressions for realloc_raw and realloc_ptr on live non-Copy places and passing regression after load consumes the payload.

## 対応結果

- `move_check` の raw memory call 分類に `Realloc` を追加した。
- `realloc_raw` / `realloc` / `realloc_ptr` / `__nepl_rt_realloc` 系 call の第 1 引数を old range の raw place として正規化し、i32 raw address と `MemPtr<T>` を同じ検査へ通すようにした。
- old range に `Initialized` または `PossiblyMoved` の non-Copy raw place が残っている場合は `D3100` を出すようにした。
- `load<T>` で payload ownership を消費した後の storage-only realloc は許可する。

## 実施した検証

- `cargo fmt --check`: pass
- `cargo check -p nepl-core`: pass
- `trunk build`: pass
- `node nodesrc/tests.js -i tests/compiler/move_effect.n.md --no-tree -o tmp/raw-realloc-live-noncopy-node.json -j 1`: `total=52`, `passed=52`
- 修正前再現ファイル `tmp/realloc-raw-duplicates-noncopy.nepl` は修正後 `D3100` で拒否されることを確認した。
