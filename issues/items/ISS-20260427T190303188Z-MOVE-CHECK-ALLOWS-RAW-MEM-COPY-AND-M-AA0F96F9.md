---
id: ISS-20260427T190303188Z-MOVE-CHECK-ALLOWS-RAW-MEM-COPY-AND-M-AA0F96F9
title: "move_check allows raw mem_copy and mem_move to duplicate non-Copy payloads"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-04-27
updated: 2026-04-28
target: "nepl-core/src/passes/move_check.rs, tests/compiler/move_effect.n.md"
---

# ISS-20260427T190303188Z-MOVE-CHECK-ALLOWS-RAW-MEM-COPY-AND-M-AA0F96F9: move_check allows raw mem_copy and mem_move to duplicate non-Copy payloads

## 概要

Raw mem_copy/mem_move operate on byte ranges, but move_check does not classify them as ownership events. A live initialized non-Copy raw place can be copied or moved as bytes to another raw range without consuming the source or checking the destination.

## 対象

- `nepl-core/src/passes/move_check.rs, tests/compiler/move_effect.n.md`

## 根拠

- `move_check` の raw memory event は `load` / `store` / `dealloc` / `realloc` を扱っていたが、raw `mem_copy` / `mem_move` は通常 call として argument visit だけで終わっていた。
- raw `mem_copy` / `mem_move` は byte range 操作であり、source の owning value を consume せず destination を初期化または上書きできる。
- 修正前の `tmp/raw-mem-copy-duplicates-noncopy.nepl` では `store<LocalToken> src ...` の直後に `mem_copy dst src size_of<LocalToken>` を呼んでも compiler が成功していた。

## 問題

Raw mem_copy/mem_move operate on byte ranges, but move_check does not classify them as ownership events. A live initialized non-Copy raw place can be copied or moved as bytes to another raw range without consuming the source or checking the destination.

## 影響

This bypasses the compiler's move model by creating shallow duplicates of owning values or overwriting live payloads, which can produce leaks, double drops, and invalid Resource IR assumptions.

## 修正方針

Classify raw mem_copy/mem_move calls in move_check. Reject bulk copy/move when the source range contains initialized or possibly moved non-Copy raw places, and reject writes when the destination range overlaps a live non-Copy raw place. Keep Copy-only byte operations and storage-only ranges allowed.

## 検証

Add compile_fail regressions for raw mem_copy/mem_move over live non-Copy source and destination ranges, plus passing regressions after load consumes the payload or for Copy payloads.

## 対応結果

- `move_check` の raw memory call 分類に `BulkCopy` を追加した。
- raw `mem_copy` / `mem_move` の i32 destination/source/size を canonical raw place range として解釈し、source range と destination range の live non-Copy state を検査するようにした。
- source range に `Initialized` / `PossiblyMoved` の non-Copy raw place が残る場合は `D3100` で拒否する。
- destination range に `Initialized` / `PossiblyMoved` の non-Copy raw place が残る場合も `D3100` で拒否する。
- `load<T>` で payload ownership を消費した後の storage-only bulk copy と、Copy bytes の bulk copy は許可する。

## 実施した検証

- `cargo fmt --check`: pass
- `cargo check -p nepl-core`: pass
- `trunk build`: pass
- `node nodesrc/tests.js -i tests/compiler/move_effect.n.md --no-tree -o tmp/raw-bulk-copy-live-noncopy-node.json -j 1`: `total=57`, `passed=57`
- 修正前再現ファイル `tmp/raw-mem-copy-duplicates-noncopy.nepl` は修正後 `D3100` で拒否されることを確認した。
