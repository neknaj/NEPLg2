---
id: ISS-20260505T010829802Z-WASIX-GET-TERMINAL-SIZE-DEALLOCATES--A1302F57
title: "WASIX get_terminal_size deallocates maybe-freed tty state"
area: stdlib
status: fixed
resolved: true
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

- `node nodesrc/tests.js -i tests/stdlib/features_tui.n.md --no-tree -o tmp/features-tui-before-wasix-state-owner-agent1.json -j 1 --dist web/dist` で、`doctest#2` が `resource.owner.maybe_freed` により compile failure した。
- 診断は `get_terminal_size__unit__TerminalSize__imp` の `Dealloc` で、`state` が `MaybeFreed { storage: None }` になっていた。
- `get_tty_state` は成功時に raw owner pointer を返し、失敗時に同じ関数内で解放して `0` sentinel を返していた。caller は `ne state 0` で分岐していたが、raw owner と non-owner sentinel が同じ `i32` 戻り値に混ざるため、Resource IR は成功 branch だけに owner が存在することを証明できなかった。

## 問題

features_tui doctest#2 now reaches Resource IR and reports resource.owner.maybe_freed plus maybe_leak in get_terminal_size: the tty state allocation/deallocation path leaves state MaybeFreed before dealloc_raw state 24.

## 影響

TUI terminal-size checks cannot compile under the stricter Resource IR owner model, and fixing the test by suppressing diagnostics would hide a real maybe-free ownership path.

## 修正方針

Review get_terminal_size allocation, tty_get error handling, and state cleanup so the raw state owner has exactly one live/free path. Keep TTY host fallback behavior but make ownership explicit enough for Resource IR.

## 検証

`features_tui.n.md::doctest#2` が `get_terminal_size` の回帰対象である。`features_tui.n.md` 全体には stdout assertion report 移行の別 issue が残っているため、この issue の完了判定は terminal size doctest を focused 実行して行う。

## 2026-05-05 対応結果

- `get_tty_state` の戻り値を raw `i32` sentinel から `Result<i32,i32>` に変更した。
- `Result::Ok state` だけが 24 byte TTY state buffer の raw owner を返し、`Result::Err errno` は関数内で buffer を解放済みの non-owner path として表すようにした。
- `enter_raw_mode` と `get_terminal_size` は `match get_tty_state` に変更し、`Ok` arm だけで `state` を読み取り・解放する構造にした。
- `get_tty_state` の nm comment に、成功 owner path、失敗 cleanup path、`Ok state` caller の `dealloc_raw state 24` 義務を明記した。

## 2026-05-05 検証

- `node nodesrc/run_doctest.js -i tests/stdlib/features_tui.n.md -n 2 --dist web/dist`: passed
- `node nodesrc/tests.js -i tests/stdlib/features_tui.n.md --no-tree -o tmp/features-tui-after-wasix-state-result-agent1.json -j 1 --dist web/dist`: 現在の workspace では total=4, passed=4, failed=0。ただし doctest#4 の stdout report 差分は別 issue の未 commit 差分を含むため、この issue の commit 対象には含めない。
- `node nodesrc/test_stdlib_match_decision_trees.js`: passed
- `node nodesrc/tests.js -i stdlib/platforms/wasix/tui.nepl --no-tree -o tmp/wasix-tui-after-state-result-agent1.json -j 1 --dist web/dist`: no runnable doctests collected。`tui.nepl` 本体には runnable doctest が無いため、検証対象は `features_tui.n.md::doctest#2` とした。
