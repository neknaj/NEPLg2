---
id: ISS-20260517T033712430Z-WASIX-TTY-RAW-MODE-EXPOSES-RAW-I32-S-45184629
title: "WASIX TTY raw mode exposes raw i32 state owner"
area: stdlib
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-17
updated: 2026-05-17
target: "stdlib/platforms/wasix/tui/tty.nepl, nodesrc/test_stdlib_wasix_tui_no_unsafe_unwraps.js"
---

# ISS-20260517T033712430Z-WASIX-TTY-RAW-MODE-EXPOSES-RAW-I32-S-45184629: WASIX TTY raw mode exposes raw i32 state owner

## 概要

WASIX TUI TTY helpers allocate tty state with alloc_raw and expose the owned state as raw i32 pointers through get_tty_state_result, enter_raw_mode, and restore_mode.

## 対象

- `stdlib/platforms/wasix/tui/tty.nepl, nodesrc/test_stdlib_wasix_tui_no_unsafe_unwraps.js`

## 根拠

- `stdlib/platforms/wasix/tui/tty.nepl` は WASIX `tty_get` / `tty_set` 用の 24 byte state buffer を `alloc_raw 24` で確保し、`get_tty_state_result <()*>Result<i32,i32>`、`enter_raw_mode <()*>i32>`、`restore_mode <(i32)*>i32>` として public surface に raw `i32` owner を出していた。
- その raw `i32` は成功時の有効 pointer、失敗時 sentinel `0`、復元後に解放すべき owner を同じ型で表すため、Resource IR が free obligation owner と非 owner 値を型で区別できない。
- `nodesrc/test_stdlib_wasix_tui_no_unsafe_unwraps.js` の source policy も `Result<i32,i32>` までしか固定しておらず、Stage 6 の `MemPtr = non-owning pointer` / `RegionToken = free obligation owner` 分離に追従していなかった。

## 問題

WASIX TUI TTY helpers allocate tty state with alloc_raw and expose the owned state as raw i32 pointers through get_tty_state_result, enter_raw_mode, and restore_mode.

## 影響

Public TUI APIs can carry a free obligation as an untyped i32 sentinel, weakening the Stage 6 MemPtr non-owning / RegionToken owner split and making leaks or double frees hard for Resource IR to prove.

## 修正方針

Represent TTY state as an owner-backed TtyState struct containing RegionToken<u8>, keep raw address extraction inside the TTY raw boundary, and make public raw-mode APIs return/consume typed Result<TtyState,i32>.

## 検証

- `node nodesrc/tests.js -i tests/stdlib/features_tui.n.md --no-tree -o tmp/wasix-tui-tty-region-state-after.json -j 1`: 5 passed。
- `node nodesrc/test_stdlib_wasix_tui_no_unsafe_unwraps.js`: passed。
- `node nodesrc/issues.js check --dir issues`: passed。

## 対応内容

- `TtyState` を追加し、TTY state buffer の free obligation を `RegionToken<u8>` owner に保持させた。
- `get_tty_state_result` は private helper に戻し、成功時は `Result<TtyState,i32>::Ok state` を返すようにした。
- `enter_raw_mode` は `Result<TtyState,i32>` を返し、`restore_mode` は `TtyState` を消費して復元後に owner を解放するようにした。
- raw address 変換は `tty_state_raw` helper に閉じ、public API から `alloc_raw` / `dealloc_raw` / raw `i32` owner を消した。
- source policy を更新し、`TtyState` owner、`RegionToken<u8>` allocation、public raw-mode API の typed owner contract、`alloc_raw` / `dealloc_raw` 再導入禁止を監視するようにした。
