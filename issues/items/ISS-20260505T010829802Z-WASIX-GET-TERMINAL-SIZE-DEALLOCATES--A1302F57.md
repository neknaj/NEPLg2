---
id: ISS-20260505T010829802Z-WASIX-GET-TERMINAL-SIZE-DEALLOCATES--A1302F57
title: "WASIX get_terminal_size deallocates maybe-freed tty state"
area: stdlib
status: open
resolved: false
priority: P1
type: bug
created: 2026-05-05
updated: 2026-05-05
target: "stdlib/platforms/wasix/tui.nepl, tests/stdlib/features_tui.n.md"
---

# ISS-20260505T010829802Z-WASIX-GET-TERMINAL-SIZE-DEALLOCATES--A1302F57: WASIX get_terminal_size deallocates maybe-freed tty state

## 概要

features_tui doctest#2 now reaches Resource IR and reports resource.owner.maybe_freed plus maybe_leak in get_terminal_size: the tty state allocation/deallocation path leaves state MaybeFreed before dealloc_raw state 24.

## 対象

- `stdlib/platforms/wasix/tui.nepl, tests/stdlib/features_tui.n.md`

## 根拠

- 未記入

## 問題

features_tui doctest#2 now reaches Resource IR and reports resource.owner.maybe_freed plus maybe_leak in get_terminal_size: the tty state allocation/deallocation path leaves state MaybeFreed before dealloc_raw state 24.

## 影響

TUI terminal-size checks cannot compile under the stricter Resource IR owner model, and fixing the test by suppressing diagnostics would hide a real maybe-free ownership path.

## 修正方針

Review get_terminal_size allocation, tty_get error handling, and state cleanup so the raw state owner has exactly one live/free path. Keep TTY host fallback behavior but make ownership explicit enough for Resource IR.

## 検証

node nodesrc/tests.js -i tests/stdlib/features_tui.n.md --no-tree -o tmp/features-tui-after-tui-state-owner.json -j 1 --dist web/dist
